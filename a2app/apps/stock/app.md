# Stock app

A polished, dark, **iOS-Stocks-style FULL-SCREEN quote card** for ONE ticker. Use
it when the request is a stock ticker or a company's share price (e.g. "AAPL",
"Tesla stock", "英伟达股价"). Reproduce `exemplars/stock-canonical.splash` closely —
header, hero price, an **intraday chart with gridlines + a range selector**, then a
dense stat grid.

## LIVE DATA — MANDATORY (never hardcode a price)

Every number MUST come from `sys.stock("<TICKER>", "key")` and the chart from
`sys.stockbar("<TICKER>", i, count)` (see `widgets/sys-helpers.md`) — NEVER invent
one. Pass the real UPPERCASE ticker: Apple `AAPL`, Tesla `TSLA`, Nvidia `NVDA`,
Microsoft `MSFT`, Amazon `AMZN`, Google `GOOGL`, Meta `META`. Values show `—` (or a
flat chart) briefly, then auto-refresh. You only choose labels and colors.

Keys: `symbol`, `name`, `exchange`, `currency`, `price`, `prev`, `high`, `low`,
`52wh`, `52wl`, `vol`, `change` (signed), `changepct` (signed %). **Do NOT use
`open`** — Yahoo omits it and it renders as a `$—` that looks broken.

## Structure (iOS Stocks), top to bottom

Root = a `flow: Overlay` SolidView with a **fixed `height: 858` so it FILLS the
screen — never leave an empty void** (do NOT use `height: Fit`), a `GradientYView`
fill (muted `#0a0e14 → #0e1826`), then a `flow: Down height: Fill` column with
padding `Inset{left: 24 top: 54 right: 24 bottom: 126}`:

1. **Header** — `symbol` (font 34); below it `name + "  ·  " + exchange + "  ·  " +
   currency` (font 13, dim #ffffff8c). Folding currency in here avoids an orphan
   "USD".
2. **Hero price** — `"$" + price` (font 48, `margin: Inset{top: 10}`).
3. **Change row** (`flow: Right`, font 17): `change + "  (" + changepct + ")"` in an
   accent color (green #30d158 up / red #ff453a down), then a dim `"Today"`.
4. **INTRADAY CHART** — a `View{ height: 150 flow: Overlay }` with, BEHIND, three
   faint full-width gridlines (`SolidView height: 1`, `#ffffff17`/`#ffffff0d`/
   `#ffffff17`, separated by `Filler`s); and, IN FRONT, a `flow: Right` holding the
   bars (`View{ width: Fill height: Fill flow: Right align: Align{y: 1.0} spacing: 2
   }` of ~40 `SolidView{ height: sys.stockbar("<TICKER>", i, 40) }`, green up / red
   down) and a narrow `flow: Down` price scale labelling `"H $"+high` (top) and
   `"L $"+low` (bottom). The gridlines + H/L labels give the chart real axes.
5. **Time axis** — a `flow: Right` row `"09:30" … "12:45" … "16:00"` (font 10, dim,
   `Filler`s between).
6. **RANGE SELECTOR** — a `flow: Right` row of pill chips `1D 1W 1M 6M 1Y`, the
   FIRST active (bg `#30d15826`, text #30d158), the rest transparent + dim text.
   Put a `Filler{}` BETWEEN each chip so the row spreads edge-to-edge and the last
   chip is never truncated.
7. **STAT GRID** — a `height: Fill flow: Down` block: a thin divider then THREE rows,
   **each `height: Fill`** so they split the remaining space exactly (no dead zone,
   no overflow). Each row is `flow: Right align: Align{y: 0.5} spacing: 20` with TWO
   cells; each cell is `flow: Down` stacking an UPPERCASE caption (font 11, #ffffff8c)
   over a value (font 20). Stats: **PREV CLOSE**=`"$"+prev`, **VOLUME**=`vol`,
   **DAY HIGH**=`"$"+high` (green), **DAY LOW**=`"$"+low` (red), **52W HIGH**=`"$"+52wh`,
   **52W LOW**=`"$"+52wl`. Keep ONE value per cell — never pack `low+" – "+high`
   (it overflows).

Widgets: GradientYView, SolidView, RoundedView, View, Label, Filler.
