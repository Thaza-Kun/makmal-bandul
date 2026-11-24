//! Reusable measuring instruments and pendulum drawings.

use wasm_bindgen::JsCast;
use web_sys::{Element, PointerEvent};
use yew::prelude::*;

use crate::domain::{format_time, MeasureTool, Protractor as ProtractorTool, Rule, BOB_PRESETS};
use crate::i18n::I18n;

/// SVG attributes need strings; keep the drawing maths in plain f64/f32.
fn px(value: impl Into<f64>) -> String {
    format!("{:.2}", value.into())
}

#[derive(Properties, PartialEq)]
pub struct BobCardProps {
    pub index: usize,
    pub selected: bool,
    pub onselect: Callback<usize>,
}

#[function_component(BobCard)]
pub fn bob_card(props: &BobCardProps) -> Html {
    let bob = &BOB_PRESETS[props.index];
    let color = bob_color(props.index);
    let index = props.index;
    let onselect = props.onselect.clone();
    html! {
        <button
            type="button"
            class={classes!("bob-card", props.selected.then_some("selected"))}
            aria-pressed={props.selected.to_string()}
            onclick={move |_| onselect.emit(index)}
        >
            <svg width="52" height="52" viewBox="0 0 44 44" aria-hidden="true">
                <circle cx="22" cy="22" r="16" fill={color} stroke="#edf2f4" stroke-width="2" />
                <circle cx="17" cy="16" r="5" fill="#ffffff" opacity="0.35" />
            </svg>
            <strong>{ bob.name.as_ref() }</strong>
        </button>
    }
}

fn bob_color(index: usize) -> &'static str {
    match index {
        0 => "#c9d3e6",
        1 => "#c99a63",
        _ => "#6fc7e8",
    }
}

const RULER_WIDTH: f64 = 320.0;
const RULER_HEIGHT: f64 = 80.0;
const RULER_CENTER: f64 = 160.0;
const RULER_WINDOW: f32 = 0.1;

fn ruler_x(value: f32, center: f32) -> f64 {
    RULER_CENTER + (value - center) as f64 / RULER_WINDOW as f64 * RULER_WIDTH
}

#[derive(Properties, PartialEq)]
pub struct RulerProps {
    pub tool: Rule,
    pub label: String,
    pub onchange: Callback<f32>,
}

