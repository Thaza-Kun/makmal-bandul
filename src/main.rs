mod components;
mod domain;
mod i18n;
mod model;
mod physics;
mod results;
mod storage;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{window, Event};
use yew::prelude::*;

use components::{
    confirm::ConfirmModal, dialogs::VariableModal, results::ResultsPanel, run::ExperimentView,
    setup::VariablesModal,
};
use domain::{MeasureTool, Protractor, Rule, Stopwatch, BOB_PRESETS};
use model::{ConfirmRow, Modal, Params, Phase, Variable};
use physics::{Simulation, FIXED_DT};
use storage::{record_swing, SwingSample, HISTORY_KEY, PARAMS_KEY, READINGS_KEY};

const LANG_KEY: &str = "pendulum_lang";

fn main() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    let i18n = use_state(i18n::I18n::embedded);
    let phase = use_state(|| Phase::Setup);
    let modal = use_state(|| None::<Modal>);
    let params = use_state(|| {
        storage::load::<Params>(PARAMS_KEY)
            .filter(|params| params.valid())
            .unwrap_or_default()
    });
    let rows = use_state(Vec::<ConfirmRow>::new);
    let adding = use_state(|| false);
    let tick = use_state(|| 0u64);
    let simulation = use_mut_ref(|| None::<Simulation>);

    {
        let i18n = i18n.clone();
        use_effect_with((), move |_| {
            let lang = match storage::load::<String>(LANG_KEY) {
                Some(lang) => lang,
                None => window()
                    .and_then(|window| window.navigator().language())
                    .map(|lang| if lang.starts_with("ms") { "ms" } else { "en" }.to_string())
                    .unwrap_or_else(|| "en".to_string()),
            };
            wasm_bindgen_futures::spawn_local(async move {
                i18n.set(i18n::load(&lang).await);
            });
            || ()
        });
    }

    {
        let params = params.clone();
        use_effect_with((*params).clone(), move |current| {
            storage::save(PARAMS_KEY, current);
            || ()
        });
    }

    // One interval while running; physics advances in fixed steps regardless
    // of display refresh rate, and the tick only forces a repaint.
    {
        let running = *phase == Phase::Running;
        let simulation = simulation.clone();
        let tick = tick.clone();
        use_effect_with(running, move |running| -> Box<dyn FnOnce()> {
            let Some(window) = window() else {
                return Box::new(|| ());
            };
            if !*running {
                return Box::new(|| ());
            }
            let mut last = now_ms();
            let mut accumulator = 0.0_f64;
            let closure = Closure::wrap(Box::new(move || {
                let now = now_ms();
                let delta = ((now - last) / 1000.0).min(0.1);
                last = now;
                accumulator += delta;
                let mut guard = simulation.borrow_mut();
                if let Some(sim) = guard.as_mut() {
                    let mut stepped = false;
                    let mut crossed = false;
                    let mut crossed_angle = sim.angle_deg;
                    while accumulator >= FIXED_DT as f64 {
                        if sim.step() {
                            crossed = true;
                            crossed_angle = sim.angle_deg;
                        }
                        accumulator -= FIXED_DT as f64;
                        stepped = true;
                    }
                    drop(guard);
                    if crossed {
                        let mut history: Vec<SwingSample> =
                            storage::load(HISTORY_KEY).unwrap_or_default();
                        record_swing(&mut history, now_ms(), crossed_angle);
                        storage::save(HISTORY_KEY, &history);
                    }
                    if stepped {
                        tick.set(tick.wrapping_add(1));
                    }
                }
            }) as Box<dyn FnMut()>);
            let id = window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    16,
                )
                .expect("interval registration");
            let holder = Rc::new(RefCell::new(Some(closure)));
            let cleanup = holder.clone();
            Box::new(move || {
                window.clear_interval_with_handle(id);
                drop(cleanup.borrow_mut().take());
            })
        });
    }

    let current = simulation.borrow();
    let angle_deg = current
        .as_ref()
        .map(|sim| sim.angle_deg)
        .unwrap_or(params.release_angle);
    let time = current.as_ref().map(|sim| sim.elapsed).unwrap_or(0.0);
    let swings = current.as_ref().map(|sim| sim.swings).unwrap_or(0);
    drop(current);

    let onstart = {
        let simulation = simulation.clone();
        let params = params.clone();
        let phase = phase.clone();
        let tick = tick.clone();
        move |_| {
            let p = &*params;
            *simulation.borrow_mut() = Some(Simulation::new(
                p.length,
                p.release_angle,
                p.swing_threshold,
                &BOB_PRESETS[p.bob_index.min(BOB_PRESETS.len() - 1)],
            ));
            tick.set(*tick + 1);
            phase.set(Phase::Running);
        }
    };
    let onstop = {
        let phase = phase.clone();
        move |_| phase.set(Phase::Paused)
    };
    let onconfirm = {
        let simulation = simulation.clone();
        let params = params.clone();
        let rows = rows.clone();
        let adding = adding.clone();
        let phase = phase.clone();
        move |_| {
            let sim = simulation.borrow();
            let rule = Rule {
                current_length: params.length,
                ..Rule::default()
            };
            let release = Protractor {
                current_angle: params.release_angle,
                ..Protractor::release()
            };
            let watch = Stopwatch {
                elapsed: sim.as_ref().map(|s| s.elapsed).unwrap_or(0.0),
                ..Stopwatch::default()
            };
            let reading = ConfirmRow {
                bob: BOB_PRESETS
                    .get(params.bob_index)
                    .map(|bob| bob.name.to_string())
                    .unwrap_or_default(),
                length: rule.quantize(params.length),
                release_angle: release.quantize(params.release_angle),
                time: watch.quantize(watch.elapsed),
                swings: sim.as_ref().map(|s| s.swings).unwrap_or(0),
            };
            drop(sim);
            let mut next = (*rows).clone();
            record_reading(&mut next, reading, *adding);
            rows.set(next);
            adding.set(false);
            phase.set(Phase::Confirming);
        }
    };
    let onadd = {
        let adding = adding.clone();
        let simulation = simulation.clone();
        let params = params.clone();
        let phase = phase.clone();
        let tick = tick.clone();
        move |_| {
            // Re-run with the same params; the new trial is appended when confirmed.
            let p = &*params;
            *simulation.borrow_mut() = Some(Simulation::new(
                p.length,
                p.release_angle,
                p.swing_threshold,
                &BOB_PRESETS[p.bob_index.min(BOB_PRESETS.len() - 1)],
            ));
            adding.set(true);
            tick.set(*tick + 1);
            phase.set(Phase::Running);
        }
    };
    let onsave = {
        let rows = rows.clone();
        let phase = phase.clone();
        move |_| {
            storage::save(READINGS_KEY, &*rows);
            phase.set(Phase::Results);
        }
    };
    let onnew = {
        let simulation = simulation.clone();
        let rows = rows.clone();
        let adding = adding.clone();
        let modal = modal.clone();
        let phase = phase.clone();
        let tick = tick.clone();
        move |_| {
            *simulation.borrow_mut() = None;
            rows.set(Vec::new());
            adding.set(false);
            modal.set(None);
            tick.set(*tick + 1);
            phase.set(Phase::Setup);
        }
    };
    let onlang = {
        let i18n = i18n.clone();
        move |event: Event| {
            if let Some(lang) = event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::HtmlSelectElement>().ok())
                .map(|select| select.value())
            {
                storage::save(LANG_KEY, &lang);
                let i18n = i18n.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    i18n.set(i18n::load(&lang).await);
                });
            }
        }
    };

    let onsetup = {
        let modal = modal.clone();
        let phase = phase.clone();
        move |_| {
            if *phase == Phase::Running {
                phase.set(Phase::Paused);
            }
            modal.set(Some(Modal::Variables));
        }
    };
    let onopenvariable = {
        let modal = modal.clone();
        move |variable: Variable| modal.set(Some(Modal::Variable(variable)))
    };
    let onclosevariable = {
        let modal = modal.clone();
        move |_| modal.set(Some(Modal::Variables))
    };
    let onclosemodal = {
        let modal = modal.clone();
        move |_| modal.set(None)
    };

    let experiment = html! {
        <ExperimentView
            params={(*params).clone()}
            {angle_deg} {time} {swings}
        />
    };
    let screen = match *phase {
        Phase::Results => html! {
            <ResultsPanel
                rows={(*rows).clone()}
                params={(*params).clone()}
                tonew={onnew.clone()}
            />
        },
        _ => experiment,
    };

    let (action_label, action_class, action) = match *phase {
        Phase::Running => (
            i18n.t("running.stopBtn"),
            "btn action stop",
            Callback::from(onstop.clone()),
        ),
        Phase::Results => (
            i18n.t("results.newExperiment"),
            "btn action primary",
            Callback::from(onnew.clone()),
        ),
        _ => (
            i18n.t("setup.startBtn"),
            "btn action start",
            Callback::from(onstart.clone()),
        ),
    };

    html! {
        <ContextProvider<i18n::I18n> context={(*i18n).clone()}>
            <div class="app">
                <header class="topbar">
                    <div class="brand">
                        <h1>{ i18n.t("app.title") }</h1>
                        <small>{ i18n.t("app.subtitle") }</small>
                    </div>
                    <span class="topbar-spacer" />
                    <span class="offline-pill">{ i18n.t("app.offline") }</span>
                    <span class="phase-pill">{ i18n.t(phase.label_key()) }</span>
                    <label class="sr-only" for="lang-select">{ i18n.t("app.langLabel") }</label>
                    <select id="lang-select" class="lang-select" value={i18n.lang.clone()} onchange={onlang}>
                        <option value="en">{"EN"}</option>
                        <option value="ms">{"MS"}</option>
                    </select>
                </header>
                <main>{ screen }</main>

                <footer class="bottombar">
                    <button type="button" id="setup-btn" class="btn small" onclick={onsetup.clone()}>
                        { i18n.t("setup.panelTitle") }
                    </button>
                    { matches!(*phase, Phase::Paused | Phase::Confirming).then(|| html! {
                        <button type="button" class="btn small confirm-btn" onclick={{
                            let onconfirm = onconfirm.clone();
                            move |_| onconfirm(())
                        }}>{ i18n.t("paused.confirmShort") }</button>
                    }) }
                    { matches!(*phase, Phase::Running | Phase::Paused | Phase::Confirming).then(|| html! {
                        <button type="button" class="btn small danger" onclick={{
                            let onnew = onnew.clone();
                            move |_| onnew(())
                        }}>{ i18n.t("running.resetBtn") }</button>
                    }) }
                    <button type="button" id="primary-action" class={action_class} onclick={move |_| action.emit(())}>
                        { action_label }
                    </button>
                </footer>

                { (*phase == Phase::Confirming).then(|| html! {
                    <ConfirmModal
                        rows={(*rows).clone()}
                        onrows={{
                            let rows = rows.clone();
                            move |next| rows.set(next)
                        }}
                        onadd={onadd.clone()}
                        onback={{
                            let phase = phase.clone();
                            move |_| phase.set(Phase::Paused)
                        }}
                        {onsave}
                    />
                }) }
                { match *modal {
                    Some(Modal::Variables) => html! {
                        <VariablesModal
                            params={(*params).clone()}
                            onopen={onopenvariable.clone()}
                            onclose={onclosemodal.clone()}
                        />
                    },
                    Some(Modal::Variable(variable)) => html! {
                        <VariableModal
                            {variable}
                            params={(*params).clone()}
                            onparams={{
                                let params = params.clone();
                                move |next| params.set(next)
                            }}
                            onclose={onclosevariable.clone()}
                        />
                    },
                    None => html! {},
                } }
            </div>
        </ContextProvider<i18n::I18n>>
    }
}

/// The first confirmation fills its prefilled row; later runs append a trial.
fn record_reading(rows: &mut Vec<ConfirmRow>, reading: ConfirmRow, adding: bool) {
    if adding || rows.is_empty() {
        rows.push(reading);
    } else if let Some(last) = rows.last_mut() {
        *last = reading;
    }
}

fn now_ms() -> f64 {
    window()
        .and_then(|window| window.performance())
        .map(|performance| performance.now())
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_replaces_first_reading_and_appends_additional_trials() {
        let mut rows = Vec::new();
        let first = ConfirmRow {
            bob: "100g".into(),
            length: 0.5,
            release_angle: 30.0,
            time: 10.0,
            swings: 20,
        };
        record_reading(&mut rows, first.clone(), false);
        assert_eq!(rows.len(), 1);

        let corrected = ConfirmRow {
            time: 9.0,
            ..first.clone()
        };
        record_reading(&mut rows, corrected, false);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].time, 9.0);

        record_reading(
            &mut rows,
            ConfirmRow {
                time: 11.0,
                ..first
            },
            true,
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1].time, 11.0);
    }
}
