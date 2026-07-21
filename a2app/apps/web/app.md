# App: web — watch-sized HTML app cards

Use this domain for an actionable single app, tool, timer, calculator, game, or
utility that is not owned by weather, stock, news, activity, or youtube.

## Output contract

- Emit exactly one `runhtml` fenced block and no surrounding prose.
- The first line is `<!-- name: <stable-kebab-name> -->`.
- Emit one complete HTML document. Keep all CSS and JavaScript inline; do not
  load external frameworks, fonts, stylesheets, or scripts.
- Include UTF-8 and viewport meta tags.

## Watch constraints

- Target a 372×430 touch screen and Chromium 83. Use broadly supported CSS and
  ES2019 JavaScript; no optional chaining, nullish coalescing, modules, `dvh`,
  `:has()`, container queries, or top-level await.
- Fill the viewport, avoid horizontal scrolling, and reserve at least 62px at
  the bottom for the collapsed native composer.
- Use one column, concise text, 12px minimum body type, and touch targets at
  least 44px. Prefer one primary interaction over phone-style navigation.
- Use a dark OLED-friendly background. Always show a visible loading or error
  state rather than a blank region.

## Data, state, and media

- Store card-local state in `localStorage`, with keys namespaced by card name.
- Keyless JSON APIs may be fetched, but failures must degrade visibly.
- Video, music, and live-stream requests belong to the `youtube` domain, not
  this generic web domain.
