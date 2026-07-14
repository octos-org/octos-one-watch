# News app

A clean, dark, **iOS-News-style top-stories card** (Hacker News front page). Use it
for headlines / what's happening / top news ("top news", "头条", "what's
happening"). Reproduce `exemplars/news-canonical.splash` closely — it is DENSE (a
lead story + a tight list), NOT a few big cards.

## LIVE DATA — MANDATORY (never invent a headline)

Every headline MUST come from `sys.news(index, "key")` — index 0 = top story, up to
~11. NEVER write a headline yourself. Values show `—` briefly, then auto-refresh.

Keys: `title`, `author`, `points`, `url`, `comments`.

## Structure (iOS News), top to bottom

Root = a `flow: Overlay` SolidView `height: 1500` (tall, scrolls) with a
`GradientYView` fill, then a `flow: Down height: Fill` column with padding
`Inset{left: 18 top: 54 right: 18 bottom: 96}` (top 54 clears the status bar;
**bottom 96 keeps the last row clear of the "+" FAB**) and `spacing: 9`:

1. **Compact masthead** — a small `"TOP STORIES"` kicker (font 11, accent #ff9f0a)
   over `"Hacker News"` (font 26). Keep it SMALL — do NOT let the header dominate.
2. **LEAD story** (index 0) — its own RoundedView (`draw_bg.color #ffffff12`,
   radius 16, padding ~16/14): a `"1  ·  TOP"` kicker (accent, font 11), the TITLE
   `width: Fill text: sys.news(0,"title")` (font 20 — `width: Fill` wraps it), then
   `points + " points  ·  " + author` (font 12, #ffffff88).
3. **DENSE list, stories 2..8** — SEVEN compact rows, each a `flow: Right`
   RoundedView (`draw_bg.color #ffffff0d`, radius 13, TIGHT padding ~14/10,
   spacing 12): a slim rank Label (`width: 20`, the number, accent, font 17) then a
   `flow: Down width: Fill` column with the TITLE (`width: Fill
   text: sys.news(N,"title")`, font 15 — wraps) over a meta line
   (`sys.news(N,"points") + " points  ·  " + sys.news(N,"author")`, font 11,
   #ffffff77). Row N uses index N. Keep rows tight so ~6 stories are on-screen.

Widgets: GradientYView, SolidView, RoundedView, View, Label — see `widgets/`.
