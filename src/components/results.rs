//! Results phase: computed g, JSON/CSV export and a fresh experiment.

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{window, Blob, HtmlAnchorElement, Url};
use yew::prelude::*;

use crate::domain::{format_precision, Stopwatch, BOB_PRESETS};
use crate::i18n::I18n;
use crate::model::{ConfirmRow, Params};
use crate::results::{summarize, Export, Trial, G_REFERENCE};

#[derive(Properties, PartialEq)]
pub struct ResultsProps {
    pub rows: Vec<ConfirmRow>,
    pub params: Params,
    pub tonew: Callback<()>,
}

#[function_component(ResultsPanel)]
pub fn results_panel(props: &ResultsProps) -> Html {
    let i18n = use_context::<I18n>().expect("i18n context");
    let toast = use_state(String::new);

    let trials: Vec<Trial> = props
        .rows
        .iter()
        .map(|row| Trial {
            time: row.time,
            swings: row.swings,
        })
        .collect();
    let summary = summarize(props.params.length, &trials);
    let time_precision = Stopwatch::default().precision;
    let export = Export {
        bob: BOB_PRESETS
            .get(props.params.bob_index)
            .map(|bob| bob.name.to_string())
            .unwrap_or_default(),
        length_m: props.params.length,
        release_angle_deg: props.params.release_angle,
        swing_threshold_deg: props.params.swing_threshold,
        trials,
    };

    let oncopy = {
        let json = export.to_json();
        let toast = toast.clone();
        let copied = i18n.t("common.copied");
        let failed = i18n.t("common.copyFailed");
        move |_: MouseEvent| {
            let json = json.clone();
            let toast = toast.clone();
            let copied = copied.clone();
            let failed = failed.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = match window() {
                    Some(window) => {
                        JsFuture::from(window.navigator().clipboard().write_text(&json)).await
                    }
                    None => Err(JsValue::NULL),
                };
                toast.set(if result.is_ok() { copied } else { failed });
            });
        }
    };
    let ondownload = {
        let csv = export.to_csv();
        let toast = toast.clone();
        let saved = i18n.t("common.saved");
        move |_: MouseEvent| {
            if download_csv(&csv) {
                toast.set(saved.clone());
            }
        }
    };

    html! {
        <section class="phase-panel">
            <h2>{ i18n.t("results.title") }</h2>
            <div class="summary-grid">
                <div class="summary-card">
                    <span class="summary-label">{ i18n.t("results.period") }</span>
                    <span class="summary-value">{ format_precision(summary.mean_period, time_precision) }</span>
                </div>
                <div class="summary-card">
                    <span class="summary-label">{ i18n.t("results.gEstimate") }</span>
                    <span class="summary-value">{ format_precision(summary.mean_g, 0.01) }</span>
                </div>
                <div class="summary-card">
                    <span class="summary-label">{ i18n.tf("results.error", &[("reference", format!("{G_REFERENCE}"))]) }</span>
                    <span class="summary-value">{ format!("{}%", format_precision(summary.error_pct, 0.1)) }</span>
                </div>
                <div class="summary-card">
                    <span class="summary-label">{ i18n.t("results.trials") }</span>
                    <span class="summary-value">{ summary.trials }</span>
                </div>
            </div>

            <p class="hint">{ i18n.t("results.summary") }</p>
            <div class="btn-row">
                <button type="button" class="btn" onclick={oncopy}>{ i18n.t("results.copyJson") }</button>
                <button type="button" class="btn" onclick={ondownload}>{ i18n.t("results.downloadCsv") }</button>
                <button type="button" class="btn primary" onclick={{
                    let callback = props.tonew.clone();
                    move |_| callback.emit(())
                }}>{ i18n.t("results.newExperiment") }</button>
            </div>

            { (!toast.is_empty()).then(|| html! { <div class="toast" role="status">{ (*toast).clone() }</div> }) }
        </section>
    }
}

fn download_csv(csv: &str) -> bool {
    let Some(window) = window() else { return false };
    let Some(document) = window.document() else {
        return false;
    };
    let array = js_sys::Array::of1(&JsValue::from_str(csv));
    let Ok(blob) = Blob::new_with_str_sequence(&array) else {
        return false;
    };
    let Ok(url) = Url::create_object_url_with_blob(&blob) else {
        return false;
    };
    let Ok(anchor) = document
        .create_element("a")
        .map_err(|_| ())
        .and_then(|el| el.dyn_into::<HtmlAnchorElement>().map_err(|_| ()))
    else {
        return false;
    };
    anchor.set_href(&url);
    anchor.set_download("pendulum-readings.csv");
    anchor.click();
    let _ = Url::revoke_object_url(&url);
    true
}
