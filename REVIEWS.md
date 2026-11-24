# REVIEWS

Decisions made while building from `DESIGN.md`, plus what was verified and what remains open.

## Design decisions / deviations

| Topic | DESIGN.md said | Built | Why |
|---|---|---|---|
| UI rendering | `macroquad` canvas, "Macroquad UI reads from HashMap" | Yew 0.23 DOM + SVG | Mobile-first and keyboard-friendly are hard requirements; native HTML inputs, focus order and touch targets come free. A hand-rolled canvas widget set (editable table, number inputs, draggable ruler/protractor) would be far more code with worse a11y. |
| Physics | `rapier2d = { version = "0.22", features = ["deterministic"] }` | `rapier2d` 0.22 with `dim2`, `f32`, `enhanced-determinism`, `wasm-bindgen` | `"deterministic"` is not a feature of rapier2d 0.22; `enhanced-determinism` is the equivalent. `dim2` is required when default features are off (otherwise rapier2d does not compile). |
| Tooling | `wasm-pack build --target web --out-dir pkg` | `trunk build --release` | `wasm-pack` is not installed in this environment; `trunk` is, and handles wasm-bindgen + asset pipeline. Output goes to `dist/` instead of `pkg/`; the deploy workflow publishes `dist/`. |
| Asset paths | Cache `pkg/*` | SW precaches the stable shell and caches hashed wasm/js/css on first fetch | Trunk emits content-hashed filenames, so hard-coding them in `sw.js` would break on every build. Offline still works after the first load. |
| UI text | No hardcoded strings | All screen text in `ui-text.en.json` / `ui-text.ms.json`; English embedded via `include_str!` for first offline load | Satisfies the externalisation requirement; bob names/materials stay in `BOB_PRESETS` because they are domain data. |
| Swing count | "each crossing of swing threshold line (both directions)" | Count outward crossings on either side (one per half-oscillation) | `period = 2 × time / swings`. Counts in/out on the same side would double-count every half-swing. |
| Number inputs | "synced number input", "clamps on blur" | Commit on `change` (blur/Enter), not on every keystroke | Writing `value` from state on each `input` re-renders and reformats mid-typing, making decimal entry impossible. `change` still fires on blur, so clamping stays correct. |
| Add Reading | "append a new trial row (re-runs experiment with same params)" | Re-runs immediately with the same params; the new reading is appended when that run is confirmed | Adding a blank editable row did not re-run the experiment and left the user stuck in the modal. |

## Confirmed behaviour

- `cargo check --target wasm32-unknown-unknown` — passes.
- `cargo test --target x86_64-unknown-linux-gnu` — 12/12 pass, including deterministic physics, swing counting, clamping, period/g calculation, CSV/JSON export, i18n lookup and trial insertion.
- `trunk build --release` — passes; `dist/` contains the wasm/js/css bundle plus manifest, both icons, SW and both translation files.
- `trunk build --release --public-url /makmal-bandul/` — passes; generated asset URLs are `/makmal-bandul/...` as needed for GitHub Pages subpath hosting.
- Physics smoke test caught and fixed a bad revolute-joint anchor (the bob initially snapped straight down instead of swinging).
- The service worker precaches the hashed JS/WASM/CSS by reading them out of `index.html` at install time; otherwise the very first load would not be available offline until a reload.

## Open / unverified

- No browser smoke test was possible in this environment: touch drag, focus/keyboard operation, clipboard, download and service-worker offline reload are implemented but not visually verified.
- No git remote is configured, so the app has not actually been deployed. `.github/workflows/deploy.yml` is ready and will publish `dist/` on push to `main`/`master`.
- `wasm-opt` is disabled (`data-wasm-opt="0"`) because Trunk's download timed out here; the release wasm is ~1.2 MB. Re-enable it when the tool can be fetched.
- `pendulum_history` grows without bound; no pruning was added.
