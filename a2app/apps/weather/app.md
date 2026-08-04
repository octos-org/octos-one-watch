# Weather app (WATCH square-screen edition)

The DEFAULT for weather is a GLANCEABLE SQUARE WATCH CARD: true-black OLED
screen; the CURRENT conditions fill the first glance (city, hero temp,
condition glyph), then a compact 3-DAY FORECAST, then a 2×2 DETAIL GRID of
frosted tiles. NO photo backdrop, NO satellite/AQ map panes (unreadable at
320dp, costly on OLED), NO 7-day list, NO 6-tile grid — those are the phone
spec's volume, they do not fit a watch.

**YOU generate this card by ASSEMBLING the widget patterns** — there is no
exemplar to copy. Build it from THIS spec + `widgets/weather-icon.md`,
`widgets/containers.md`, `widgets/sys-helpers.md` (the data helpers ONLY — the
image helpers are unused on the watch), and `widgets/interaction.md`. Reproduce
EXACTLY this structure: a full-screen true-black root, then a Down column =
BLOCK: CURRENT, the 3-day forecast, then BLOCK: DETAIL-TILES (2×2).
`// name: weather-app` is the first line.

Composed "what should I DO in this weather" intents are NOT this card — they
route to `apps/weather-activity/app.md`, a composed app that reuses this
spec's `BLOCK: CURRENT`.

## LIVE DATA — MANDATORY (never hardcode weather numbers)

Every weather/air number in this card MUST come from a live data helper — you do
NOT know the real weather, so you must NEVER type an invented number. Use
`sys.weather(LAT, LON, "path")` and `sys.airquality(LAT, LON, "path")` (see
`widgets/sys-helpers.md`) as the `text` of each value `Label`, concatenating the
unit string, e.g. `text: sys.weather(LAT, LON, "current.temperature_2m") + "°"`.
Pass the city's REAL decimal lat/lon. A value shows "—" for a moment while it
loads, then the card auto-refreshes with the real reading. The ONLY things you
choose yourself are labels, the `WeatherIcon`/emoji condition, and the color
categories.

## BLOCK: WATCH-FRAME (the weather app's visual identity — reusable)

The frame every weather-family card sits in on the watch: a full-screen
`SolidView{ width: Fill height: 460 flow: Overlay new_batch: true draw_bg.color: #000000 }`
— true black (OLED pixels off), NO gradient, NO photo, NO scrim. Content
sections sit on `surface/frosted` `#ffffff14` RoundedViews (border_radius 14)
per the watch design system. Composed apps that reuse BLOCK: CURRENT MUST
reproduce THIS frame too.

## Layout rules

- The inner `flow: Down` column uses `padding: Inset{left: 12 top: 12 right: 12 bottom: 56}`.
- ONE column; nothing side-by-side except the two-tile stat rows.
- Text: `text/primary` `#ffffff`, secondary `#ffffff99`, dim `#ffffff73`.
- Narrow-watch budget: reference hardware is 372×430 px @ 320 dpi = **186×215 dp
  logical**. After column+row padding a row has ~140 dp — keep each row's
  FIXED-width sum well under that and give text `width: Fill` so it flexes.

## Structure, top to bottom

### BLOCK: CURRENT

(1) The current-conditions block, filling the first glance:
- City (font 20, `#ffffff99`).
- The hero temperature ALONE on its line (font 34, bold,
  `margin: Inset{top: 4 bottom: 0}` so its tall glyphs are not clipped) — its
  text is LIVE: `text: sys.weather(LAT, LON, "current.temperature_2m") + "°"`.
- A `flow: Right` row (height 44, align y 0.5, spacing 8) holding a
  `WeatherIcon{ draw_bg.cond: <N> width: 40 height: 40 }` followed by the
  condition `Label` (font 16). Pick `draw_bg.cond` by CURRENT condition:
  0 clear/sunny, 1 partly cloudy, 2 cloudy/overcast, 3 rain/drizzle, 4
  thunderstorm, 5 snow, 6 wind, 7 fog/haze/mist. (See `widgets/weather-icon.md`.)
- Then an `H:__°  L:__°` line (font 13, `#ffffff99`), every number LIVE:
  `"H:" + sys.weather(LAT, LON, "daily.temperature_2m_max.0") + "°  L:" +
  sys.weather(LAT, LON, "daily.temperature_2m_min.0") + "°"`.

**(2) 3-DAY FORECAST** — directly under the current block. A frosted RoundedView
(`#ffffff14`, border_radius 14) with ONE SolidView row per day, EACH ROW a FIXED
`height: 36`: day name `width: Fill` (font 13 — Fill absorbs the slack so the
row fits ANY watch width; never give the day name a fixed dp width), a weather
EMOJI width 28 (☀️ ⛅ ☁️ 🌧️ ⛈️ ❄️), a Filler, then lo° dim (`#ffffff73`)
width 36 and hi° white width 40, all font 13. THREE rows ONLY: Today, tomorrow,
day after. The lo°/hi° of row N
are LIVE: `sys.weather(LAT, LON, "daily.temperature_2m_min.N")` and
`sys.weather(LAT, LON, "daily.temperature_2m_max.N")` for N = 0 (Today) … 2.

### BLOCK: DETAIL-TILES

(3) The detail grid — below the forecast. TWO `flow: Right` rows, each holding
TWO equal frosted tiles (`width: Fill`, `#ffffff14`, border_radius 14). Every
tile stacks an UPPERCASE caption (font 10, `#ffffff73`, `width: Fill` so a long
caption wraps instead of clipping at the tile edge), a big value (font 17),
and a sub-line (font 11, `#ffffff99`). The FOUR tiles in order — every value
LIVE (sys.airquality / sys.weather); only captions and sub-lines are yours:
- AIR QUALITY — value = `sys.airquality(LAT, LON, "current.us_aqi")`; set its
  `draw_text.color` by the US-AQI band — Good ≤50 #32d74b, Moderate 51–100
  #ffd60a, Unhealthy 101–150 #ff9f0a, Very Unhealthy 151–200 #ff453a — and put
  the category word in the sub.
- UV INDEX — `sys.weather(LAT, LON, "daily.uv_index_max.0")`; sub Low/Moderate/
  High/Very High.
- HUMIDITY — `sys.weather(LAT, LON, "current.relative_humidity_2m") + "%"`; sub free.
- WIND — `sys.weather(LAT, LON, "current.wind_speed_10m") + " km/h"`; sub free.

The whole column is a short vertically-scrolling page (~560dp) — the glance
(BLOCK: CURRENT) is complete without scrolling; forecast + tiles are one drag away.

## Data shape it needs

- city
- temp (hero)
- H / L
- 3 × (day name, weather emoji, lo°, hi°)
- aqi + category
- uv (0–11)
- humidity (percent)
- wind (e.g. `8 km/h`)
- lat / lon (real decimal)

---

Widgets used: WeatherIcon, RoundedView, SolidView, Filler, Label — NOT Image,
NOT GradientYView, NOT the sys.* image helpers — see `widgets/`.