/// Horizontal ~10 cm ruler segment; ticks slide while the selected value stays centred.
#[function_component(Ruler)]
pub fn ruler(props: &RulerProps) -> Html {
    let dragging = use_state(|| false);
    let tool = props.tool;

    let value_from_pointer = {
        move |event: &PointerEvent| -> Option<f32> {
            let element = event.current_target()?.dyn_into::<Element>().ok()?;
            let rect = element.get_bounding_client_rect();
            let scale = rect.width() / RULER_WIDTH;
            if scale <= 0.0 {
                return None;
            }
            let local_x = (event.client_x() as f64 - rect.left()) / scale;
            let delta = ((local_x - RULER_CENTER) / RULER_WIDTH) as f32 * RULER_WINDOW;
            Some(tool.quantize(tool.read() + delta))
        }
    };

    let ondown = {
        let dragging = dragging.clone();
        let onchange = props.onchange.clone();
        let read_pointer = value_from_pointer.clone();
        move |event: PointerEvent| {
            if let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) {
                let _ = target.set_pointer_capture(event.pointer_id());
            }
            dragging.set(true);
            if let Some(value) = read_pointer(&event) {
                onchange.emit(value);
            }
        }
    };
    let onmove = {
        let dragging = dragging.clone();
        let onchange = props.onchange.clone();
        let read_pointer = value_from_pointer.clone();
        move |event: PointerEvent| {
            if *dragging {
                if let Some(value) = read_pointer(&event) {
                    onchange.emit(value);
                }
            }
        }
    };
    let onup = {
        let dragging = dragging.clone();
        move |_: PointerEvent| dragging.set(false)
    };
    let onkey = {
        let onchange = props.onchange.clone();
        move |event: KeyboardEvent| {
            let delta = match event.key().as_str() {
                "ArrowUp" | "ArrowRight" => Some(tool.precision),
                "ArrowDown" | "ArrowLeft" => Some(-tool.precision),
                "PageUp" => Some(0.01),
                "PageDown" => Some(-0.01),
                "Home" => Some(tool.min_length - tool.read()),
                "End" => Some(tool.max_length - tool.read()),
                _ => None,
            };
            if let Some(delta) = delta {
                event.prevent_default();
                onchange.emit(tool.quantize(tool.read() + delta));
            }
        }
    };

    let center = tool.read();
    let first = ((center - RULER_WINDOW / 2.0) * 1000.0).ceil() as i32;
    let last = ((center + RULER_WINDOW / 2.0) * 1000.0).floor() as i32;
    let ticks = (first..=last)
        .filter(|mm| *mm >= (tool.min_length * 1000.0) as i32)
        .filter(|mm| *mm <= (tool.max_length * 1000.0) as i32)
        .map(|mm| {
            let value = mm as f32 / 1000.0;
            let x = ruler_x(value, center);
            let (top, label) = if mm % 10 == 0 {
                (10.0, Some(format!("{value:.2}")))
            } else if mm % 5 == 0 {
                (16.0, None)
            } else {
                (21.0, None)
            };
            html! {
                <g key={mm.to_string()}>
                    <line x1={px(x)} y1={px(top)} x2={px(x)} y2="38" stroke="#8d99ae" stroke-width="1.5" />
                    { label.map(|text| html! { <text x={px(x)} y="9" fill="#edf2f4" font-size="10" text-anchor="middle">{ text }</text> }) }
                </g>
            }
        })
        .collect::<Html>();

    html! {
        <svg
            class="ruler-svg"
            viewBox={format!("0 0 {RULER_WIDTH} {RULER_HEIGHT}")}
            xmlns="http://www.w3.org/2000/svg"
            role="slider"
            tabindex="0"
            aria-label={props.label.clone()}
            aria-valuemin={px(tool.min_length)}
            aria-valuemax={px(tool.max_length)}
            aria-valuenow={px(tool.current_length)}
            onpointerdown={ondown}
            onpointermove={onmove}
            onpointerup={onup.clone()}
            onpointercancel={onup}
            onkeydown={onkey}
        >
            <rect x="0" y="38" width={px(RULER_WIDTH)} height="14" rx="4" fill="#1a1c2e" stroke="#8d99ae" />
            { ticks }
            <line x1={px(RULER_CENTER)} y1="4" x2={px(RULER_CENTER)} y2="56" stroke="#ef233c" stroke-width="3" />
            <polygon points="160,4 153,12 167,12" fill="#ef233c" />
            <text x={px(RULER_CENTER)} y="74" fill="#edf2f4" font-size="12" text-anchor="middle">{ format!("{:.3} m", center) }</text>
        </svg>
    }
}

const PRO_WIDTH: f64 = 340.0;
const PRO_HEIGHT: f64 = 210.0;
const PRO_CX: f64 = 170.0;
const PRO_CY: f64 = 185.0;
const PRO_R: f64 = 150.0;

fn pro_point(radius: f64, degrees: f64) -> (f64, f64) {
    let radians = degrees.to_radians();
    (
        PRO_CX + radius * radians.sin(),
        PRO_CY - radius * radians.cos(),
    )
}

#[derive(Properties, PartialEq)]
pub struct ProtractorProps {
    pub tool: ProtractorTool,
    pub label: String,
    pub onchange: Callback<f32>,
}

