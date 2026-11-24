//! Running and paused phases share one view; paused only changes the controls.

use yew::prelude::*;

use super::widgets::PendulumView;
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
    let (title_key, hint_key, counter_key) = if props.paused {
        ("paused.title", "paused.frozenHint", "paused.swingCounter")
    } else {
        (
            "running.title",
            "running.runningHint",
            "running.swingCounter",
        )
    };
    let (action_label, action_class, action) = if props.paused {
        (
            i18n.t("paused.resumeBtn"),
            "btn wide action resume",
            props.onresume.clone(),
        )
    } else {
        (
            i18n.t("running.stopBtn"),
            "btn wide action stop",
            props.onstop.clone(),
        )
    };

    html! {
        <section class="phase-panel run-phase">
            <h2>{ i18n.t(title_key) }</h2>
            <p class="hint">{ i18n.t(hint_key) }</p>

            <PendulumView
                length={props.params.length}
                angle_deg={props.angle_deg}
                threshold_deg={props.params.swing_threshold}
                bob_index={props.params.bob_index}
                time={props.time}
                swings={props.swings}
            />
            <p class="sr-only">{ i18n.tf(counter_key, &[("count", props.swings.to_string())]) }</p>

            <div class="run-controls">
                <div class="btn-row">
                    <button type="button" class="btn" onclick={{
                        let callback = props.onconfirm.clone();
                        move |_| callback.emit(())
                    }}>{ i18n.t("paused.confirmBtn") }</button>
                    <button type="button" class="btn danger" onclick={{
                        let callback = props.onreset.clone();
                        move |_| callback.emit(())
                    }}>{ i18n.t("running.resetBtn") }</button>
                </div>
                <button type="button" class={action_class} onclick={move |_| action.emit(())}>
                    { action_label }
                </button>
            </div>
        </section>
    }
}
