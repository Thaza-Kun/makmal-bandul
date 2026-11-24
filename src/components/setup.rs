//! Setup phase: pick a bob and read the three instruments.

use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement};
use yew::prelude::*;

use super::widgets::{BobCard, Protractor, Ruler};
use crate::domain::{MeasureTool, Protractor as ProtractorTool, Rule};
use crate::i18n::I18n;
use crate::model::Params;

macro_rules! number_value {
    ($event:expr) => {
        $event
            .target()
            .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
            .and_then(|input| input.value().parse::<f32>().ok())
    };
}

#[derive(Properties, PartialEq)]
pub struct SetupProps {
    pub params: Params,
    pub onparams: Callback<Params>,
    pub onstart: Callback<()>,
}

#[function_component(SetupPanel)]
pub fn setup_panel(props: &SetupProps) -> Html {
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
        current_angle: params.swing_threshold,
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
            next.length = value;
            onparams.emit(next);
        }
    };
    let set_release = {
        let onparams = props.onparams.clone();
        let base = params.clone();
        move |value: f32| {
            let mut next = base.clone();
            next.release_angle = value;
            onparams.emit(next);
        }
    };
    let set_threshold = {
        let onparams = props.onparams.clone();
        let base = params.clone();
        move |value: f32| {
            let mut next = base.clone();
            next.swing_threshold = value;
            onparams.emit(next);
        }
    };

    let bob_cards = (0..3)
        .map(|index| {
            html! {
                <BobCard
                    key={index.to_string()}
                    {index}
                    selected={params.bob_index == index}
                    onselect={set_bob.clone()}
                />
            }
        })
        .collect::<Html>();

    html! {
        <section class="phase-panel">
            <h2>{ i18n.t("setup.bobTitle") }</h2>
            <p class="hint">{ i18n.t("setup.bobHint") }</p>
            <div class="bob-grid">{ bob_cards }</div>

            <div class="control-grid">
                <div class="card">
                    <h3>{ i18n.t("setup.lengthTitle") }</h3>
                    <p class="hint">{ i18n.t("setup.lengthHint") }</p>
                    <div class="tool-row">
                        <Ruler
                            tool={rule}
                            label={i18n.t("setup.lengthLabel")}
                            onchange={set_length.clone()}
                        />
                        <div>
                            <label class="field-label" for="length-input">{ i18n.t("setup.lengthLabel") }</label>
                            <input
                                id="length-input"
                                class="num-input"
                                type="number"
                                min={px(rule.min_length)}
                                max={px(rule.max_length)}
                                step={px(rule.precision)}
                                value={format!("{:.3}", params.length)}
                                onchange={{
                                    let set = set_length.clone();
                                    let tool = rule;
                                    move |event: Event| {
                                        if let Some(value) = number_value!(event) {
                                            set(tool.quantize(value));
                                        }
                                    }
                                }}
                            />
                        </div>
                    </div>
                </div>

                <div class="card">
                    <h3>{ i18n.t("setup.releaseTitle") }</h3>
                    <p class="hint">{ i18n.t("setup.releaseHint") }</p>
                    <div class="tool-row">
                        <Protractor
                            tool={release}
                            label={i18n.t("setup.releaseLabel")}
                            onchange={set_release.clone()}
                        />
                        <div>
                            <label class="field-label" for="release-input">{ i18n.t("setup.releaseLabel") }</label>
                            <input
                                id="release-input"
                                class="num-input"
                                type="number"
                                min={px(release.min_angle)}
                                max={px(release.max_angle)}
                                step={px(release.precision)}
                                value={format!("{:.0}", params.release_angle)}
                                onchange={{
                                    let set = set_release.clone();
                                    let tool = release;
                                    move |event: Event| {
                                        if let Some(value) = number_value!(event) {
                                            set(tool.quantize(value));
                                        }
                                    }
                                }}
                            />
                        </div>
                    </div>
                </div>

                <div class="card">
                    <h3>{ i18n.t("setup.thresholdTitle") }</h3>
                    <p class="hint">{ i18n.t("setup.thresholdHint") }</p>
                    <div class="tool-row">
                        <Protractor
                            tool={threshold}
                            label={i18n.t("setup.thresholdLabel")}
                            onchange={set_threshold.clone()}
                        />
                        <div>
                            <label class="field-label" for="threshold-input">{ i18n.t("setup.thresholdLabel") }</label>
                            <input
                                id="threshold-input"
                                class="num-input"
                                type="number"
                                min={px(threshold.min_angle)}
                                max={px(threshold.max_angle)}
                                step={px(threshold.precision)}
                                value={format!("{:.0}", params.swing_threshold)}
                                onchange={{
                                    let set = set_threshold.clone();
                                    let tool = threshold;
                                    move |event: Event| {
                                        if let Some(value) = number_value!(event) {
                                            set(tool.quantize(value));
                                        }
                                    }
                                }}
                            />
                        </div>
                    </div>
                </div>
            </div>

            <div class="btn-row">
                <button
                    type="button"
                    class="btn primary"
                    disabled={!params.valid()}
                    onclick={{
                        let onstart = props.onstart.clone();
                        move |_| onstart.emit(())
                    }}
                >
                    { i18n.t("setup.startBtn") }
                </button>
            </div>
            { (!params.valid()).then(|| html! { <p class="hint">{ i18n.t("setup.invalidHint") }</p> }) }
        </section>
    }
}

fn px(value: f32) -> String {
    format!("{value}")
}
