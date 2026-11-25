//! The state machine's serialisable data.

use serde::{Deserialize, Serialize};

use crate::domain::{Protractor, Rule, BOB_PRESETS};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Phase {
    Setup,
    Running,
    Paused,
    Confirming,
    Results,
}

/// The three parameters exposed by the wireframe's Variables modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variable {
    Bob,
    Length,
    Angle,
}

/// Which modal is open; `None` leaves the experiment visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modal {
    Variables,
    Variable(Variable),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariableRole {
    #[default]
    Constant,
    Manipulated,
}

impl Phase {
    pub fn label_key(&self) -> &'static str {
        match self {
            Phase::Setup => "setup.panelTitle",
            Phase::Running => "running.title",
            Phase::Paused => "paused.title",
            Phase::Confirming => "confirm.title",
            Phase::Results => "results.title",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Params {
    pub bob_index: usize,
    pub length: f32,
    pub release_angle: f32,
    pub swing_threshold: f32,
    #[serde(default)]
    pub bob_role: VariableRole,
    #[serde(default)]
    pub length_role: VariableRole,
    #[serde(default)]
    pub angle_role: VariableRole,
}

impl Default for Params {
    fn default() -> Self {
        Self {
            bob_index: 0,
            length: Rule::default().current_length,
            release_angle: Protractor::release().current_angle,
            swing_threshold: Protractor::threshold().current_angle,
            bob_role: VariableRole::Constant,
            length_role: VariableRole::Constant,
            angle_role: VariableRole::Constant,
        }
    }
}

impl Params {
    pub fn valid(&self) -> bool {
        let rule = Rule::default();
        let release = Protractor::release();
        let threshold = Protractor::threshold();
        self.bob_index < BOB_PRESETS.len()
            && self.length >= rule.min_length
            && self.length <= rule.max_length
            && self.release_angle >= release.min_angle
            && self.release_angle <= release.max_angle
            && self.swing_threshold >= threshold.min_angle
            && self.swing_threshold <= threshold.max_angle
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfirmRow {
    pub bob: String,
    pub length: f32,
    pub release_angle: f32,
    pub time: f32,
    pub swings: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_validate_their_ranges() {
        let mut params = Params::default();
        assert!(params.valid());
        params.length = 2.0;
        assert!(!params.valid());
        params.length = 0.5;
        params.bob_index = 99;
        assert!(!params.valid());
    }

    #[test]
    fn saved_params_without_roles_stay_constant() {
        let legacy: Params = serde_json::from_str(
            r#"{"bob_index":1,"length":0.5,"release_angle":30.0,"swing_threshold":5.0}"#,
        )
        .unwrap();
        assert_eq!(legacy.bob_index, 1);
        assert_eq!(legacy.bob_role, VariableRole::Constant);
        assert_eq!(legacy.length_role, VariableRole::Constant);
        assert_eq!(legacy.angle_role, VariableRole::Constant);
    }
}
