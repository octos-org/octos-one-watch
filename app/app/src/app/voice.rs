//! Small-screen voice capture and playback for the watch client.
//!
//! This intentionally uses Makepad's platform audio API directly instead of
//! `WindowVoiceInput`: that widget owns a local Whisper worker, while the watch
//! must send 16 kHz mono WAV to the Octos/OMiniX ASR path.

use makepad_widgets::makepad_platform::{
    audio::{AudioBuffer, AudioDeviceId, AudioDevicesEvent},
    permission::{Permission, PermissionResult, PermissionStatus},
    Cx, CxMediaApi, SignalToUI,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const TARGET_SAMPLE_RATE: u32 = 16_000;
const MAX_RECORDING_SECS: usize = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionOutcome {
    Granted,
    Denied,
}

#[derive(Default)]
struct CaptureBuffer {
    sample_rate: u32,
    samples: Vec<f32>,
    overflowed: bool,
}

#[derive(Default)]
struct PlaybackBuffer {
    sample_rate: u32,
    samples: Vec<f32>,
    position: f64,
    playing: bool,
}

/// Owns one Makepad audio-input callback and one audio-output callback.
/// Callback state is shared because Makepad invokes both on real-time threads.
pub struct WatchVoiceIo {
    callbacks_installed: bool,
    default_input: Option<AudioDeviceId>,
    default_output: Option<AudioDeviceId>,
    permission_request: Option<i32>,
    wants_recording: bool,
    capture_enabled: Arc<AtomicBool>,
    capture: Arc<Mutex<CaptureBuffer>>,
    playback: Arc<Mutex<PlaybackBuffer>>,
    playback_finished: Arc<AtomicBool>,
    playback_signal: SignalToUI,
}

impl Default for WatchVoiceIo {
    fn default() -> Self {
        Self {
            callbacks_installed: false,
            default_input: None,
            default_output: None,
            permission_request: None,
            wants_recording: false,
            capture_enabled: Arc::new(AtomicBool::new(false)),
            capture: Arc::new(Mutex::new(CaptureBuffer::default())),
            playback: Arc::new(Mutex::new(PlaybackBuffer::default())),
            playback_finished: Arc::new(AtomicBool::new(false)),
            playback_signal: SignalToUI::new(),
        }
    }
}

impl WatchVoiceIo {
    pub fn install_callbacks(&mut self, cx: &mut Cx) {
        if self.callbacks_installed {
            return;
        }

        let capture_enabled = self.capture_enabled.clone();
        let capture = self.capture.clone();
        cx.audio_input(0, move |info, input: &AudioBuffer| {
            if !capture_enabled.load(Ordering::Relaxed)
                || input.frame_count() == 0
                || input.channel_count() == 0
            {
                return;
            }
            let Ok(mut state) = capture.try_lock() else {
                return;
            };
            let rate = info.sample_rate.round().max(1.0) as u32;
            if state.sample_rate != rate {
                state.sample_rate = rate;
                state.samples.clear();
                state.overflowed = false;
            }
            let max_samples = rate as usize * MAX_RECORDING_SECS;
            let frames_left = max_samples.saturating_sub(state.samples.len());
            let frames = input.frame_count().min(frames_left);
            for frame in 0..frames {
                let mut mono = 0.0f32;
                for channel in 0..input.channel_count() {
                    mono += input.channel(channel)[frame];
                }
                state
                    .samples
                    .push(mono / input.channel_count() as f32);
            }
            if frames < input.frame_count() {
                state.overflowed = true;
            }
        });

        let playback = self.playback.clone();
        let playback_finished = self.playback_finished.clone();
        let playback_signal = self.playback_signal.clone();
        cx.audio_output(0, move |info, output: &mut AudioBuffer| {
            output.data.fill(0.0);
            let Ok(mut state) = playback.try_lock() else {
                return;
            };
            if !state.playing || state.samples.is_empty() || state.sample_rate == 0 {
                return;
            }

            let step = state.sample_rate as f64 / info.sample_rate.max(1.0);
            let frame_count = output.frame_count();
            for frame in 0..frame_count {
                let index = state.position.floor() as usize;
                if index >= state.samples.len() {
                    state.playing = false;
                    playback_finished.store(true, Ordering::Release);
                    playback_signal.set();
                    break;
                }
                let next = (index + 1).min(state.samples.len() - 1);
                let frac = (state.position - index as f64) as f32;
                let sample = state.samples[index] * (1.0 - frac) + state.samples[next] * frac;
                for channel in 0..output.channel_count() {
                    output.channel_mut(channel)[frame] = sample;
                }
                state.position += step;
            }
        });

        self.callbacks_installed = true;
    }

    pub fn handle_audio_devices(&mut self, cx: &mut Cx, devices: &AudioDevicesEvent) {
        self.default_input = devices.default_input().into_iter().next();
        self.default_output = devices.default_output().into_iter().next();
        if self.wants_recording && self.permission_request.is_none() {
            self.start_input_device(cx);
        }
        let playing = self.playback.lock().map(|state| state.playing).unwrap_or(false);
        if playing {
            self.start_output_device(cx);
        }
    }

    pub fn begin_recording(&mut self, cx: &mut Cx) {
        self.stop_playback(cx);
        self.install_callbacks(cx);
        self.wants_recording = true;
        if let Ok(mut state) = self.capture.lock() {
            *state = CaptureBuffer::default();
        }
        self.permission_request = Some(cx.request_permission(Permission::AudioInput));
        // Starting immediately is safe on already-granted devices and mirrors
        // Makepad's WindowVoiceInput behaviour. Android will gate the stream
        // until the permission result on first use.
        self.start_input_device(cx);
    }

    pub fn handle_permission_result(
        &mut self,
        cx: &mut Cx,
        result: &PermissionResult,
    ) -> Option<PermissionOutcome> {
        if result.permission != Permission::AudioInput
            || self.permission_request != Some(result.request_id)
        {
            return None;
        }
        self.permission_request = None;
        match result.status {
            PermissionStatus::Granted => {
                if self.wants_recording {
                    self.start_input_device(cx);
                }
                Some(PermissionOutcome::Granted)
            }
            PermissionStatus::DeniedCanRetry
            | PermissionStatus::DeniedPermanent
            | PermissionStatus::NotDetermined => {
                self.cancel_recording(cx);
                Some(PermissionOutcome::Denied)
            }
        }
    }

    fn start_input_device(&mut self, cx: &mut Cx) {
        if let Some(device) = self.default_input {
            self.capture_enabled.store(true, Ordering::Release);
            cx.use_audio_inputs(&[device]);
        }
    }

    pub fn cancel_recording(&mut self, cx: &mut Cx) {
        self.wants_recording = false;
        self.permission_request = None;
        self.capture_enabled.store(false, Ordering::Release);
        cx.use_audio_inputs(&[]);
        if let Ok(mut state) = self.capture.lock() {
            *state = CaptureBuffer::default();
        }
    }

    /// Stop capture, resample to 16 kHz mono and write a PCM16 WAV in the
    /// app-private Octos home. Returns `(path, byte_len, overflowed)`.
    pub fn finish_recording(&mut self, cx: &mut Cx) -> Result<(PathBuf, u64, bool), String> {
        self.wants_recording = false;
        self.permission_request = None;
        self.capture_enabled.store(false, Ordering::Release);
        cx.use_audio_inputs(&[]);

        let (rate, samples, overflowed) = {
            let mut state = self.capture.lock().map_err(|_| "voice capture lock poisoned")?;
            let rate = state.sample_rate;
            let samples = std::mem::take(&mut state.samples);
            let overflowed = state.overflowed;
            *state = CaptureBuffer::default();
            (rate, samples, overflowed)
        };
        if rate == 0 || samples.len() < rate as usize / 4 {
            return Err("录音太短，请至少说 0.25 秒".into());
        }

        let resampled = resample_linear(&samples, rate, TARGET_SAMPLE_RATE);
        let wav = encode_pcm16_wav(&resampled, TARGET_SAMPLE_RATE);
        let home = std::env::var("HOME").map_err(|_| "HOME 未设置")?;
        let dir = Path::new(&home).join("octos-home/voice");
        std::fs::create_dir_all(&dir).map_err(|e| format!("创建录音目录失败: {e}"))?;
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let path = dir.join(format!("watch-{stamp}.wav"));
        std::fs::write(&path, &wav).map_err(|e| format!("写入录音失败: {e}"))?;
        Ok((path, wav.len() as u64, overflowed))
    }

    pub fn play_wav_file(&mut self, cx: &mut Cx, path: &Path) -> Result<(), String> {
        self.install_callbacks(cx);
        // `file/attached` deliberately carries a workspace-relative path so
        // remote clients can fetch it through `/api/files`. In the embedded
        // stdio deployment the kernel workspace is its cwd,
        // `$APP_HOME/octos-home`, and the watch can read the file directly.
        let resolved = resolve_embedded_kernel_path(path)?;
        let bytes = std::fs::read(&resolved).map_err(|e| format!("读取语音回复失败: {e}"))?;
        let (sample_rate, samples) = decode_pcm_wav(&bytes)?;
        if samples.is_empty() {
            return Err("语音回复为空".into());
        }
        self.playback_finished.store(false, Ordering::Release);
        {
            let mut state = self.playback.lock().map_err(|_| "voice playback lock poisoned")?;
            *state = PlaybackBuffer {
                sample_rate,
                samples,
                position: 0.0,
                playing: true,
            };
        }
        self.start_output_device(cx);
        Ok(())
    }

    fn start_output_device(&mut self, cx: &mut Cx) {
        if let Some(device) = self.default_output {
            cx.use_audio_outputs(&[device]);
        }
    }

    pub fn stop_playback(&mut self, cx: &mut Cx) {
        if let Ok(mut state) = self.playback.lock() {
            *state = PlaybackBuffer::default();
        }
        self.playback_finished.store(false, Ordering::Release);
        cx.use_audio_outputs(&[]);
    }

    pub fn take_playback_finished(&mut self, cx: &mut Cx) -> bool {
        if self.playback_finished.swap(false, Ordering::AcqRel) {
            cx.use_audio_outputs(&[]);
            true
        } else {
            false
        }
    }
}

fn resolve_embedded_kernel_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let app_home = std::env::var("HOME").map_err(|_| "HOME 未设置".to_string())?;
    Ok(Path::new(&app_home).join("octos-home").join(path))
}

