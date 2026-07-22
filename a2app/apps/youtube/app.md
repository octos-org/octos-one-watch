# App: youtube — compact watch player

Route every video, music, live-stream, and watch/play request to this domain.
The watch client normally serves its hand-authored `youtube-watch` HTML card
directly so it cannot be truncated by the model. This contract defines the
fallback shape if generation is ever used.

## Output contract

- Emit exactly one `runhtml` fenced block and no prose.
- First line: `<!-- name: youtube-watch -->`.
- Produce a complete self-contained HTML document compatible with Chromium 83.
- Use the injected `octos.ytSearch(query)` Piped helper for live search and a
  known-good video result as the selectable fallback.

## Required watch UI

- Use two distinct states: a searchable result carousel and a dedicated player.
- Results are watch-native full-width cards: one large 16:9 thumbnail per
  screen, followed by at most two title lines and one channel line. Vertical
  swipes snap to the next result. Never use a phone-style thumbnail/text row.
- Selecting a result switches to the player; Back stops playback and restores
  the carousel.
- Give the entire document a 360px layout viewport at 0.5 initial scale on the
  OWW212. Inset the video inside a rounded, unclipped 16:9 safe area; never
  repair an iframe with a per-iframe CSS transform.
- Chromium 83 does not reliably lay out YouTube's small embedded controls. Use
  the YouTube IFrame API with `controls=0`, `enablejsapi=1`, and `playsinline=1`,
  plus large watch-owned Play/Pause controls outside the iframe. Playback must
  still begin from an explicit user tap so audio is allowed.
- One-column 372×430 layout, large touch targets, and a dark OLED background.
  Keep the native floating `+` available. The result screen must also expose an
  explicit Back/Exit action that invokes `ui.home` and returns to the app's
  blank compose state.
- Do not add comments, account sign-in, subscriptions, a mini-player, or a
  phone-style tab bar.
- Handle IFrame API error 101/150 explicitly: explain that the publisher blocks
  embedded playback and provide a large Back-to-results action.
- On Piped, iframe, or JavaScript failure, keep the fixed fallback selectable and
  show an explicit status message.
- Stop and unload the iframe on Back or when the document becomes hidden.
