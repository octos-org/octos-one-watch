//! Dedicated native WebView surface for the watch voice assistant.
//!
//! The WebView owns presentation only. Microphone capture, Octos transport and
//! reply playback remain in trusted Rust code. A separate browser id keeps this
//! document isolated from LLM-generated `runhtml` cards such as YouTube.

use makepad_widgets::{makepad_derive_widget::*, makepad_draw::*, widget::*};

script_mod! {
    use mod.prelude.widgets_internal.*

    mod.widgets.WatchVoiceWebViewBase = #(WatchVoiceWebView::register_widget(vm))

    mod.widgets.WatchVoiceWebView = set_type_default() do mod.widgets.WatchVoiceWebViewBase{
        width: Fill
        height: Fill
        draw_bg +: { color: #x000000 }
    }
}

pub fn voice_browser_id() -> SystemBrowserId {
    SystemBrowserId(live_id!(octos_watch_voice))
}

#[derive(Script, ScriptHook, Widget)]
pub struct WatchVoiceWebView {
    #[uid]
    uid: WidgetUid,
    #[source]
    source: ScriptObjectRef,
    #[walk]
    walk: Walk,
    #[redraw]
    #[live]
    draw_bg: DrawColor,
    #[visible]
    #[live(true)]
    visible: bool,
    #[rust]
    html: String,
    #[rust]
    loaded_html: String,
    #[rust]
    spawned: bool,
}

impl WatchVoiceWebView {
    fn load(&mut self, cx: &mut Cx) {
        if self.html.is_empty() || self.html == self.loaded_html {
            return;
        }
        let id = voice_browser_id();
        if !self.spawned {
            cx.system_browser(id).spawn("about:blank");
            self.spawned = true;
        }
        cx.system_browser(id)
            .set_html(&self.html, "https://octos-one.app/voice/");
        self.loaded_html.clone_from(&self.html);
        self.redraw(cx);
    }
}

impl Widget for WatchVoiceWebView {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        if !self.visible {
            return DrawStep::done();
        }
        let rect = cx.walk_turtle(walk);
        self.draw_bg.draw_abs(cx, rect);
        if self.spawned || !self.loaded_html.is_empty() {
            cx.system_browser(voice_browser_id())
                .update(self.draw_bg.area(), true);
        }
        DrawStep::done()
    }

    fn text(&self) -> String {
        self.html.clone()
    }

    fn set_text(&mut self, cx: &mut Cx, value: &str) {
        if self.html == value {
            return;
        }
        self.html.clear();
        self.html.push_str(value);
        self.load(cx);
    }
}
