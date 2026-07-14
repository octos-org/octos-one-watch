# Stock app

A clean, dark, iOS-Stocks-style full-screen quote card for ONE ticker. Use it when
the request is a stock ticker or a company's share price (e.g. "AAPL", "Tesla
stock", "英伟达股价"). Reproduce the structure of `exemplars/stock-canonical.splash`.

## LIVE DATA — MANDATORY (never hardcode a price)

Every number MUST come from `sys.stock("<TICKER>", "key")` (see
`widgets/sys-helpers.md`) — you do NOT know the price, so NEVER invent one. Pass the
real ticker symbol (uppercase): Apple `AAPL`, Tesla `TSLA`, Nvidia `NVDA`, Microsoft
`MSFT`, Amazon `AMZN`, Google `GOOGL`, Meta `META`. A value shows "—" for a moment
while it loads, then the card auto-refreshes. The ONLY things you choose are labels
and colors.

Keys: `symbol`, `name`, `price`, `change` (signed, e.g. "+1.99"), `changepct`
(signed %, e.g. "+0.63%"), `prev`, `open`, `high`, `low`, `currency`.

## Structure, top to bottom

Root = a `flow: Overlay` SolidView (dark `draw_bg.color`, e.g. #0b0f17) with a
`GradientYView` filling it (subtle dark-blue gradient), then a `flow: Down` inner
column carrying ALL padding `Inset{left: 24 top: 54 right: 24 bottom: 26}` (the
`top: 54` clears the status bar — NEVER a small top, the ticker gets jammed under
the clock):

1. **Header** — `sys.stock(SYM,"symbol")` (font 34) with `sys.stock(SYM,"name")`
   below it (font 15, #ffffff99).
2. **Hero price** — `"$" + sys.stock(SYM,"price")` alone on its line (font 56,
   `margin: Inset{top: 10 bottom: 0}`).
3. **Change row** — a `flow: Right` row: `sys.stock(SYM,"change")` then
   `"(" + sys.stock(SYM,"changepct") + ")"` (accent color — use #5ac8fa, a neutral
   accent, since up/down can't be colored at render time; the +/- sign shows
   direction) then `sys.stock(SYM,"currency")` (dim, font 14).
4. **STAT GRID** — a `flow: Down` of TWO `flow: Right` rows, each two frosted tiles
   (RoundedView, `draw_bg.color #ffffff12`, radius 16, `width: Fill`), each stacking
   an UPPERCASE caption (font 11, #ffffff99) over a value (font 20): PREV CLOSE =
   `prev`, OPEN = `open`, DAY HIGH = `high`, DAY LOW = `low`.

Widgets: GradientYView, SolidView, RoundedView, View, Label, Filler — see `widgets/`.
