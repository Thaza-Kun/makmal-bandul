//! Confirming phase: editable readings in a modal overlay.

use std::rc::Rc;

use wasm_bindgen::JsCast;
use web_sys::{Event, HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use crate::domain::{Protractor, Rule, Stopwatch, BOB_PRESETS};
use crate::i18n::I18n;
use crate::model::ConfirmRow;

type RowUpdater = Rc<dyn Fn(usize, Box<dyn FnOnce(&mut ConfirmRow)>)>;

#[derive(Properties, PartialEq)]
pub struct ConfirmProps {
    pub rows: Vec<ConfirmRow>,
    pub onrows: Callback<Vec<ConfirmRow>>,
    pub onadd: Callback<()>,
    pub onback: Callback<()>,
    pub onsave: Callback<()>,
}

#[function_component(ConfirmModal)]
pub fn confirm_modal(props: &ConfirmProps) -> Html {
    let i18n = use_context::<I18n>().expect("i18n context");
    let rule = Rule::default();
    let release = Protractor::release();
    let stopwatch = Stopwatch::default();

    let update: RowUpdater = {
        let rows = props.rows.clone();
        let onrows = props.onrows.clone();
        Rc::new(move |index, change| {
            let mut next = rows.clone();
            if let Some(row) = next.get_mut(index) {
                change(row);
            }
            onrows.emit(next);
        })
    };

    let table_rows = props
        .rows
        .iter()
        .enumerate()
        .map(|(index, row)| {
            let bob_options = BOB_PRESETS
                .iter()
                .map(|bob| {
                    let name = bob.name.to_string();
                    html! { <option value={name.clone()} selected={row.bob == name}>{ name }</option> }
                })
                .collect::<Html>();

            let on_bob = {
                let update = update.clone();
                move |event: Event| {
                    if let Some(value) = event
                        .target()
                        .and_then(|target| target.dyn_into::<HtmlSelectElement>().ok())
                        .map(|select| select.value())
                    {
                        update(index, Box::new(move |row| row.bob = value));
                    }
                }
            };

            let length_cell = numeric_cell(
                index,
                format!("{:.3}", row.length),
                rule.precision,
                rule.min_length,
                rule.max_length,
                i18n.t("confirm.length"),
                &update,
                |row, value| row.length = value,
            );
            let release_cell = numeric_cell(
                index,
                format!("{:.0}", row.release_angle),
                release.precision,
                release.min_angle,
                release.max_angle,
                i18n.t("confirm.releaseAngle"),
                &update,
                |row, value| row.release_angle = value,
            );
            let time_cell = numeric_cell(
                index,
                format!("{:.2}", row.time),
                stopwatch.precision,
                stopwatch.min_time,
                stopwatch.max_time,
                i18n.t("confirm.time"),
                &update,
                |row, value| row.time = value,
            );
            let swings_cell = numeric_cell(
                index,
                row.swings.to_string(),
                1.0,
                0.0,
                10_000.0,
                i18n.t("confirm.swings"),
                &update,
                |row, value| row.swings = value.max(0.0) as u32,
            );

            html! {
                <tr key={index.to_string()}>
                    <td>{ index + 1 }</td>
                    <td><select class="cell-input" onchange={on_bob}>{ bob_options }</select></td>
                    <td>{ length_cell }</td>
                    <td>{ release_cell }</td>
                    <td>{ time_cell }</td>
                    <td>{ swings_cell }</td>
                </tr>
            }
        })
        .collect::<Html>();

    html! {
        <div class="modal-backdrop" role="presentation">
            <div class="modal" role="dialog" aria-modal="true" aria-labelledby="confirm-title">
                <h2 id="confirm-title">{ i18n.t("confirm.title") }</h2>
                <p class="hint">{ i18n.t("confirm.subtitle") }</p>

                <div class="table-wrap">
                    <table>
                        <thead>
                            <tr>
                                <th>{ i18n.t("confirm.trial") }</th>
                                <th>{ i18n.t("confirm.bob") }</th>
                                <th>{ i18n.t("confirm.length") }</th>
                                <th>{ i18n.t("confirm.releaseAngle") }</th>
                                <th>{ i18n.t("confirm.time") }</th>
                                <th>{ i18n.t("confirm.swings") }</th>
                            </tr>
                        </thead>
                        <tbody>{ table_rows }</tbody>
                    </table>
                </div>
                { props.rows.is_empty().then(|| html! { <p class="hint">{ i18n.t("confirm.noReadings") }</p> }) }
                <p class="hint">{ i18n.t("confirm.editHint") }</p>

                <div class="btn-row">
                    <button type="button" class="btn" onclick={{
                        let callback = props.onback.clone();
                        move |_| callback.emit(())
                    }}>{ i18n.t("confirm.backBtn") }</button>
                    <button type="button" class="btn" onclick={{
                        let callback = props.onadd.clone();
                        move |_| callback.emit(())
                    }}>{ i18n.t("confirm.addReading") }</button>
                    <button type="button" class="btn primary" onclick={{
                        let callback = props.onsave.clone();
                        move |_| callback.emit(())
                    }}>{ i18n.t("confirm.saveBtn") }</button>
                </div>
            </div>
        </div>
    }
}

#[allow(clippy::too_many_arguments)]
fn numeric_cell<F>(
    index: usize,
    value_text: String,
    step: f32,
    min: f32,
    max: f32,
    label: String,
    update: &RowUpdater,
    apply: F,
) -> Html
where
    F: Fn(&mut ConfirmRow, f32) + Clone + 'static,
{
    let input_id = format!("cell-{index}-{label}");
    let onchange = {
        let update = update.clone();
        move |event: Event| {
            if let Some(parsed) = event
                .target()
                .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
                .and_then(|input| input.value().parse::<f32>().ok())
            {
                let clamped = parsed.clamp(min, max);
                let apply = apply.clone();
                update(index, Box::new(move |row| apply(row, clamped)));
            }
        }
    };
    html! {
        <input
            id={input_id}
            class="cell-input"
            type="number"
            aria-label={label}
            min={min.to_string()}
            max={max.to_string()}
            step={step.to_string()}
            value={value_text}
            {onchange}
        />
    }
}
