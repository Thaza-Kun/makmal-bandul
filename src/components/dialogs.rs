//! Per-variable editor: visual tool, stepper, role toggle and Save.

use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, MouseEvent};
use yew::prelude::*;

use super::widgets::{BobCard, Protractor, Ruler};
use crate::domain::{
    format_precision, MeasureTool, Protractor as ProtractorTool, Rule, BOB_PRESETS,
};
use crate::i18n::I18n;
use crate::model::{Params, Variable, VariableRole};

#[derive(Properties, PartialEq)]
pub struct VariableModalProps {
    pub variable: Variable,
    pub params: Params,
    pub onparams: Callback<Params>,
    pub onclose: Callback<()>,
}

#[function_component(VariableModal)]
pub fn variable_modal(props: &VariableModalProps) -> Html {
    let i18n = use_context::<I18n>().expect("i18n context");
    let rule = Rule {
        current_length: props.params.length,
        ..Rule::default()
    };
    let angle_tool = ProtractorTool {
        current_angle: props.params.release_angle,
        ..ProtractorTool::release()
    };

    let set_bob = {
        let onparams = props.onparams.clone();
        let base = props.params.clone();
        move |index: usize| {
            let mut next = base.clone();
            next.bob_index = index;
            onparams.emit(next);
        }
    };
    let set_length = {
        let onparams = props.onparams.clone();
        let base = props.params.clone();
        move |value: f32| {
            let mut next = base.clone();
            next.length = rule.quantize(value);
            onparams.emit(next);
        }
    };
    let set_angle = {
        let onparams = props.onparams.clone();
        let base = props.params.clone();
        move |value: f32| {
            let mut next = base.clone();
            next.release_angle = angle_tool.quantize(value);
            onparams.emit(next);
        }
    };
    let set_role = {
        let onparams = props.onparams.clone();
        let base = props.params.clone();
        let variable = props.variable;
        move |role: VariableRole| {
            let mut next = base.clone();
            match variable {
                Variable::Bob => next.bob_role = role,
                Variable::Length => next.length_role = role,
                Variable::Angle => next.angle_role = role,
            }
            onparams.emit(next);
        }
    };

    let tool = match props.variable {
        Variable::Bob => html! {
            <div class="bob-grid">
                {(0..BOB_PRESETS.len()).map(|index| html! {
                    <BobCard
                        key={index.to_string()}
                        {index}
                        selected={props.params.bob_index == index}
                        onselect={set_bob.clone()}
                    />
                }).collect::<Html>()}
            </div>
        },
        Variable::Length => html! {
            <Ruler
                tool={rule}
                label={i18n.t("variables.length")}
                onchange={set_length.clone()}
            />
        },
        Variable::Angle => html! {
            <Protractor
                tool={angle_tool}
                bob_index={props.params.bob_index}
                label={i18n.t("variables.angle")}
                onchange={set_angle.clone()}
            />
        },
    };

    let (value, unit, min, max, step) = match props.variable {
        Variable::Bob => (
            BOB_PRESETS
                .get(props.params.bob_index)
                .map(|bob| format!("{}", (bob.mass * 1000.0).round()))
                .unwrap_or_default(),
            i18n.t("common.unitGram"),
            "20".to_string(),
            "100".to_string(),
            "1".to_string(),
        ),
        Variable::Length => (
            format_precision(props.params.length, rule.precision),
            i18n.t("common.unitMetre"),
            px(rule.min_length),
            px(rule.max_length),
            px(rule.precision),
        ),
        Variable::Angle => (
            format_precision(props.params.release_angle, angle_tool.precision),
            i18n.t("common.unitDegree"),
            px(angle_tool.min_angle),
            px(angle_tool.max_angle),
            px(angle_tool.precision),
        ),
    };

    let change_step = {
        let variable = props.variable;
        let params = props.params.clone();
        let set_bob = set_bob.clone();
        let set_length = set_length.clone();
        let set_angle = set_angle.clone();
        Callback::from(move |delta: f32| match variable {
            Variable::Bob => {
                let index = if delta > 0.0 {
                    (params.bob_index + 1).min(BOB_PRESETS.len() - 1)
                } else {
                    params.bob_index.saturating_sub(1)
                };
                set_bob(index);
            }
            Variable::Length => set_length(params.length + delta * rule.precision),
            Variable::Angle => set_angle(params.release_angle + delta * angle_tool.precision),
        })
    };
    let down = {
        let change_step = change_step.clone();
        Callback::from(move |_: MouseEvent| change_step.emit(-1.0))
    };
    let up = {
        let change_step = change_step.clone();
        Callback::from(move |_: MouseEvent| change_step.emit(1.0))
    };
    let oninput = {
        let variable = props.variable;
        let set_bob = set_bob.clone();
        let set_length = set_length.clone();
        let set_angle = set_angle.clone();
        Callback::from(move |event: Event| {
            let Some(text) = event
                .target()
                .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
                .and_then(|input| input.value().parse::<f32>().ok())
            else {
                return;
            };
            match variable {
                Variable::Bob => set_bob(bob_index_from_grams(text)),
                Variable::Length => set_length(text),
                Variable::Angle => set_angle(text),
            }
        })
    };

    let (title_id, name_key, role) = match props.variable {
        Variable::Bob => ("variable-bob-title", "variables.bob", props.params.bob_role),
        Variable::Length => (
            "variable-length-title",
            "variables.length",
            props.params.length_role,
        ),
        Variable::Angle => (
            "variable-angle-title",
            "variables.angle",
            props.params.angle_role,
        ),
    };
    let manipulated = role == VariableRole::Manipulated;

    html! {
        <div class="modal-backdrop" role="presentation">
            <div class="modal variable-modal" role="dialog" aria-modal="true" aria-labelledby={title_id}>
                <h2 id={title_id}>{ i18n.t(name_key) }</h2>
                <div class="variable-tool">{ tool }</div>
                <div class="stepper">
                    <button type="button" id="variable-down" class="btn small" aria-label={i18n.t("common.decrease")} onclick={down}>{ "−" }</button>
                    <div class="value-input">
                        <input
                            id="variable-value"
                            type="number"
                            min={min}
                            max={max}
                            step={step}
                            value={value}
                            onchange={oninput}
                        />
                        <span class="unit">{ unit }</span>
                    </div>
                    <button type="button" id="variable-up" class="btn small" aria-label={i18n.t("common.increase")} onclick={up}>{ "+" }</button>
                </div>
                <div class="role-section">
                    <p class="role-heading">{ i18n.t("variables.roleHeading") }</p>
                    <div class="role-row">
                        <span>{ i18n.t(name_key) }</span>
                        <button
                            type="button"
                            id="variable-role"
                            class={classes!("btn", "role-toggle", manipulated.then_some("manipulated"))}
                            aria-pressed={manipulated.to_string()}
                            onclick={{
                                let set_role = set_role.clone();
                                let next = if manipulated { VariableRole::Constant } else { VariableRole::Manipulated };
                                move |_| set_role(next)
                            }}
                        >
                            <span>{ i18n.t(if manipulated { "variables.manipulated" } else { "variables.constant" }) }</span>
                            <span class="switch" aria-hidden="true" />
                        </button>
                    </div>
                </div>
                <div class="btn-row">
                    <button type="button" class="btn primary" onclick={{
                        let onclose = props.onclose.clone();
                        move |_| onclose.emit(())
                    }}>{ i18n.t("variables.saveBtn") }</button>
                </div>
            </div>
        </div>
    }
}

fn px(value: f32) -> String {
    format!("{value}")
}

/// Mass presets are discrete; typing a mass picks the nearest available bob.
fn bob_index_from_grams(grams: f32) -> usize {
    BOB_PRESETS
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = (a.mass * 1000.0 - grams).abs();
            let db = (b.mass * 1000.0 - grams).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_mass_selects_the_nearest_bob() {
        assert_eq!(bob_index_from_grams(100.0), 0);
        assert_eq!(bob_index_from_grams(30.0), 1);
        assert_eq!(bob_index_from_grams(24.0), 2);
        assert_eq!(bob_index_from_grams(25.0), 1);
        assert_eq!(bob_index_from_grams(500.0), 0);
    }
}
