## User Workflow

> The state machine below still applies, but UI details are superseded by `wireframe.md`: setup is a Variables modal instead of a guided wizard and collapsible panel.

### State Machine: `Setup → Running → Paused → Confirming → Results`

#### Persistent Setup Panel
The experiment setup panel is **collapsible and inline** with the experiment state (positioned above the visualization on mobile, sidebar on desktop). **Parameter space is collapsed by default** — expandable accordion showing all parameters editable:
  | Parameter | Value |
  |-----------|-------|
  | Bob | [selected] — dropdown to change |
  | Length (m) | `Rule.read()` rounded to `Rule.precision()` — number input with ± |
  | Release Angle (°) | `Protractor.read()` (release) rounded to `Protractor.precision()` — number input with ± |
When collapsed, only a summary bar with key values (bob, length, angle) remains visible; tap/click to expand.

**Menu Buttons** (in setup panel header):
- **Change Bob** → re-opens Dialog 1 (Bob Selection)
- **Change Length** → re-opens Dialog 2 (Length Input)

#### 1. Setup Phase (Modal Sequence on First Load)
On initial load (or `New Experiment`), a **modal dialog sequence** guides the user. Each dialog must be completed before the next appears. Values are written to the persistent panel immediately.

- **Dialog 1 — Bob Selection**: 3-item card grid (icon, **name by mass** e.g., "100g", "30g", "20g"). Single select → binds to `PendulumBob` preset. `Close` → next dialog.
- **Dialog 2 — Length**: **Partial ruler on top of number input with ± buttons**. A horizontal ruler segment (showing ~10cm range around current value, 1mm divisions) slides above a centered number display; **value marked on ruler**, ruler slides as value changes. **Plus/Minus buttons** on sides increment/decrement by 1mm. Range 0.1–1.0m. Binds to `Rule` tool. `Close` → next dialog.
- **Dialog 3 — Release Angle**: Full setup view with **pendulum highlighted** (pulsing glow on bob/rod). **Pendulum bob is draggable** — drag bob to set release angle; angle value updates live; synced number input with ±. Binds to `Protractor` (release angle). `Close` → returns to setup panel with **Start button enabled**.

#### Start Button
- Prominent **Start** button in setup panel (or floating action button) — user must press to begin experiment
- **Visual cue**: Green accent color, play icon, pulse animation when ready
- Transitions to `Running` only on explicit press

#### 2. Running Phase
- **Physics Simulation**: Rapier2D pendulum with selected bob, length, initial angle — **realistic air damping** applied (linear + quadratic drag); oscillation amplitude decays naturally with decreasing peak angle over time; damping coefficient tuned per bob mass/radius
- **Visual**: **Retort stand** (visual only — front-facing, faded; base, rod, clamp) from which pendulum hangs; pendulum bob + rod + **swing threshold line at 0° (vertical, dashed)** — fixed, not adjustable
- **Timer**: Digital display (MM:SS.cc) **positioned on top of pendulum pivot**; updates at `Stopwatch::precision` (0.01s)
- **Swing Counter**: Prominent visual counter (large numeric display) — **positioned on top of pendulum pivot**; increments each crossing of the 0° threshold line (one direction only, counted on return swing)
- **Displayed Values**: All numeric readouts (length, angle, time, swing count) **rounded to their tool's `precision()`** — never show sub-precision digits
- **Controls** (bottom-anchored menu):
  - **Wide Start/Stop button** (full-width, toggles between green Start / red Stop with pulse)
  - **Two buttons above Start/Stop**:
    - **Confirm Readings** → captures current timer + swing count, adds row to readings table
    - **Reset Experiment** → returns to `Setup` (preserves params, re-opens Dialog 1)

#### 3. Paused Phase
- Physics frozen, timer stopped
- **Controls** (bottom-anchored menu):
  - **Wide Resume/Stop button** (toggles between green Resume / red Stop with pulse)
  - **Two buttons above Resume/Stop**:
    - **Confirm Readings** → transitions to `Confirming`
    - **Reset Experiment** → returns to `Setup` (re-opens Dialog 1)

#### 4. Confirming Phase (Modal/Overlay)
- **Readings Table** (pre-filled from tools, values rounded to tool precision):
  | Parameter | Value |
  |-----------|-------|
  | Bob | [selected] |
  | Length (m) | `Rule.read()` rounded to `Rule.precision()` |
  | Release Angle (°) | `Protractor.read()` (release) rounded to `Protractor.precision()` |
  | Time (s) | `Stopwatch.read()` rounded to `Stopwatch.precision()` |
  | Swing Count | [computed] |
- **Validation**: Each value clamps to its tool's `min_value()`/`max_value()` on blur
- **Add Reading**: Button to append a new trial row (re-runs experiment with same params, adds new Time/Swing Count row)
- **Actions**:
  - `Save & Continue` → `Results` (persists all trials to `localStorage`)
  - `Back` → `Paused`

#### 5. Results Phase
- **Summary Card**: Final computed values (period, g estimate, % error vs 9.81) — all rounded to appropriate precision
- **Export**: Copy JSON / Download CSV
- **New Experiment** button → `Setup` (clears stopwatch, keeps bob presets, re-opens Dialog 1)

### PWA/Offline Requirements
- Service Worker caches: `index.html`, `manifest.json`, `pkg/*`, `sw.js`
- `localStorage` keys: `pendulum_params`, `pendulum_readings`, `pendulum_history[]` (bounded to last 5 seconds of swing data)
- Works fully offline after first load