### Best Practices
1. **WASM Optimization**
   - `cargo build --release --target wasm32-unknown-unknown`
   - Use `wasm-opt -Oz` for size reduction
   - Enable `lto = true`, `opt-level = "z"` in `Cargo.toml` `[profile.release]`

2. **PWA Requirements**
   - Register Service Worker in `index.html` for offline support
   - Cache `pkg/` assets and `index.html` via `Cache API`
   - Add `manifest.json` with `display: "standalone"` for installability
   - Use `localStorage` for persisting user experiment settings

3. **Physics Determinism**
   - Enable `rapier2d` `deterministic` feature for reproducible simulations
   - Fixed timestep (e.g., 1/60s) for pendulum integration
   - Store state as `f32` for WASM compatibility

4. **Architecture**
   - **Two-crate workspace**: `pendulum-core` (physics, domain, serialization) + `pendulum-ui` (WASM, rendering, input)
   - `pendulum-core`: Rapier2D physics, `MeasureTool` trait, `Rule`/`Stopwatch`/`Protractor`/`PendulumBob`, serde
   - `pendulum-ui`: Macroquad rendering, setup panel/dialogs/controls, PWA handling, wasm-bindgen entry
   - Shared types via `pendulum-core` dependency; no JS interop between crates
   - Separate physics from rendering; fixed timestep (1/60s); state machine for workflow

5. **UI Text Externalization**
   - All user-facing strings in a single `ui-text.json` (loaded at startup via `fetch`)
   - Schema: `{ "setup": { "bobTitle": "Select Pendulum Bob", "lengthLabel": "Length (m)", ... }, "running": { "stopBtn": "Stop", "swingCounter": "Swings: {count}" }, ... }`
   - Supports i18n: `ui-text.en.json`, `ui-text.ms.json`, etc.; fallback to `en`
   - Macroquad UI reads from in-memory `HashMap<String, String>`; hot-reload in dev via `localStorage` override
   - No hardcoded strings in Rust/WASM or JS glue code

6. **GitHub Pages Deployment**
   - `wasm-pack build --target web --out-dir pkg`
   - Deploy `pkg/`, `index.html`, `manifest.json`, `sw.js` to `gh-pages` branch
   - Configure `base href="/<repo-name>/"` in `index.html` for subpath hosting