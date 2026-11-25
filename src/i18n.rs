//! UI strings live in JSON files, never in Rust or JS glue.

use std::collections::HashMap;

use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::window;

const EN: &str = include_str!("../static/ui-text.en.json");

#[derive(Clone, PartialEq)]
pub struct I18n {
    pub lang: String,
    strings: HashMap<String, String>,
}

impl I18n {
    pub fn from_json(lang: &str, json: &str) -> Self {
        let mut strings = HashMap::new();
        if let Ok(value) = serde_json::from_str::<Value>(json) {
            flatten(&value, "", &mut strings);
        }
        Self {
            lang: lang.to_string(),
            strings,
        }
    }

    /// Bundled English is always available, including on the very first offline load.
    pub fn embedded() -> Self {
        Self::from_json("en", EN)
    }

    pub fn t(&self, key: &str) -> String {
        self.strings.get(key).cloned().unwrap_or_else(|| {
            let fallback = I18n::embedded();
            fallback
                .strings
                .get(key)
                .cloned()
                .unwrap_or_else(|| key.to_string())
        })
    }

    /// Translate and substitute `{placeholders}` such as `{count}`.
    pub fn tf(&self, key: &str, vars: &[(&str, String)]) -> String {
        let mut text = self.t(key);
        for (name, value) in vars {
            text = text.replace(&format!("{{{name}}}"), value);
        }
        text
    }
}

fn flatten(value: &Value, prefix: &str, out: &mut HashMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, child) in map {
                let path = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten(child, &path, out);
            }
        }
        Value::String(text) => {
            out.insert(prefix.to_string(), text.clone());
        }
        _ => {}
    }
}

/// Fetch the requested language; fall back to bundled English.
pub async fn load(lang: &str) -> I18n {
    if lang == "en" {
        return I18n::embedded();
    }
    let Some(window) = window() else {
        return I18n::embedded();
    };
    let url = format!("ui-text.{lang}.json");
    let response = match JsFuture::from(window.fetch_with_str(&url)).await {
        Ok(value) => value.dyn_into::<web_sys::Response>(),
        Err(_) => Err(wasm_bindgen::JsValue::NULL),
    };
    let Ok(response) = response else {
        return I18n::embedded();
    };
    if !response.ok() {
        return I18n::embedded();
    }
    let text = JsFuture::from(match response.text() {
        Ok(promise) => promise,
        Err(_) => return I18n::embedded(),
    })
    .await
    .ok()
    .and_then(|v| v.as_string());
    match text {
        Some(text) => I18n::from_json(lang, &text),
        None => I18n::embedded(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_bundle_has_the_expected_keys() {
        let i18n = I18n::embedded();
        assert_eq!(i18n.t("setup.bobTitle"), "Select pendulum bob");
        assert_eq!(i18n.t("dialogs.selectBtn"), "Select");
        assert_eq!(i18n.t("running.pendulumLabel"), "Pendulum");
        assert_eq!(i18n.t("variables.title"), "Variables");
        assert_eq!(
            i18n.tf("experiment.lengthValue", &[("value", "50".into())]),
            "Length: 50cm"
        );
        assert_eq!(
            i18n.tf("running.swingCounter", &[("count", "4".into())]),
            "Swings: 4"
        );
    }
}
