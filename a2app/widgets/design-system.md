# Design system — global styles for ALL apps (enforced, WATCH square-screen edition)

This is the stylesheet: every app card MUST draw from these tokens. An app's
`app.md` defines structure and content; THIS file defines the look. Never
invent a new color or font size when a token fits.

This build targets **square watch screens** (~320×320 dp, 480×480 px class,
always-on OLED). Watch rules override phone habits: ONE column, generous
touch targets, restrained content volume, nothing tiny, nothing decorative
that costs battery. Dark theme (OLED power saving): true-black background.

## Color tokens

- `bg/base` `#000000` — true black (OLED pixels off), always the screen
- `surface/frosted` `#ffffff14` — stat cards, sheets (RoundedView + `new_batch: true`)
- `hairline` `#ffffff26` (separators, gridlines)
- `text/primary` `#ffffff` · `text/secondary` `#ffffff99` · `text/dim` `#ffffff73`
- `accent/positive` `#30d158` · `accent/negative` `#ff453a`

## Type scale (`draw_text.text_style.font_size`) — watch floor: NOTHING below 10

- `display` 34 — hero value (a price, a temperature); ONE hero per screen
- `title` 24 — screen title / symbol
- `heading` 18 — row primary text, stat values
- `body` 14 — change lines, buttons (watch minimum for tappable text)
- `caption` 12 — names, chips, eyebrows
- `micro` 10–11 — axis labels, stat labels (never below 10)

Pick FROM the scale; no in-between sizes.

## Spacing & layout (square, glanceable)

- Screen: root `SolidView{ width: Fill height: 460 flow: Overlay new_batch: true }`
  over `bg/base`, content `View` padded `Inset{left: 12 top: 12 right: 12 bottom: 56}`.
  A watch card is SHORT and SCROLLS: build one continuous `flow: Down` column —
  sections stack, nothing sits side-by-side except two-tile stat rows.
- Vertical rhythm: 10–14 between sections, 4–6 within a group. Radii: pills 10,
  cards/sheets 14 (rounded helps the square screen read softer).
- Rows: `flow: Right` + `align: Align{y: 0.5}`; space-between via `Filler{}`;
  list rows `height: Fit` with `padding: Inset{top: 12 bottom: 12}` — taller
  than phone; a row IS a touch target and must never clip.
- Separators: `SolidView{ width: Fill height: 1 }` hairline.
- FORBIDDEN on the watch: three-column grids, side-by-side map panes,
  full-bleed photo backdrops (unreadable at 320dp and costly on OLED),
  charts wider than they are tall, any text under 10pt, decorative vector art.

## Emphasis rules

- Directional values (price change, deltas): `accent/positive` when up,
  `accent/negative` when down — NEVER a green negative or red positive.
- Active selection (chips, tabs): bright text (`accent/positive` or
  `text/primary`) plus a 2dp underline bar (`SolidView{ width: Fill height: 2
  new_batch: true }` in the accent color) under the label; inactive =
  `text/dim` with a transparent 2dp spacer so the row height never shifts.
  **Do NOT use translucent filled pills for selection** — tinted fills inside
  nested overlays do not render reliably; the underline is the standard.

## Content volume (the watch edit)

- Every app shows its ESSENCE on the first 460dp: one hero block + one compact
  section. Everything else goes BELOW the scroll fold.
- Lists cap at 5 rows (phone shows 7–10). Forecasts show 3 days, not 7.
  Stat grids show 4 tiles (2×2), not 6. Chart height 96–120, never taller.
- Buttons/chips: minimum 44×44 touch area, text ≥ 14.