/// One-sided arc: the instrument measures a release/threshold angle, not ±.
#[function_component(Protractor)]
pub fn protractor(props: &ProtractorProps) -> Html {
    let dragging = use_state(|| false);
    let tool = props.tool;

    let value_from_pointer = {
        move |event: &PointerEvent| -> Option<f32> {
            let element = event.current_target()?.dyn_into::<Element>().ok()?;
            let rect = element.get_bounding_client_rect();
            let scale = rect.width() / PRO_WIDTH;
            if scale <= 0.0 {
                return None;
            }
            let local_x = (event.client_x() as f64 - rect.left()) / scale;
            let local_y = (event.client_y() as f64 - rect.top()) / scale;
            let angle = (local_x - PRO_CX).atan2(PRO_CY - local_y).to_degrees() as f32;
            Some(tool.quantize(angle.clamp(tool.min_angle, tool.max_angle)))
        }
    };

    let ondown = {
        let dragging = dragging.clone();
        let onchange = props.onchange.clone();
        let read_pointer = value_from_pointer.clone();
        move |event: PointerEvent| {
            if let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) {
                let _ = target.set_pointer_capture(event.pointer_id());
            }
            dragging.set(true);
            if let Some(value) = read_pointer(&event) {
                onchange.emit(value);
            }
        }
    };
    let onmove = {
        let dragging = dragging.clone();
        let onchange = props.onchange.clone();
        let read_pointer = value_from_pointer.clone();
        move |event: PointerEvent| {
            if *dragging {
                if let Some(value) = read_pointer(&event) {
                    onchange.emit(value);
                }
            }
        }
    };
    let onup = {
        let dragging = dragging.clone();
        move |_: PointerEvent| dragging.set(false)
    };
    let onkey = {
        let onchange = props.onchange.clone();
        move |event: KeyboardEvent| {
            let delta = match event.key().as_str() {
                "ArrowUp" | "ArrowRight" => Some(1.0),
                "ArrowDown" | "ArrowLeft" => Some(-1.0),
                "PageUp" => Some(10.0),
                "PageDown" => Some(-10.0),
                "Home" => Some(tool.min_angle - tool.read()),
                "End" => Some(tool.max_angle - tool.read()),
                _ => None,
            };
            if let Some(delta) = delta {
                event.prevent_default();
                onchange.emit(tool.quantize(tool.read() + delta));
            }
        }
    };

    let min = tool.min_angle as i32;
    let max = tool.max_angle as i32;
    let (start_x, start_y) = pro_point(PRO_R, min as f64);
    let (end_x, end_y) = pro_point(PRO_R, max as f64);
    let arc = format!("M {start_x} {start_y} A {PRO_R} {PRO_R} 0 0 1 {end_x} {end_y}");
    let ticks = (min..=max)
        .map(|degree| {
            let long = degree % 15 == 0;
            let (x1, y1) = pro_point(if long { PRO_R - 16.0 } else { PRO_R - 8.0 }, degree as f64);
            let (x2, y2) = pro_point(PRO_R, degree as f64);
            let (lx, ly) = pro_point(PRO_R - 30.0, degree as f64);
            html! {
                <g key={degree.to_string()}>
                    <line x1={px(x1)} y1={px(y1)} x2={px(x2)} y2={px(y2)} stroke="#8d99ae" stroke-width="1.5" />
                    { long.then(|| html! { <text x={px(lx)} y={px(ly + 4.0)} fill="#8d99ae" font-size="11" text-anchor="middle">{ degree }</text> }) }
                </g>
            }
        })
        .collect::<Html>();
    let (nx, ny) = pro_point(PRO_R - 4.0, tool.current_angle as f64);

    html! {
        <svg
            class="protractor-svg"
            viewBox={format!("0 0 {PRO_WIDTH} {PRO_HEIGHT}")}
            xmlns="http://www.w3.org/2000/svg"
            role="slider"
            tabindex="0"
            aria-label={props.label.clone()}
            aria-valuemin={px(tool.min_angle)}
            aria-valuemax={px(tool.max_angle)}
            aria-valuenow={px(tool.current_angle)}
            onpointerdown={ondown}
            onpointermove={onmove}
            onpointerup={onup.clone()}
            onpointercancel={onup}
            onkeydown={onkey}
        >
            <path d={arc} fill="none" stroke="#8d99ae" stroke-width="12" stroke-linecap="round" />
            <line x1={px(PRO_CX)} y1={px(PRO_CY)} x2={px(PRO_CX)} y2={px(PRO_CY - PRO_R)} stroke="#8d99ae" stroke-dasharray="4 5" />
            { ticks }
            <line x1={px(PRO_CX)} y1={px(PRO_CY)} x2={px(nx)} y2={px(ny)} stroke="#ef233c" stroke-width="5" stroke-linecap="round" />
            <circle cx={px(PRO_CX)} cy={px(PRO_CY)} r="8" fill="#ef233c" />
            <text x={px(PRO_CX)} y={px(PRO_HEIGHT - 4.0)} fill="#8d99ae" font-size="13" text-anchor="middle">{ format!("{}°", tool.current_angle) }</text>
        </svg>
    }
}

const PIVOT_X: f64 = 210.0;
const PIVOT_Y: f64 = 70.0;
const PENDULUM_SCALE: f64 = 370.0;

fn bob_xy(length: f32, angle_deg: f32) -> (f64, f64) {
    let rod = length as f64 * PENDULUM_SCALE;
    let angle = angle_deg.to_radians() as f64;
    (PIVOT_X + rod * angle.sin(), PIVOT_Y + rod * angle.cos())
}

fn snap(value: f32, precision: f32, min: f32, max: f32) -> f32 {
    ((value / precision).round() * precision).clamp(min, max)
}

