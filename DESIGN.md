## Deployment
- Must be deployed to github pages
- Must be installable as a Progressive Web App
- Must be working offline
- Must prioritize mobile experience
- Is keyboard friendly also

## User Workflow

### State Machine: `Setup → Running → Paused → Confirming → Results`

#### 1. Setup Phase
- **Bob Selection**: 3-item card grid — each card shows bob icon, name, material, mass; single select → binds to `PendulumBob` preset
- **Length Input**: Vertical ruler string UI — draggable marker on calibrated ruler image (0.1–1.0m, 1mm divisions); synced number input — binds to `Rule` tool
- **Release Angle**: `Protractor` tool — protractor UI (arc with degree markings 5°–90°); draggable needle + number input, precision 1°
- **Swing Threshold**: `Protractor` tool — same protractor UI (1°–89°); sets angle threshold for swing detection
- **Persist**: All params saved to `localStorage` on change
- **Start Button**: Enabled when all fields valid → transitions to `Running`

#### 2. Running Phase
- **Physics Simulation**: Rapier2D pendulum with selected bob, length, initial angle
- **Visual**: Pendulum bob + rod + swing threshold line (dashed at threshold angle)
- **Swing Counter**: Prominent visual counter (large numeric display) — increments each crossing of swing threshold line (both directions)
- **Stopwatch UI**: Digital display (MM:SS.cc), updates at `Stopwatch::precision` (0.01s)
- **Controls**: 
  - `Stop` button → transitions to `Paused`
  - `Reset` button → returns to `Setup` (preserves params)

#### 3. Paused Phase
- Physics frozen, stopwatch stopped
- **Controls**:
  - `Resume` → `Running`
  - `Confirm Readings` → `Confirming`
  - `Reset` → `Setup`

#### 4. Confirming Phase (Modal/Overlay)
- **Readings Table** (editable, pre-filled from tools):
  | Parameter | Value |
  |-----------|-------|
  | Bob | [selected] |
  | Length (m) | `Rule.read()` |
  | Release Angle (°) | `Protractor.read()` |
  | Time (s) | `Stopwatch.read()` |
  | Swing Count | [computed] |
- **Validation**: Each value clamps to its tool's `min_value()`/`max_value()` on blur
- **Add Reading**: Button to append a new trial row (re-runs experiment with same params, adds new Time/Swing Count row)
- **Actions**:
  - `Save & Continue` → `Results` (persists all trials to `localStorage`)
  - `Back` → `Paused`

#### 5. Results Phase
- **Summary Card**: Final computed values (period, g estimate, % error vs 9.81)
- **Export**: Copy JSON / Download CSV
- **New Experiment** button → `Setup` (clears stopwatch, keeps bob presets)

### PWA/Offline Requirements
- Service Worker caches: `index.html`, `manifest.json`, `pkg/*`, `sw.js`
- `localStorage` keys: `pendulum_params`, `pendulum_readings`, `pendulum_history[]`
- Works fully offline after first load

## Developer Design

### Domain Model: Measurement Tools

```rust
/// Trait for real-world measurement instruments
pub trait MeasureTool {
    type Reading;
    fn precision(&self) -> f32;      // Smallest division (e.g., 0.001m for rule, 0.01s for stopwatch)
    fn min_value(&self) -> f32;      // Minimum measurable value
    fn max_value(&self) -> f32;      // Maximum measurable value
    fn read(&self) -> Self::Reading; // Current reading
    fn clamp(&self, value: f32) -> f32 { value.clamp(self.min_value(), self.max_value()) }
}

/// Physical ruler for length measurements
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Rule {
    pub precision: f32,  // e.g., 0.001 (1mm)
    pub min_length: f32, // e.g., 0.0
    pub max_length: f32, // e.g., 1.0 (1 meter rule)
    pub current_length: f32,
}

impl MeasureTool for Rule {
    type Reading = f32;
    fn precision(&self) -> f32 { self.precision }
    fn min_value(&self) -> f32 { self.min_length }
    fn max_value(&self) -> f32 { self.max_length }
    fn read(&self) -> f32 { self.current_length }
}

/// Stopwatch for time measurements
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Stopwatch {
    pub precision: f32,   // e.g., 0.01 (10ms)
    pub min_time: f32,    // e.g., 0.0
    pub max_time: f32,    // e.g., 3600.0 (1 hour)
    pub elapsed: f32,
    pub running: bool,
}

impl MeasureTool for Stopwatch {
    type Reading = f32;
    fn precision(&self) -> f32 { self.precision }
    fn min_value(&self) -> f32 { self.min_time }
    fn max_value(&self) -> f32 { self.max_time }
    fn read(&self) -> f32 { self.elapsed }
}

/// Protractor for angle measurements
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Protractor {
    pub precision: f32,   // e.g., 1.0 (1 degree)
    pub min_angle: f32,   // e.g., 0.0
    pub max_angle: f32,   // e.g., 180.0
    pub current_angle: f32,
}

impl MeasureTool for Protractor {
    type Reading = f32;
    fn precision(&self) -> f32 { self.precision }
    fn min_value(&self) -> f32 { self.min_angle }
    fn max_value(&self) -> f32 { self.max_angle }
    fn read(&self) -> f32 { self.current_angle }
}

// Preset bobs with physical properties
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct PendulumBob {
    pub name: &'static str,
    pub mass: f32,      // kg
    pub radius: f32,    // m (for moment of inertia)
    pub material: &'static str,
}

pub const BOB_PRESETS: [PendulumBob; 3] = [
    PendulumBob { name: "Steel", mass: 0.1, radius: 0.0125, material: "Steel" },
    PendulumBob { name: "Wood", mass: 0.03, radius: 0.015, material: "Wood" },
    PendulumBob { name: "Plastic", mass: 0.02, radius: 0.015, material: "Plastic" },
];
```

### Core Crates
- **macroquad** = "0.4" — 2D rendering, input, audio, WASM-first
- **rapier2d** = { version = "0.22", features = ["deterministic"] } — 2D physics (pendulum simulation)
- **wasm-bindgen** = "0.2" — WASM ↔ JS interop
- **web-sys** = { version = "0.3", features = ["Window", "Document", "LocalStorage"] } — Browser APIs for PWA
- **js-sys** = "0.3" — JS standard library bindings
- **serde** = { version = "1.0", features = ["derive"] } — Serialization for settings/readings
- **serde_json** = "1.0" — JSON for localStorage/config
- **getrandom** = { version = "0.2", features = ["js"] } — WASM-compatible RNG
- **console_error_panic_hook** = "0.1" — Panic messages in browser console
- **wee_alloc** = "0.4" — Small WASM allocator (optional, reduces binary size)

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
   - Separate physics (Rapier) from rendering (Macroquad)
   - Use `localStorage` for: bob presets, length, angle, swing line, timer state
   - State machine for workflow: `Setup → Running → Paused → Confirming → Results`

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
