//! localStorage persistence. All values are JSON so the tools stay serializable.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use web_sys::window;

pub const PARAMS_KEY: &str = "pendulum_params";
pub const READINGS_KEY: &str = "pendulum_readings";
pub const HISTORY_KEY: &str = "pendulum_history";

const SWING_WINDOW_MS: f64 = 5_000.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SwingSample {
    pub at_ms: f64,
    pub angle_deg: f32,
}

/// Keep only the last five seconds of swing samples so storage stays bounded.
pub fn record_swing(history: &mut Vec<SwingSample>, at_ms: f64, angle_deg: f32) {
    history.retain(|sample| at_ms - sample.at_ms <= SWING_WINDOW_MS);
    history.push(SwingSample { at_ms, angle_deg });
}

fn storage() -> Option<web_sys::Storage> {
    window()?.local_storage().ok().flatten()
}

pub fn load<T: DeserializeOwned>(key: &str) -> Option<T> {
    let raw = storage()?.get_item(key).ok()??;
    serde_json::from_str(&raw).ok()
}

pub fn save<T: Serialize>(key: &str, value: &T) {
    if let Ok(json) = serde_json::to_string(value) {
        if let Some(storage) = storage() {
            let _ = storage.set_item(key, &json);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swing_history_keeps_only_the_last_five_seconds() {
        let mut history = vec![
            SwingSample {
                at_ms: 0.0,
                angle_deg: 10.0,
            },
            SwingSample {
                at_ms: 4_000.0,
                angle_deg: 12.0,
            },
        ];
        record_swing(&mut history, 6_000.0, 8.0);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].at_ms, 4_000.0);
        assert_eq!(history[1].at_ms, 6_000.0);
    }
}
