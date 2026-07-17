# News app — requirements spec (assemble from widgets; no exemplar)

A polished, dark, **iOS-News-style top-stories app** for Hacker News, with a
list↔detail flow that is entirely Splash-local. Use it for headlines / what's
happening / top news ("top news", "头条", "what's happening").

**YOU generate this card by ASSEMBLING the widget patterns** — there is no
exemplar to copy. Build it from `widgets/interaction.md` (Splash-local state +
named widgets, style templates, ScrollYView tap-target rules),
`widgets/containers.md`, and `widgets/sys-helpers.md` (`sys.news`).
Requirements below are MANDATORY. Keep the whole block under 8,000 bytes.

## State model (full-script body, Splash-local)

- `// name: news-app` is the first line; the FIRST executable line after it is
  `let news_app = { detail: false selected: 0 }` — no comment before it.
- Two `fn` helpers drive ALL navigation by mutating named widgets
  (interaction.md § Splash-local): `fn show_story(i)` (detail = true, masthead
  → `< Top Stories`, page title → `Story`, lead becomes story `i`: kicker
  `SELECTED STORY`, live title/meta, live `url`) and `fn show_list()` (masthead
  → `TOP STORIES`, page title → `Hacker News`, lead back to story 0, kicker
  `1 · TOP STORY`, url cleared). Never `agent.notify`, never native handlers.

## Closure form — MANDATORY

Every `on_click` is an EXPRESSION closure calling exactly ONE fn:
`on_click: || show_story(0)` — NEVER the block form `on_click: ||{ ... }`.
Put ALL branching inside the fn bodies (e.g. `fn header_click() { if
news_app.detail { show_list() } }`, masthead `on_click: || header_click()`).

## Layout — MANDATORY, top to bottom

Root: `SolidView{ width: Fill height: 460 flow: Overlay new_batch: true }`,
TRUE BLACK `#000000` (OLED watch), NO gradient, under a
padded `flow: Down` column (`Inset{left: 12 top: 12 right: 12 bottom: 56}`).
Orange accent `#ff9f0a`, white primary text, muted `#ffffff77-88` metadata,
8px card radius, cards never nest.

1. **Masthead** — `header_btn := Button{ width: Fill height: 44 }`,
   orange 11pt text `TOP STORIES`; `on_click` calls `show_list()` when in
   detail (the big Back target). Under it `page_title := Label` 26pt white.
2. **Lead card** — fixed `RoundedView` height 200 (`#ffffff12`) for story 0 /
   the selected story: `lead_kicker :=` (orange 11), `lead_title :=` (20pt,
   `width: Fill`, live `sys.news(0, "title")`), `lead_meta :=` (12pt, live
   points · comments · by author), `lead_url :=` (10pt `#ffb340`, empty in
   list mode). A full-size transparent Button overlay opens
   `show_story(0)` from list mode (allowed here — the lead is OUTSIDE the
   scroll view).
3. **Section label** — `section_label :=` orange 10pt, `LATEST` (list) /
   `TOP STORIES` (detail).
4. **Dense feed** — `ScrollYView{ width: Fill height: Fill }` holding SEVEN
   story rows for indexes 1..5 (the watch cap), built from TWO style templates you define
   (interaction.md § style templates): a 136dp `StoryRow` RoundedView
   (`#ffffff0d`) and a transparent 72dp trailing `RowTap` Button (plain `Button`, fully transparent draw_bg colors) whose
   `on_click: || show_story(i)` (expression form). Each row: rank number (orange 17),
   wrapping live title (12pt, `width: Fill`), live meta line (9pt), `>`
   chevron (orange 16). The row BODY stays a swipe surface — no full-row
   buttons inside the scroll (gesture capture).
5. Masthead, page title, lead, and section label stay FIXED (outside the
   scroll); only the seven rows scroll. Detail changes ONLY the named widgets
   (masthead text, page title, lead content, section label) — the feed stays
   below so users keep browsing.

## LIVE DATA — MANDATORY

Every title, author, points, comments, url is `sys.news(NUMERIC_INDEX,
"key")`, index 0 = top story; keys exactly `title`, `author`, `points`,
`url`, `comments` (never path-style calls, never `source`/`published`).
Values may show `—` briefly while the feed loads. Never invent story data.

## Failure conditions

A missing `// name: news-app` first line, a missing `let news_app` opening
state line, missing `fn show_story(` or `fn show_list()`, fewer than 7
`StoryRow{` instantiations, `sys.news` bound with fewer than 8 distinct
indexes, a full-row button inside the ScrollYView, any `agent.notify`, any
invented story text, or a block over 8,000 bytes = FAILURE.