fn resample_linear(input: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if input.is_empty() || source_rate == 0 || target_rate == 0 {
        return Vec::new();
    }
    if source_rate == target_rate {
        return input.to_vec();
    }
    let output_len = ((input.len() as u64 * target_rate as u64) / source_rate as u64) as usize;
    let step = source_rate as f64 / target_rate as f64;
    let mut output = Vec::with_capacity(output_len);
    for i in 0..output_len {
        let position = i as f64 * step;
        let index = position.floor() as usize;
        let next = (index + 1).min(input.len() - 1);
        let frac = (position - index as f64) as f32;
        output.push(input[index] * (1.0 - frac) + input[next] * frac);
    }
    output
}

fn encode_pcm16_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_len).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&(sample_rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for sample in samples {
        let pcm = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
        out.extend_from_slice(&pcm.to_le_bytes());
    }
    out
}

fn decode_pcm_wav(bytes: &[u8]) -> Result<(u32, Vec<f32>), String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("TTS 回复不是有效 WAV".into());
    }
    let mut offset = 12usize;
    let mut format = None;
    let mut data = None;
    while offset + 8 <= bytes.len() {
        let id = &bytes[offset..offset + 4];
        let len = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().unwrap()) as usize;
        let start = offset + 8;
        let end = start.checked_add(len).ok_or("WAV chunk 长度溢出")?;
        if end > bytes.len() {
            return Err("WAV chunk 被截断".into());
        }
        if id == b"fmt " && len >= 16 {
            let audio_format = u16::from_le_bytes(bytes[start..start + 2].try_into().unwrap());
            let channels = u16::from_le_bytes(bytes[start + 2..start + 4].try_into().unwrap());
            let sample_rate = u32::from_le_bytes(bytes[start + 4..start + 8].try_into().unwrap());
            let bits = u16::from_le_bytes(bytes[start + 14..start + 16].try_into().unwrap());
            format = Some((audio_format, channels, sample_rate, bits));
        } else if id == b"data" {
            data = Some(&bytes[start..end]);
        }
        offset = end + (len & 1);
    }

    let (audio_format, channels, sample_rate, bits) = format.ok_or("WAV 缺少 fmt chunk")?;
    if audio_format != 1 || bits != 16 || channels == 0 {
        return Err(format!(
            "暂不支持的 WAV 格式: format={audio_format} channels={channels} bits={bits}"
        ));
    }
    let data = data.ok_or("WAV 缺少 data chunk")?;
    let channel_count = channels as usize;
    let frame_bytes = channel_count * 2;
    let mut samples = Vec::with_capacity(data.len() / frame_bytes);
    for frame in data.chunks_exact(frame_bytes) {
        let mut mono = 0.0f32;
        for channel in 0..channel_count {
            let at = channel * 2;
            let pcm = i16::from_le_bytes([frame[at], frame[at + 1]]);
            mono += pcm as f32 / 32768.0;
        }
        samples.push(mono / channel_count as f32);
    }
    Ok((sample_rate, samples))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_roundtrip_preserves_shape() {
        let source = vec![-1.0, -0.25, 0.0, 0.25, 1.0];
        let wav = encode_pcm16_wav(&source, 16_000);
        let (rate, decoded) = decode_pcm_wav(&wav).unwrap();
        assert_eq!(rate, 16_000);
        assert_eq!(decoded.len(), source.len());
        for (left, right) in source.iter().zip(decoded) {
            assert!((left - right).abs() < 0.0001);
        }
    }

    #[test]
    fn resample_has_expected_duration() {
        let input = vec![0.5; 48_000];
        let output = resample_linear(&input, 48_000, 16_000);
        assert_eq!(output.len(), 16_000);
    }
}
