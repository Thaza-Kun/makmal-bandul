//! Collapsible setup summary with live values and the explicit Start action.

use yew::prelude::*;

use crate::domain::{format_precision, Protractor, Rule, BOB_PRESETS};
use crate::i18n::I18n;
use crate::model::Params;

#[derive(Properties, PartialEq)]
pub struct SetupProps {
    pub params: Params,
    pub onbob: Callback<()>,
    pub onlength: Callback<()>,
    pub onstart: Callback<()>,
}

#[function_component(SetupPanel)]
pub fn setup_panel(props: &SetupProps) -> Html {
    let i18n = use_context::<I18n>().expect("i18n context");
    let expanded = use_state(|| true);
    let rule = Rule::default();
    let release = Protractor::release();
    let threshold = Protractor::threshold();
    let bob = BOB_PRESETS
        .get(props.params.bob_index)
        .map(|bob| bob.name.to_string())
        .unwrap_or_default();
    let length = format_precision(props.params.length, rule.precision);
    let angle = format_precision(props.params.release_angle, release.precision);

    if !*expanded {
        return html! {
            <aside class="setup-panel collapsed">
                <button
                    type="button"
                    class="setup-summary"
                    aria-label={i18n.t("setup.expand")}
                    onclick={{
                        let expanded = expanded.clone();
                        move |_| expanded.set(true)
                    }}
                >
                    <span>{ bob }</span>
                    <span>{ format!("{length} m") }</span>
                    <span>{ format!("{angle}°") }</span>
                </button>
            </aside>
        };
    }

    html! {
        <aside class="setup-panel" aria-label={i18n.t("setup.panelTitle")}>
            <header class="setup-panel-head">
                <h2>{ i18n.t("setup.panelTitle") }</h2>
                <button
                    type="button"
                    class="btn small"
                    aria-label={i18n.t("setup.collapse")}
                    onclick={{
                        let expanded = expanded.clone();
                        move |_| expanded.set(false)
                    }}
                >{ "−" }</button>
                <div class="setup-menu">
                    <button type="button" class="btn small" onclick={{
                        let callback = props.onbob.clone();
                        move |_| callback.emit(())
                    }}>{ i18n.t("setup.changeBob") }</button>
                    <button type="button" class="btn small" onclick={{
                        let callback = props.onlength.clone();
                        move |_| callback.emit(())
                    }}>{ i18n.t("setup.changeLength") }</button>
                </div>
            </header>
            <dl class="setup-values">
                <div><dt>{ i18n.t("setup.valueBob") }</dt><dd>{ bob }</dd></div>
                <div><dt>{ i18n.t("setup.lengthLabel") }</dt><dd>{ format!("{length} m") }</dd></div>
                <div><dt>{ i18n.t("setup.releaseLabel") }</dt><dd>{ format!("{angle}°") }</dd></div>
                <div><dt>{ i18n.t("setup.thresholdLabel") }</dt><dd>{ format!("{}°", format_precision(props.params.swing_threshold, threshold.precision)) }</dd></div>
            </dl>
            <button type="button" class="btn start" onclick={{
                let callback = props.onstart.clone();
                move |_| callback.emit(())
            }}>
                <span class="play-icon" aria-hidden="true">{ "▶" }</span>
                { i18n.t("setup.startBtn") }
            </button>
        </aside>
    }
}
