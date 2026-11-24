//! Physical measurement instruments and bob presets.
//!
//! Every value the student can edit is owned by one of these tools, so the
//! same precision/min/max rules apply in the UI, in validation and in export.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

/// A real-world measuring instrument with a smallest division and a range.
pub trait MeasureTool {
    type Reading;
    fn precision(&self) -> f32;
    fn min_value(&self) -> f32;
    fn max_value(&self) -> f32;
    fn read(&self) -> Self::Reading;
    fn clamp(&self, value: f32) -> f32 {
        value.clamp(self.min_value(), self.max_value())
    }
    /// Snap to the instrument's smallest division, then clamp to its range.
    fn quantize(&self, value: f32) -> f32 {
        let p = self.precision();
        let snapped = (value / p).round() * p;
        self.clamp(snapped)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    pub precision: f32,
    pub min_length: f32,
    pub max_length: f32,
    pub current_length: f32,
}

impl Default for Rule {
    fn default() -> Self {
        Self {
            precision: 0.001,
            min_length: 0.1,
            max_length: 1.0,
            current_length: 0.5,
        }
    }
}

impl MeasureTool for Rule {
    type Reading = f32;
    fn precision(&self) -> f32 {
        self.precision
    }
    fn min_value(&self) -> f32 {
        self.min_length
    }
    fn max_value(&self) -> f32 {
        self.max_length
    }
    fn read(&self) -> f32 {
        self.current_length
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Stopwatch {
    pub precision: f32,
    pub min_time: f32,
    pub max_time: f32,
    pub elapsed: f32,
    pub running: bool,
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self {
            precision: 0.01,
            min_time: 0.0,
            max_time: 3600.0,
            elapsed: 0.0,
            running: false,
        }
    }
}

impl MeasureTool for Stopwatch {
    type Reading = f32;
    fn precision(&self) -> f32 {
        self.precision
    }
    fn min_value(&self) -> f32 {
        self.min_time
    }
    fn max_value(&self) -> f32 {
        self.max_time
    }
    fn read(&self) -> f32 {
        self.elapsed
    }
}

pub fn format_time(seconds: f32) -> String {
    let centiseconds = (seconds.max(0.0) * 100.0).round() as u64;
    let minutes = centiseconds / 6000;
    let secs = (centiseconds % 6000) / 100;
    let cs = centiseconds % 100;
    format!("{minutes:02}:{secs:02}.{cs:02}")
}

/// Show a reading with exactly the digits its instrument can resolve.
pub fn format_precision(value: f32, precision: f32) -> String {
    let digits = if precision >= 1.0 {
        0
    } else if precision >= 0.1 {
        1
    } else if precision >= 0.01 {
        2
    } else {
        3
    };
    format!("{value:.digits$}")
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Protractor {
    pub precision: f32,
    pub min_angle: f32,
    pub max_angle: f32,
    pub current_angle: f32,
}

impl Protractor {
    pub fn release() -> Self {
        Self {
            precision: 1.0,
            min_angle: 5.0,
            max_angle: 90.0,
            current_angle: 30.0,
        }
    }
    pub fn threshold() -> Self {
        Self {
            precision: 1.0,
            min_angle: 1.0,
            max_angle: 89.0,
            current_angle: 5.0,
        }
    }
}

impl MeasureTool for Protractor {
    type Reading = f32;
    fn precision(&self) -> f32 {
        self.precision
    }
    fn min_value(&self) -> f32 {
        self.min_angle
    }
    fn max_value(&self) -> f32 {
        self.max_angle
    }
    fn read(&self) -> f32 {
        self.current_angle
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendulumBob {
    /// Mass label shown on the card, e.g. "100g".
    pub name: Cow<'static, str>,
    pub mass: f32,
    pub radius: f32,
}

pub const BOB_PRESETS: [PendulumBob; 3] = [
    PendulumBob {
        name: Cow::Borrowed("100g"),
        mass: 0.1,
        radius: 0.0125,
    },
    PendulumBob {
        name: Cow::Borrowed("30g"),
        mass: 0.03,
        radius: 0.015,
    },
    PendulumBob {
        name: Cow::Borrowed("20g"),
        mass: 0.02,
        radius: 0.015,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_quantizes_to_millimetres_and_clamps() {
        let rule = Rule::default();
        assert_eq!(rule.quantize(0.5124), 0.512);
        assert_eq!(rule.quantize(9.0), 1.0);
        assert_eq!(rule.quantize(-1.0), 0.1);
    }

    #[test]
    fn protractor_ranges_match_the_ui() {
        let release = Protractor::release();
        let threshold = Protractor::threshold();
        assert_eq!(release.clamp(200.0), 90.0);
        assert_eq!(threshold.clamp(0.0), 1.0);
        assert_eq!(release.precision(), 1.0);
    }

    #[test]
    fn stopwatch_display_is_centisecond_precision() {
        let mut sw = Stopwatch::default();
        assert_eq!(format_time(sw.elapsed), "00:00.00");
        sw.elapsed = 65.43;
        assert_eq!(format_time(sw.elapsed), "01:05.43");
    }

    #[test]
    fn readings_show_only_the_digits_their_tool_resolves() {
        assert_eq!(format_precision(0.5124, Rule::default().precision), "0.512");
        assert_eq!(
            format_precision(30.0, Protractor::release().precision),
            "30"
        );
        assert_eq!(
            format_precision(9.876, Stopwatch::default().precision),
            "9.88"
        );
    }

    #[test]
    fn bob_cards_are_labelled_by_mass() {
        assert_eq!(BOB_PRESETS[0].name, "100g");
        assert_eq!(BOB_PRESETS[2].name, "20g");
    }
}
