# Stock app

The stock app renders **two** kinds of dark, iOS-Stocks-style FULL-SCREEN card.
Pick the one that matches the request:

1. **MOVERS LIST** — a top-10 "day gainers" list. Use when the request is about the
   market's best/top/most-performant stocks, movers, gainers, a watchlist, or "top
   N stocks" (e.g. "top 10 stocks", "best performing stocks", "today's movers",
   "涨幅榜"). Reproduce `exemplars/stock-movers.splash`.
2. **DETAIL QUOTE** — the single-ticker quote page. Use when the request names ONE
   specific ticker or company (e.g. "AAPL", "Tesla stock", "英伟达股价"), OR when a
   movers-list row is tapped (the app hands you the tapped ticker). Reproduce
   `exemplars/stock-canonical.splash`.

## LIVE DATA — MANDATORY (never hardcode a number)

- MOVERS LIST: every field is `sys.movers(i, "field")` — `i` = 0..9, 0 = the biggest
  % gainer today. Fields: `symbol`, `name`, `price`, `change` (signed), `changepct`
  (signed %), `high`, `low`, `marketcap`, `vol`, `52wh`, `52wl`, `currency`,
  `exchange`. ONE fetch serves all rows.
- DETAIL QUOTE: every number is `sys.stock("<TICKER>", "key")` and the chart is
  `sys.stockbar("<TICKER>", i, count, maxh)`. Keys: `symbol`, `name`, `exchange`,
  `currency`, `price`, `prev`, `high`, `low`, `52wh`, `52wl`, `vol`, `change`,
  `changepct`. **Do NOT use `open`** (Yahoo often omits it → `$—`).

Values show `—` briefly, then auto-refresh. You only choose labels and colors.

## MOVERS LIST structure (top to bottom) — `exemplars/stock-movers.splash`

Root = a `flow: Overlay` SolidView `height: 858` + `GradientYView` fill, then a
`flow: Down` column, padding `Inset{left: 22 top: 54 right: 22 bottom: 96}`:

1. **Masthead** — a small `"TODAY · TOP GAINERS"` kicker (font 11, accent #30d158)
   over `"Movers"` (font 30). A thin divider under it.
2. **10 TAPPABLE rows**, one per `i = 0..9`, each a **fixed-height** (60)
   `flow: Overlay` holding TWO children:
   - the **visual** row (`flow: Right align: Align{y: 0.5}`): a rank `Label{width:26}`
     (dim), a `flow: Down width: Fill` column with `sys.movers(i,"symbol")` (font 18)
     over `sys.movers(i,"name")` (font 12, dim), then a right `flow: Down` column with
     `"$"+sys.movers(i,"price")` (font 17) over a small green pill (RoundedView,
     `#30d15826`) showing `sys.movers(i,"changepct")`.
   - a **transparent tap target** LAST (so it sits on top): `Button{ width: Fill
     height: Fill draw_bg.color: #00000000 text: "" on_click: || agent.notify("open",
     {ticker: sys.movers(i, "symbol")}) }`. This is what makes the whole row tappable
     and opens that ticker's detail card — **do NOT omit it, and keep the `ticker`
     payload keyed to the SAME `i`**. A hairline `SolidView` divider sits between rows.

Gainers are all positive → the pill is green. Fixed row height (not `Fit`) is
required so the `Fill` tap-Button resolves inside the `Overlay`.

## DETAIL QUOTE structure (top to bottom) — `exemplars/stock-canonical.splash`

Root = `flow: Overlay` SolidView `height: 858` + `GradientYView`, then `flow: Down`,
padding `Inset{left: 22 top: 50 right: 22 bottom: 118}`:

1. **Header** — `symbol` (font 32); below it `name + "  ·  " + exchange + "  ·  " +
   currency` (font 13, dim).
2. **Hero price** — `"$" + price` (font 42); **change row** `change + "  (" +
   changepct + ")"` (accent green up / red down) + a dim `"Today"`.
3. **INTRADAY CHART** — `View{ height: 116 flow: Right }`: a LEFT y-axis
   (`width: 58` `flow: Down`: `"$"+high` top, `"$"+low` bottom) + a plot
   (`flow: Overlay`) with three faint full-width gridlines BEHIND and, in FRONT, the
   area = `flow: Right align: Align{y: 1.0}` of ~68 `SolidView{ height:
   sys.stockbar("<TICKER>", i, 68, 108) draw_bg.color: #34c759d9 }` (translucent so
   gridlines show through). A time axis `09:30 … 12:45 … 16:00` and a range-selector
   row of pill chips `1D 1W 1M 6M 1Y` (first active, spread with `Filler`s).
4. **STAT GRID** — one frosted inset RoundedView (`#ffffff0a`, radius 22) with THREE
   `height: Fill` rows of two `flow: Down` cells (caption font 11 dim over value font
   20) split by hairline dividers: PREV CLOSE, VOLUME, DAY HIGH, DAY LOW, 52W HIGH,
   52W LOW. All money values already come 2-decimal from `sys.stock`.

Muted gradient (`#0a0e14 → #0e1826`). Widgets: GradientYView, SolidView, RoundedView,
View, Label, Filler, Button.
