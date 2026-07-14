# Adding a new app card

This walks through adding a whole new app type end-to-end, using **crypto** as a
worked example (a live coin price card). The existing weather / stock / news apps
are your templates — copy the closest one.

There are two layers to touch:

- **Data** (only if your app needs a *new* live source) → the framework fork.
- **App definition + routing** (always) → `a2app/` + `app/`.

---

## Step 0 — pick a live data source

Use a **keyless, CORS-free JSON** API (the card fetches it directly at render
time, through the device's network/proxy). Examples in use: open-meteo (weather),
Yahoo Finance chart (stock), Hacker News Algolia (news). For crypto:
`https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd`.

Verify it returns clean JSON and works **without** an API key before continuing.

---

## Step 1 — add a `sys.<domain>` data helper (framework fork)

Only needed for a new source. In the framework fork
`widgets/src/splash.rs`, inside `register_agent_module`, add a helper next to
`sys.stock` / `sys.news`. It should:

1. Read its args (`script_value!`, `cast_to_string`).
2. Build the API URL.
3. Fetch + pluck:
   ```rust
   let out = match vm.host.cx_mut().script_data_fetch(&url) {
       Some(bytes) => json_pluck(&bytes, &path).unwrap_or_else(|| "—".to_string()),
       None => "—".to_string(),
   };
   vm.bx.heap.new_string_from_str(&out)
   ```
   `script_data_fetch` (dedup + async), `json_pluck` (dot-path, array indices,
   ISO→HH:MM), and the `DATA_FETCH_EPOCH` re-eval are all reused for free — see
   [ARCHITECTURE.md §4](ARCHITECTURE.md#4-the-splash-card--live-data-binding).

3b. **Add your helper name to `body_binds_live_data()`** (same file) so cards that
   call it arm the frame pump and re-evaluate when the fetch lands:
   ```rust
   fn body_binds_live_data(body: &str) -> bool {
       body.contains("sys.weather") || body.contains("sys.airquality")
           || body.contains("sys.stock") || body.contains("sys.news")
           || body.contains("sys.crypto")           // ← add
   }
   ```

Rebuild is required for helper changes (it's native code) — see
[BUILDING-ANDROID.md](BUILDING-ANDROID.md).

Document the helper's keys in `a2app/widgets/sys-helpers.md`.

---

## Step 2 — write the app definition (`a2app/`, no rebuild)

Create the spec + a full known-good exemplar:

```
a2app/apps/crypto/app.md                        # what the card is; mandatory sections
a2app/apps/crypto/exemplars/crypto-canonical.splash   # a complete working card
```

- **`app.md`** — describe the layout and, crucially, that **every number is a
  `sys.crypto(...)` call, never hardcoded**. Copy the tone of `apps/stock/app.md`.
- **The exemplar is the highest-leverage file** — the LLM mirrors it closely.
  Make it a real, complete card with `sys.crypto(...)` bindings and `// name: …`
  as line 1. Copy `apps/stock/exemplars/stock-canonical.splash` and adapt.
- Reuse the shared conventions: root `flow: Overlay` container, inner column with
  `padding: Inset{... top: 54 ...}` (clears the status bar), frosted
  `RoundedView` tiles, `width: Fill` on any wrapping text.

---

## Step 3 — register the domain (`app/src/main.rs`, rebuild)

Four edits so the AMA knows the domain and an agent exists for it:

1. **`clear_chat`** — create the agent:
   ```rust
   let crypto = agent.create_session(cx, app_cfg());
   self.apps = vec![ …, AppRecord::with_domain(crypto, "Crypto", "crypto") ];
   ```
2. **`AMA_SYSTEM_PROMPT`** — add the domain + a routing hint:
   `crypto = a cryptocurrency's price (BTC, ETH, 比特币)`.
3. **`APP_SPLASH_ROUTER`** — mention `crypto` in the "which app type" list.
4. **`a2app/framework.md`** — add `crypto` to the app-types list agents read.

`route_to_app` / `app_splash_router_for` need **no change** — they already match
`AppRecord.domain` to the AMA's decision generically.

---

## Step 4 — rebuild memory + deploy

1. `python3 scripts/build_memory.py --check` — confirm the token estimate is under
   `_main.json`'s `max_inject_tokens` (raise the cap if not — see the ⚠️ in
   ARCHITECTURE §3). Add your app's two lines to `FILES` in `build_memory.py`.
2. `python3 scripts/build_memory.py` — writes `MEMORY.md`.
3. Deploy `MEMORY.md` + the `a2app/` tree to the device and rebuild the APK — see
   [BUILDING-ANDROID.md](BUILDING-ANDROID.md#deploy-the-app-card-memory).
4. Test: send a crypto intent; confirm the log shows
   `AMA → activate 'crypto' app agent`, the card renders, and
   `grep -c 'sys.crypto' <saved card>` shows the values are DSL-bound (not
   hardcoded), matching the live API.

---

## Checklist

- [ ] Keyless JSON source verified
- [ ] `sys.<domain>` helper + `body_binds_live_data` (framework fork) — if new source
- [ ] Keys documented in `a2app/widgets/sys-helpers.md`
- [ ] `apps/<domain>/app.md` + `exemplars/<domain>-canonical.splash`
- [ ] `clear_chat` agent, `AMA_SYSTEM_PROMPT`, `APP_SPLASH_ROUTER`, `framework.md`
- [ ] `build_memory.py` FILES updated; `--check` fits the token cap
- [ ] Rebuilt, deployed, verified live on device (values match the API)
