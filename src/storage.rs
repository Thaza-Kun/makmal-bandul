//! localStorage persistence. All values are JSON so the tools stay serializable.

use serde::{de::DeserializeOwned, Serialize};
use web_sys::window;

pub const PARAMS_KEY: &str = "pendulum_params";
pub const READINGS_KEY: &str = "pendulum_readings";
pub const HISTORY_KEY: &str = "pendulum_history";

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
