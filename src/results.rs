//! Period / gravity calculations and export formats.

use serde::{Deserialize, Serialize};

pub const G_REFERENCE: f32 = 9.81;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Trial {
    pub time: f32,
    pub swings: u32,
}

/// One counted swing is a full oscillation (return to the release side).
pub fn period_seconds(trial: &Trial) -> Option<f32> {
    if trial.swings == 0 || trial.time <= 0.0 {
        None
    } else {
        Some(trial.time / trial.swings as f32)
    }
}

pub fn g_estimate(length_m: f32, period_s: f32) -> Option<f32> {
    if length_m <= 0.0 || period_s <= 0.0 {
        None
    } else {
        Some(4.0 * std::f32::consts::PI.powi(2) * length_m / period_s.powi(2))
    }
}

pub fn percent_error(estimate: f32) -> f32 {
    (estimate - G_REFERENCE).abs() / G_REFERENCE * 100.0
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Summary {
    pub trials: usize,
    pub mean_period: f32,
    pub mean_g: f32,
    pub error_pct: f32,
}

pub fn summarize(length_m: f32, trials: &[Trial]) -> Summary {
    let values: Vec<(f32, f32)> = trials
        .iter()
        .filter_map(period_seconds)
        .filter_map(|t| g_estimate(length_m, t).map(|g| (t, g)))
        .collect();
    if values.is_empty() {
        return Summary::default();
    }
    let n = values.len() as f32;
    let mean_period = values.iter().map(|(t, _)| t).sum::<f32>() / n;
    let mean_g = values.iter().map(|(_, g)| g).sum::<f32>() / n;
    Summary {
        trials: values.len(),
        mean_period,
        mean_g,
        error_pct: percent_error(mean_g),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Export {
    pub bob: String,
    pub length_m: f32,
    pub release_angle_deg: f32,
    pub swing_threshold_deg: f32,
    pub trials: Vec<Trial>,
}

impl Export {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }

    pub fn to_csv(&self) -> String {
        let mut out = String::from("trial,time_s,swings,period_s,g_estimate,error_pct\n");
        for (i, trial) in self.trials.iter().enumerate() {
            let period = period_seconds(trial);
            let g = period.and_then(|p| g_estimate(self.length_m, p));
            out.push_str(&format!(
                "{},{},{},{},{},{}\n",
                i + 1,
                trial.time,
                trial.swings,
                period.map(|p| p.to_string()).unwrap_or_default(),
                g.map(|g| g.to_string()).unwrap_or_default(),
                g.map(percent_error).unwrap_or_default()
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn period_and_g_for_a_half_metre_pendulum() {
        // 1 m pendulum has T ≈ 2.006 s at small angles.
        let trial = Trial {
            time: 20.06,
            swings: 10,
        };
        let period = period_seconds(&trial).unwrap();
        assert!((period - 2.006).abs() < 0.001);
        let g = g_estimate(1.0, period).unwrap();
        assert!((g - G_REFERENCE).abs() < 0.06, "g was {g}");
        assert!(percent_error(g) < 1.0);
    }

    #[test]
    fn zero_swings_are_ignored() {
        assert_eq!(
            period_seconds(&Trial {
                time: 5.0,
                swings: 0
            }),
            None
        );
        let summary = summarize(
            1.0,
            &[Trial {
                time: 5.0,
                swings: 0,
            }],
        );
        assert_eq!(summary.trials, 0);
    }

    #[test]
    fn csv_has_a_header_and_one_row_per_trial() {
        let data = Export {
            bob: "100g".into(),
            length_m: 0.5,
            release_angle_deg: 30.0,
            swing_threshold_deg: 5.0,
            trials: vec![Trial {
                time: 14.2,
                swings: 10,
            }],
        };
        assert_eq!(data.to_csv().lines().count(), 2);
        assert!(data.to_json().contains("\"100g\""));
    }
}