fn angle_from_pointer(event: &PointerEvent) -> Option<f32> {
    let element = event.current_target()?.dyn_into::<Element>().ok()?;
    let rect = element.get_bounding_client_rect();
    let scale = rect.width() / 420.0;
    if scale <= 0.0 {
        return None;
    }
    let x = (event.client_x() as f64 - rect.left()) / scale;
    let y = (event.client_y() as f64 - rect.top()) / scale;
    Some((x - PIVOT_X).atan2(y - PIVOT_Y).to_degrees() as f32)
}

fn stand() -> Html {
    html! {
        <g opacity="0.55">
            <rect x="36" y="452" width="190" height="14" rx="5" fill="#2b2d42" stroke="#8d99ae" />
            <line x1="60" y1="452" x2="60" y2="64" stroke="#8d99ae" stroke-width="9" stroke-linecap="round" />
            <line x1="60" y1="70" x2={px(PIVOT_X)} y2={px(PIVOT_Y)} stroke="#8d99ae" stroke-width="7" stroke-linecap="round" />
            <rect x={px(PIVOT_X - 12.0)} y={px(PIVOT_Y - 8.0)} width="24" height="16" rx="4" fill="#2b2d42" stroke="#edf2f4" />
        </g>
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum SetupMode {
    Angle,
    Threshold,
}

#[derive(Properties, PartialEq)]
pub struct SetupPendulumProps {
    pub length: f32,
    pub angle_deg: f32,
    pub threshold_deg: f32,
    pub bob_index: usize,
    pub mode: SetupMode,
    pub onchange: Callback<f32>,
}

/// Highlighted pendulum used by dialogs 3 and 4: drag the bob or the threshold handle.
#[function_component(SetupPendulum)]
pub fn setup_pendulum(props: &SetupPendulumProps) -> Html {
    let dragging = use_state(|| false);
    let (min, max, precision) = match props.mode {
        SetupMode::Angle => (
            ProtractorTool::release().min_angle,
            ProtractorTool::release().max_angle,
            ProtractorTool::release().precision,
        ),
        SetupMode::Threshold => (
            ProtractorTool::threshold().min_angle,
            ProtractorTool::threshold().max_angle,
            ProtractorTool::threshold().precision,
        ),
    };
    let current = match props.mode {
        SetupMode::Angle => props.angle_deg,
        SetupMode::Threshold => props.threshold_deg,
    };

    let ondown = {
        let dragging = dragging.clone();
        let onchange = props.onchange.clone();
        move |event: PointerEvent| {
            if let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) {
                let _ = target.set_pointer_capture(event.pointer_id());
            }
            dragging.set(true);
            if let Some(value) = angle_from_pointer(&event) {
                onchange.emit(snap(value, precision, min, max));
            }
        }
    };
    let onmove = {
        let dragging = dragging.clone();
        let onchange = props.onchange.clone();
        move |event: PointerEvent| {
            if *dragging {
                if let Some(value) = angle_from_pointer(&event) {
                    onchange.emit(snap(value, precision, min, max));
                }
            }
        }
    };
    let onup = {
        let dragging = dragging.clone();
        move |_: PointerEvent| dragging.set(false)
    };
    let onkey = {
        let onchange = props.onchange.clone();
        move |event: KeyboardEvent| {
            let delta = match event.key().as_str() {
                "ArrowUp" | "ArrowRight" => Some(1.0),
                "ArrowDown" | "ArrowLeft" => Some(-1.0),
                "Home" => Some(min - current),
                "End" => Some(max - current),
                _ => None,
            };
            if let Some(delta) = delta {
                event.prevent_default();
                onchange.emit(snap(current + delta, precision, min, max));
            }
        }
    };

    let rod = props.length as f64 * PENDULUM_SCALE;
    let (bx, by) = bob_xy(props.length, props.angle_deg);
    let bob = &BOB_PRESETS[props.bob_index.min(BOB_PRESETS.len() - 1)];
    let radius = 12.0 + bob.radius as f64 * 300.0;
    let (tx, ty) = bob_xy(
        props.length,
        match props.mode {
            SetupMode::Angle => props.angle_deg,
            SetupMode::Threshold => props.threshold_deg,
        },
    );

    html! {
        <svg
            class="pendulum-setup highlighted"
            viewBox="0 0 420 480"
            xmlns="http://www.w3.org/2000/svg"
            role="slider"
            tabindex="0"
            aria-label={match props.mode {
                SetupMode::Angle => "release angle",
                SetupMode::Threshold => "swing threshold",
            }}
            aria-valuemin={px(min)}
            aria-valuemax={px(max)}
            aria-valuenow={px(current)}
            onpointerdown={ondown}
            onpointermove={onmove}
            onpointerup={onup.clone()}
            onpointercancel={onup}
            onkeydown={onkey}
        >
            { stand() }
            <g class="pendulum-glow">
                <line x1={px(PIVOT_X)} y1={px(PIVOT_Y)} x2={px(bx)} y2={px(by)} stroke="#ef233c" stroke-width="5" stroke-linecap="round" />
                <circle cx={px(bx)} cy={px(by)} r={px(radius)} fill={bob_color(props.bob_index)} stroke="#edf2f4" stroke-width="2" />
            </g>
            <line x1={px(PIVOT_X)} y1={px(PIVOT_Y)} x2={px(tx)} y2={px(ty)} stroke="#ef233c" stroke-width="2" stroke-dasharray="7 7" />
            <circle cx={px(tx)} cy={px(ty)} r="11" fill="#ef233c" stroke="#0b1020" stroke-width="3" />
            <circle cx={px(PIVOT_X)} cy={px(PIVOT_Y)} r="7" fill="#ef233c" />
            <text x={px(PIVOT_X + rod * (props.threshold_deg.to_radians() as f64).sin() + 16.0)} y={px(PIVOT_Y + rod * (props.threshold_deg.to_radians() as f64).cos())} fill="#ef233c" font-size="15">{ format!("{}°", props.threshold_deg) }</text>
        </svg>
    }
}

#[derive(Properties, PartialEq)]
pub struct PendulumViewProps {
    pub length: f32,
    pub angle_deg: f32,
    pub threshold_deg: f32,
    pub bob_index: usize,
    pub time: f32,
    pub swings: u32,
}

/// Running view: faded retort stand, one-sided threshold, and a pivot-mounted HUD.
#[function_component(PendulumView)]
pub fn pendulum_view(props: &PendulumViewProps) -> Html {
    let i18n = use_context::<I18n>().expect("i18n context");
    let (bx, by) = bob_xy(props.length, props.angle_deg);
    let bob = &BOB_PRESETS[props.bob_index.min(BOB_PRESETS.len() - 1)];
    let radius = 12.0 + bob.radius as f64 * 300.0;
    let (tx, ty) = bob_xy(props.length, props.threshold_deg);

    html! {
        <div class="viz">
            <svg viewBox="0 0 420 480" xmlns="http://www.w3.org/2000/svg" role="img" aria-label={i18n.t("running.pendulumLabel")}>
                { stand() }
                <line x1={px(PIVOT_X)} y1={px(PIVOT_Y)} x2={px(tx)} y2={px(ty)} stroke="#ef233c" stroke-width="2" stroke-dasharray="7 7" opacity="0.8" />
                <circle cx={px(tx)} cy={px(ty)} r="10" fill="#ef233c" stroke="#2b2d42" stroke-width="3" />
                <line x1={px(PIVOT_X)} y1={px(PIVOT_Y)} x2={px(bx)} y2={px(by)} stroke="#8d99ae" stroke-width="4" stroke-linecap="round" />
                <circle cx={px(bx)} cy={px(by)} r={px(radius)} fill={bob_color(props.bob_index)} stroke="#edf2f4" stroke-width="2" />
                <circle cx={px(PIVOT_X)} cy={px(PIVOT_Y)} r="7" fill="#ef233c" />
            </svg>
            <div class="pivot-hud">
                <StopwatchFace time={props.time} />
                <span class="pivot-counter" aria-label={i18n.t("running.swingLabel")}>
                    <strong aria-live="polite">{ props.swings }</strong>
                </span>
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct StopwatchFaceProps {
    pub time: f32,
}

#[function_component(StopwatchFace)]
pub fn stopwatch_face(props: &StopwatchFaceProps) -> Html {
    let i18n = use_context::<I18n>().expect("i18n context");
    html! {
        <span class="pivot-stopwatch" aria-label={i18n.t("running.timeLabel")}>{ format_time(props.time) }</span>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_numbers_have_a_stable_format() {
        assert_eq!(px(1.5_f64), "1.50");
        assert_eq!(px(0.1_f32), "0.10");
    }

    #[test]
    fn pendulum_position_follows_length_and_angle() {
        let (x, y) = bob_xy(0.5, 30.0);
        assert!(x > PIVOT_X);
        assert!((y - PIVOT_Y - 0.5 * PENDULUM_SCALE * 30f64.to_radians().cos()).abs() < 0.01);
    }
}
