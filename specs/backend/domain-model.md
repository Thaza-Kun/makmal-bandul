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
    pub name: &'static str,  // e.g., "100g", "30g", "20g"
    pub mass: f32,           // kg
    pub radius: f32,         // m (for moment of inertia)
}

pub const BOB_PRESETS: [PendulumBob; 3] = [
    PendulumBob { name: "100g", mass: 0.1, radius: 0.0125 },
    PendulumBob { name: "30g", mass: 0.03, radius: 0.015 },
    PendulumBob { name: "20g", mass: 0.02, radius: 0.015 },
];
```