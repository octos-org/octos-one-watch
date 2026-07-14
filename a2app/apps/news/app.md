# News app

A clean, dark, full-screen "top stories" list (Hacker News front page). Use it when
the request is for headlines / what's happening / top news (e.g. "top news", "头条",
"what's happening"). Reproduce the structure of `exemplars/news-canonical.splash`.

## LIVE DATA — MANDATORY (never invent a headline)

Every headline MUST come from `sys.news(index, "key")` (see `widgets/sys-helpers.md`)
— index 0 = the top story, 1 = next, up to ~11. You do NOT know the news, so NEVER
write a headline yourself. A value shows "—" for a moment while it loads, then the
card auto-refreshes. The ONLY things you choose are the ranks, labels and colors.

Keys: `title`, `author`, `points`, `url`, `comments`.

## Structure, top to bottom

Root = a `flow: Overlay` SolidView (dark `draw_bg.color`) with a `GradientYView`
filling it, then a `flow: Down` inner column carrying ALL padding
`Inset{left: 22 top: 54 right: 22 bottom: 24}` (`top: 54` clears the status bar):

1. **Header** — a title Label "Hacker News" (font 30, an accent like #ff9f0a) and a
   "Top Stories" sub-line (font 15, #ffffff99).
2. **STORY ROWS** — give SIX rows for index 0..5, EACH a `flow: Right` RoundedView
   (`draw_bg.color #ffffff0f`, radius 14, `width: Fill`, comfortable padding,
   spacing 12) holding:
   - a fixed-width rank Label (`width: 26`, the number "1".."6", accent color,
     font 20),
   - then a `flow: Down width: Fill` column with the TITLE Label
     (`width: Fill text: sys.news(N,"title")`, font 16 — `width: Fill` makes long
     titles WRAP to multiple lines) over a meta Label
     (`sys.news(N,"points") + " points · " + sys.news(N,"author")`, font 12,
     #ffffff77).
   Row N uses index N in BOTH sys.news calls (title + meta). The whole column is a
   tall scrolling page — comfortable spacing, do not cram.

Widgets: GradientYView, SolidView, RoundedView, View, Label — see `widgets/`.
