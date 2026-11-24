//! Running and paused phases share one view; paused only changes the controls.

use yew::prelude::*;

use super::widgets::PendulumView;
use crate::domain::format_time;
use crate::i18n::I18n;
use crate::model::Params;

#[derive(Properties, PartialEq)]
pub struct RunProps {
    pub params: Params,
    pub angle_deg: f32,
    pub time: f32,
    pub swings: u32,
    pub paused: bool,
    pub onstop: Callback<()>,
    pub onresume: Callback<()>,
    pub onconfirm: Callback<()>,
    pub onreset: Callback<()>,
}

#[function_component(RunPanel)]
pub fn run_panel(props: &RunProps) -> Html {
    let i18n = use_context::<I18n>().expect("i18n context");
    let (title_key, hint_key, swing_key, counter_key) = if props.paused {
        (
            "paused.title",
            "paused.frozenHint",
            "paused.swingLabel",
            "paused.swingCounter",
        )
    } else {
        (
            "running.title",
            "running.runningHint",
            "running.swingLabel",
            "running.swingCounter",
        )
    };

    html! {
        <section class="phase-panel">
            <h2>{ i18n.t(title_key) }</h2>
            <p class="hint">{ i18n.t(hint_key) }</p>

            <PendulumView
                length={props.params.length}
                angle_deg={props.angle_deg}
                release_deg={props.params.release_angle}
                threshold_deg={props.params.swing_threshold}
                bob_index={props.params.bob_index}
            />

            <div class="readouts">
                <div class="readout counter">
                    <span class="readout-label">{ i18n.t(swing_key) }</span>
                    <span class="readout-value" aria-live="polite">{ props.swings }</span>
                </div>
                <div class="readout">
                    <span class="readout-label">{ i18n.t("running.timeLabel") }</span>
                    <span class="readout-value" aria-live="off">{ format_time(props.time) }</span>
                </div>
            </div>
            <p class="sr-only">{ i18n.tf(counter_key, &[("count", props.swings.to_string())]) }</p>

            <div class="btn-row">
                { if props.paused {
                    html! {
                        <>
                            <button type="button" class="btn primary" onclick={{
                                let callback = props.onresume.clone();
                                move |_| callback.emit(())
                            }}>{ i18n.t("paused.resumeBtn") }</button>
                            <button type="button" class="btn" onclick={{
                                let callback = props.onconfirm.clone();
                                move |_| callback.emit(())
                            }}>{ i18n.t("paused.confirmBtn") }</button>
                        </>
                    }
                } else {
                    html! {
                        <button type="button" class="btn primary" onclick={{
                            let callback = props.onstop.clone();
                            move |_| callback.emit(())
                        }}>{ i18n.t("running.stopBtn") }</button>
                    }
                } }
                <button type="button" class="btn danger" onclick={{
                    let callback = props.onreset.clone();
                    move |_| callback.emit(())
                }}>{ i18n.t("running.resetBtn") }</button>
            </div>
        </section>
    }
}
