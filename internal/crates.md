### Workspace Structure
```
pendulum-experiment/
├── Cargo.toml (workspace)
├── pendulum-core/          # Backend crate (physics, domain, serialization)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── physics.rs      # Rapier2D pendulum simulation
│       ├── domain.rs       # MeasureTool, Rule, Stopwatch, Protractor, PendulumBob
│       └── serialization.rs # serde for localStorage/config
└── pendulum-ui/            # Frontend crate (WASM, rendering, input)
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── main.rs         # wasm-bindgen entry point
        ├── rendering.rs    # Macroquad canvas, retort stand, pendulum, threshold
        ├── ui.rs           # Setup panel, dialogs, controls, parameter space
        └── pwa.rs          # Service worker, manifest, offline handling
```

### Core Crates (pendulum-core)
- **rapier2d** = { version = "0.22", features = ["deterministic"] } — 2D physics (pendulum simulation)
- **serde** = { version = "1.0", features = ["derive"] } — Serialization for settings/readings
- **serde_json** = "1.0" — JSON for localStorage/config

### UI Crates (pendulum-ui)
- **macroquad** = "0.4" — 2D rendering, input, audio, WASM-first
- **wasm-bindgen** = "0.2" — WASM ↔ JS interop
- **web-sys** = { version = "0.3", features = ["Window", "Document", "LocalStorage"] } — Browser APIs for PWA
- **js-sys** = "0.3" — JS standard library bindings
- **getrandom** = { version = "0.2", features = ["js"] } — WASM-compatible RNG
- **console_error_panic_hook** = "0.1" — Panic messages in browser console
- **wee_alloc** = "0.4" — Small WASM allocator (optional, reduces binary size)
- **pendulum-core** = { path = "../pendulum-core" } — Backend domain & physics

### Communication
- `pendulum-ui` calls `pendulum-core` functions via direct Rust calls (same WASM module)
- Shared types: `PendulumBob`, `Rule`, `Stopwatch`, `Protractor`, `SimulationState`
- No JS interop needed between crates — both compile to single WASM