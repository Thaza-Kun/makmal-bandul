//! Experiment view shared by Setup, Running, Paused and Confirming.

use yew::prelude::*;

use super::widgets::PendulumView;
use crate::domain::{format_precision, format_time, Protractor};
use crate::i18n::I18n;
use crate::model::Params;

#[derive(Properties, PartialEq)]
pub struct RunProps {
    pub params: Params,
    pub angle_deg: f32,
    pub time: f32,
    pub swings: u32,
}

#[function_component(ExperimentView)]
pub fn experiment_view(props: &RunProps) -> Html {
    let i18n = use_context::<I18n>().expect("i18n context");
    let release = Protractor::release();
    let length_cm = (props.params.length * 100.0).round();
    let angle = format_precision(props.angle_deg, release.precision);

    html! {
        <section class="experiment" aria-label={i18n.t("experiment.title")}>
            <div class="values">
                <div class="value-box">
                    <span class="value-label">{ i18n.t("experiment.stopwatch") }</span>
                    <strong class="value-number" aria-label={i18n.t("running.timeLabel")}>{ format_time(props.time) }</strong>
                </div>
                <div class="value-box">
                    <span class="value-label">{ i18n.t("experiment.count") }</span>
                    <strong class="value-number" aria-label={i18n.t("running.swingLabel")} aria-live="polite">{ props.swings }</strong>
                </div>
            </div>

            <PendulumView
                length={props.params.length}
                angle_deg={props.angle_deg}
                threshold_deg={props.params.swing_threshold}
                bob_index={props.params.bob_index}
            />

            <div class="readouts">
                <span>{ i18n.tf("experiment.lengthValue", &[("value", length_cm.to_string())]) }</span>
                <span>{ i18n.tf("experiment.angleValue", &[("value", angle)]) }</span>
            </div>
            <p class="sr-only">{ i18n.tf("running.swingCounter", &[("count", props.swings.to_string())]) }</p>
        </section>
    }
}
