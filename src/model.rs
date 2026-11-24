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

impl Phase {
    pub fn label_key(&self) -> &'static str {
        match self {
            Phase::Setup => "setup.bobTitle",
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
}

impl Default for Params {
    fn default() -> Self {
        Self {
            bob_index: 0,
            length: Rule::default().current_length,
            release_angle: Protractor::release().current_angle,
            swing_threshold: Protractor::threshold().current_angle,
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
}
