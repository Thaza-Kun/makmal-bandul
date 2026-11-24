//! Guided setup dialogs shown on first load and after a reset.

use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

use super::widgets::{BobCard, Protractor, Ruler, SetupMode, SetupPendulum};
use crate::domain::{
    format_precision, MeasureTool, Protractor as ProtractorTool, Rule, BOB_PRESETS,
};
use crate::i18n::I18n;
use crate::model::{Params, SetupDialog};

#[derive(Properties, PartialEq)]
pub struct SetupDialogsProps {
    pub dialog: SetupDialog,
    pub params: Params,
    pub onparams: Callback<Params>,
    pub onclose: Callback<()>,
}

#[function_component(SetupDialogs)]
pub fn setup_dialogs(props: &SetupDialogsProps) -> Html {
    let i18n = use_context::<I18n>().expect("i18n context");
    let params = props.params.clone();
    let rule = Rule {
        current_length: params.length,
        ..Rule::default()
    };
    let release = ProtractorTool {
        current_angle: params.release_angle,
        ..ProtractorTool::release()
    };
    let threshold = ProtractorTool {
        current_angle: props.params.swing_threshold,
        ..ProtractorTool::threshold()
    };

    let set_bob = {
        let onparams = props.onparams.clone();
        let base = params.clone();
        move |index: usize| {
            let mut next = base.clone();
            next.bob_index = index;
            onparams.emit(next);
        }
    };
    let set_length = {
        let onparams = props.onparams.clone();
        let base = params.clone();
        move |value: f32| {
            let mut next = base.clone();
            next.length = rule.quantize(value);
            onparams.emit(next);
        }
    };
    let set_release = {
        let onparams = props.onparams.clone();
        let base = params.clone();
        move |value: f32| {
            let mut next = base.clone();
            next.release_angle = release.quantize(value);
            onparams.emit(next);
        }
    };
    let set_threshold = {
        let onparams = props.onparams.clone();
        let base = params.clone();
        move |value: f32| {
            let mut next = base.clone();
            next.swing_threshold = threshold.quantize(value);
            onparams.emit(next);
        }
    };

    let bob_cards = (0..BOB_PRESETS.len())
        .map(|index| {
            html! {
                <BobCard
                    key={index.to_string()}
                    {index}
                    selected={props.params.bob_index == index}
                    onselect={set_bob.clone()}
                />
            }
        })
        .collect::<Html>();

    let (title_key, hint_key, body) = match props.dialog {
        SetupDialog::Bob => (
            "dialogs.bobTitle",
            "dialogs.bobHint",
            html! { <div class="bob-grid">{ bob_cards }</div> },
        ),
        SetupDialog::Length => (
            "dialogs.lengthTitle",
            "dialogs.lengthHint",
            html! {
                <>
                    <div class="length-stepper">
                        <button type="button" class="btn small" aria-label={i18n.t("common.decrement")} onclick={{
                            let set_length = set_length.clone();
                            let current = props.params.length;
                            move |_| set_length(current - rule.precision)
                        }}>{ "−" }</button>
                        <Ruler
                            tool={rule}
                            label={i18n.t("setup.lengthLabel")}
                            onchange={set_length.clone()}
                        />
                        <button type="button" class="btn small" aria-label={i18n.t("common.increment")} onclick={{
                            let set_length = set_length.clone();
                            let current = props.params.length;
                            move |_| set_length(current + rule.precision)
                        }}>{ "+" }</button>
                    </div>
                    <NumberField
                        id="dialog-length"
                        label={i18n.t("setup.lengthLabel")}
                        value={format!("{}", format_precision(props.params.length, rule.precision))}
                        min={px(rule.min_length)}
                        max={px(rule.max_length)}
                        step={px(rule.precision)}
                        onchange={set_length.clone()}
                    />
                </>
            },
        ),
        SetupDialog::Release => (
            "dialogs.releaseTitle",
            "dialogs.releaseHint",
            html! {
                <>
                    <SetupPendulum
                        length={props.params.length}
                        angle_deg={props.params.release_angle}
                        threshold_deg={props.params.swing_threshold}
                        bob_index={props.params.bob_index}
                        mode={SetupMode::Angle}
                        onchange={set_release.clone()}
                    />
                    <div class="dialog-controls">
                        <Protractor
                            tool={release}
                            label={i18n.t("setup.releaseLabel")}
                            onchange={set_release.clone()}
                        />
                        <NumberField
                            id="dialog-release"
                            label={i18n.t("setup.releaseLabel")}
                            value={format!("{}", format_precision(props.params.release_angle, release.precision))}
                            min={px(release.min_angle)}
                            max={px(release.max_angle)}
                            step={px(release.precision)}
                            onchange={set_release}
                        />
                    </div>
                </>
            },
        ),
        SetupDialog::Threshold => (
            "dialogs.thresholdTitle",
            "dialogs.thresholdHint",
            html! {
                <>
                    <SetupPendulum
                        length={props.params.length}
                        angle_deg={props.params.release_angle}
                        threshold_deg={props.params.swing_threshold}
                        bob_index={props.params.bob_index}
                        mode={SetupMode::Threshold}
                        onchange={set_threshold.clone()}
                    />
                    <div class="dialog-controls">
                        <Protractor
                            tool={threshold}
                            label={i18n.t("setup.thresholdLabel")}
                            onchange={set_threshold.clone()}
                        />
                        <NumberField
                            id="dialog-threshold"
                            label={i18n.t("setup.thresholdLabel")}
                            value={format!("{}", format_precision(props.params.swing_threshold, threshold.precision))}
                            min={px(threshold.min_angle)}
                            max={px(threshold.max_angle)}
                            step={px(threshold.precision)}
                            onchange={set_threshold}
                        />
                    </div>
                </>
            },
        ),
    };

    html! {
        <div class="modal-backdrop" role="presentation">
            <div class="modal setup-dialog" role="dialog" aria-modal="true" aria-labelledby="setup-dialog-title">
                <h2 id="setup-dialog-title">{ i18n.t(title_key) }</h2>
                <p class="hint">{ i18n.t(hint_key) }</p>
                { body }
                <div class="btn-row">
                    <button type="button" class="btn primary" onclick={{
                        let callback = props.onclose.clone();
                        move |_| callback.emit(())
                    }}>{ i18n.t("dialogs.selectBtn") }</button>
                </div>
            </div>
        </div>
    }
}

fn px(value: f32) -> String {
    format!("{value}")
}

#[derive(Properties, PartialEq)]
struct NumberFieldProps {
    id: String,
    label: String,
    value: String,
    min: String,
    max: String,
    step: String,
    onchange: Callback<f32>,
}

#[function_component(NumberField)]
fn number_field(props: &NumberFieldProps) -> Html {
    let onchange = {
        let callback = props.onchange.clone();
        move |event: Event| {
            if let Some(value) = event
                .target()
                .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
                .and_then(|input| input.value().parse::<f32>().ok())
            {
                callback.emit(value);
            }
        }
    };
    html! {
        <div class="number-field">
            <label class="field-label" for={props.id.clone()}>{ props.label.clone() }</label>
            <input
                id={props.id.clone()}
                class="num-input"
                type="number"
                min={props.min.clone()}
                max={props.max.clone()}
                step={props.step.clone()}
                value={props.value.clone()}
                {onchange}
            />
        </div>
    }
}
