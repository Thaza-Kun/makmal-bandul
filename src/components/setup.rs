//! Variable list opened by the persistent Setup button.

use yew::prelude::*;

use crate::domain::{format_precision, Protractor, Rule, BOB_PRESETS};
use crate::i18n::I18n;
use crate::model::{Params, Variable};

#[derive(Properties, PartialEq)]
pub struct VariablesProps {
    pub params: Params,
    pub onopen: Callback<Variable>,
    pub onclose: Callback<()>,
}

#[function_component(VariablesModal)]
pub fn variables_modal(props: &VariablesProps) -> Html {
    let i18n = use_context::<I18n>().expect("i18n context");
    let rule = Rule::default();
    let release = Protractor::release();
    let bob = BOB_PRESETS
        .get(props.params.bob_index)
        .map(|bob| bob.name.to_string())
        .unwrap_or_default();
    let length = format_precision(props.params.length, rule.precision);
    let angle = format_precision(props.params.release_angle, release.precision);

    let row = |id: &'static str, label: String, value: String, variable: Variable| {
        let onopen = props.onopen.clone();
        html! {
            <button
                type="button"
                id={id}
                class="variable-row"
                onclick={move |_| onopen.emit(variable)}
            >
                <span>{ label }</span>
                <strong>{ value }</strong>
            </button>
        }
    };

    html! {
        <div class="modal-backdrop" role="presentation">
            <div class="modal variable-list" role="dialog" aria-modal="true" aria-labelledby="variables-title">
                <h2 id="variables-title">{ i18n.t("variables.title") }</h2>
                <div class="variable-rows">
                    { row("variable-bob", i18n.t("variables.bob"), bob, Variable::Bob) }
                    { row("variable-length", i18n.t("variables.length"), format!("{length} m"), Variable::Length) }
                    { row("variable-angle", i18n.t("variables.angle"), format!("{angle}°"), Variable::Angle) }
                </div>
                <div class="btn-row">
                    <button type="button" class="btn primary" onclick={{
                        let onclose = props.onclose.clone();
                        move |_| onclose.emit(())
                    }}>{ i18n.t("variables.closeBtn") }</button>
                </div>
            </div>
        </div>
    }
}
