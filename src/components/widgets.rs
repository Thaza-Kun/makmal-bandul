//! Reusable measuring instruments and the pendulum drawing.

use wasm_bindgen::JsCast;
use web_sys::{Element, PointerEvent};
use yew::prelude::*;

use crate::domain::{MeasureTool, Protractor as ProtractorTool, Rule, BOB_PRESETS};

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
            <svg width="44" height="44" viewBox="0 0 44 44" aria-hidden="true">
                <circle cx="22" cy="22" r="16" fill={color} stroke="#eef2ff" stroke-width="2" />
                <circle cx="17" cy="16" r="5" fill="#ffffff" opacity="0.35" />
            </svg>
            <span class="bob-meta">
                <strong>{ bob.name.as_ref() }</strong>
                <small>{ bob.material.as_ref() } { " · " } { format!("{:.3} kg", bob.mass) }</small>
            </span>
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

const RULER_TOP: f64 = 20.0;
const RULER_BOTTOM: f64 = 340.0;
const RULER_VIEW_HEIGHT: f64 = 360.0;
const RULER_VIEW_WIDTH: f64 = 110.0;

fn ruler_y(value: f32, tool: &Rule) -> f64 {
    let fraction =
        ((value - tool.min_length) / (tool.max_length - tool.min_length)).clamp(0.0, 1.0) as f64;
    RULER_BOTTOM - fraction * (RULER_BOTTOM - RULER_TOP)
}

#[derive(Properties, PartialEq)]
pub struct RulerProps {
    pub tool: Rule,
    pub label: String,
    pub onchange: Callback<f32>,
}

#[function_component(Ruler)]
pub fn ruler(props: &RulerProps) -> Html {
    let dragging = use_state(|| false);
    let tool = props.tool;

    let value_from_pointer = {
        move |event: &PointerEvent| -> Option<f32> {
            let element = event.current_target()?.dyn_into::<Element>().ok()?;
            let rect = element.get_bounding_client_rect();
            let scale = rect.width() / RULER_VIEW_WIDTH;
            if scale <= 0.0 {
                return None;
            }
            let local_y = (event.client_y() as f64 - rect.top()) / scale;
            let fraction = ((RULER_BOTTOM - local_y) / (RULER_BOTTOM - RULER_TOP)).clamp(0.0, 1.0);
            Some(
                tool.quantize(
                    tool.min_length + fraction as f32 * (tool.max_length - tool.min_length),
                ),
            )
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
                "ArrowUp" | "ArrowRight" => Some(0.01),
                "ArrowDown" | "ArrowLeft" => Some(-0.01),
                "PageUp" => Some(0.05),
                "PageDown" => Some(-0.05),
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

    let ticks = (0..=90)
        .map(|i| {
            let y = RULER_BOTTOM - (i as f64 / 90.0) * (RULER_BOTTOM - RULER_TOP);
            let (line_x, line_w, label) = if i % 10 == 0 {
                (
                    28.0,
                    16.0,
                    Some(format!("{:.1}", tool.min_length + i as f32 / 100.0)),
                )
            } else if i % 5 == 0 {
                (33.0, 11.0, None)
            } else {
                (37.0, 7.0, None)
            };
            html! {
                <g key={i.to_string()}>
                    <line x1={px(line_x)} y1={px(y)} x2={px(line_x + line_w)} y2={px(y)} stroke="#a9b4d8" stroke-width="1.5" />
                    { label.map(|text| html! { <text x="4" y={px(y + 4.0)} fill="#a9b4d8" font-size="11">{ text }</text> }) }
                </g>
            }
        })
        .collect::<Html>();
    let marker_y = ruler_y(tool.current_length, &tool);

    html! {
        <svg
            class="ruler-svg"
            viewBox={format!("0 0 {RULER_VIEW_WIDTH} {RULER_VIEW_HEIGHT}")}
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
            <rect x="44" y="20" width="34" height="320" rx="4" fill="#1d2745" stroke="#34406a" />
            { ticks }
            <line x1="26" y1={px(marker_y)} x2="96" y2={px(marker_y)} stroke="#ffcf70" stroke-width="4" />
            <polygon
                points={format!("96,{my} 88,{t} 88,{b}", my = px(marker_y), t = px(marker_y - 7.0), b = px(marker_y + 7.0))}
                fill="#ffcf70"
            />
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

    let max = tool.max_angle as i32;
    let (right_x, right_y) = pro_point(PRO_R, max as f64);
    let (left_x, left_y) = pro_point(PRO_R, -(max as f64));
    let arc = format!("M {left_x} {left_y} A {PRO_R} {PRO_R} 0 0 1 {right_x} {right_y}");
    let ticks = (-max..=max)
        .filter(|degree| *degree != 0)
        .map(|degree| {
            let long = degree.abs() % 15 == 0;
            let (x1, y1) = pro_point(if long { PRO_R - 16.0 } else { PRO_R - 8.0 }, degree as f64);
            let (x2, y2) = pro_point(PRO_R, degree as f64);
            let (lx, ly) = pro_point(PRO_R - 30.0, degree as f64);
            html! {
                <g key={degree.to_string()}>
                    <line x1={px(x1)} y1={px(y1)} x2={px(x2)} y2={px(y2)} stroke="#a9b4d8" stroke-width="1.5" />
                    { long.then(|| html! { <text x={px(lx)} y={px(ly + 4.0)} fill="#a9b4d8" font-size="11" text-anchor="middle">{ degree.abs() }</text> }) }
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
            <path d={arc} fill="none" stroke="#34406a" stroke-width="12" stroke-linecap="round" />
            <line x1={px(PRO_CX)} y1={px(PRO_CY)} x2={px(PRO_CX)} y2={px(PRO_CY - PRO_R)} stroke="#34406a" stroke-dasharray="4 5" />
            { ticks }
            <line x1={px(PRO_CX)} y1={px(PRO_CY)} x2={px(nx)} y2={px(ny)} stroke="#5ee0c0" stroke-width="5" stroke-linecap="round" />
            <circle cx={px(PRO_CX)} cy={px(PRO_CY)} r="8" fill="#ffcf70" />
            <text x={px(PRO_CX)} y={px(PRO_HEIGHT - 4.0)} fill="#a9b4d8" font-size="13" text-anchor="middle">{ format!("{}°", tool.current_angle) }</text>
        </svg>
    }
}

#[derive(Properties, PartialEq)]
pub struct PendulumViewProps {
    pub length: f32,
    pub angle_deg: f32,
    pub release_deg: f32,
    pub threshold_deg: f32,
    pub bob_index: usize,
}

#[function_component(PendulumView)]
pub fn pendulum_view(props: &PendulumViewProps) -> Html {
    let scale = 380.0;
    let (cx, cy) = (210.0, 40.0);
    let rod = props.length as f64 * scale;
    let angle = props.angle_deg.to_radians() as f64;
    let bx = cx + rod * angle.sin();
    let by = cy + rod * angle.cos();
    let bob = &BOB_PRESETS[props.bob_index.min(BOB_PRESETS.len() - 1)];
    let radius = 12.0 + bob.radius as f64 * 300.0;

    let threshold_lines = [-props.threshold_deg, props.threshold_deg]
        .iter()
        .map(|deg| {
            let a = deg.to_radians() as f64;
            html! {
                <line
                    x1={px(cx)} y1={px(cy)}
                    x2={px(cx + rod * a.sin())} y2={px(cy + rod * a.cos())}
                    stroke="#ffcf70" stroke-width="2" stroke-dasharray="7 7" opacity="0.75"
                />
            }
        })
        .collect::<Html>();

    let release = props.release_deg.to_radians() as f64;
    let arc = format!(
        "M {x0} {y0} A {r} {r} 0 0 1 {x1} {y1}",
        x0 = px(cx - rod * release.sin()),
        y0 = px(cy + rod * release.cos()),
        r = px(rod),
        x1 = px(cx + rod * release.sin()),
        y1 = px(cy + rod * release.cos()),
    );

    html! {
        <div class="viz">
            <svg viewBox="0 0 420 480" xmlns="http://www.w3.org/2000/svg" role="img" aria-label="Pendulum">
                <line x1={px(cx)} y1={px(cy)} x2={px(cx)} y2={px(cy + 400.0)} stroke="#34406a" stroke-dasharray="3 6" />
                { threshold_lines }
                <path d={arc} fill="none" stroke="#5ee0c0" stroke-width="1" opacity="0.35" />
                <line x1={px(cx)} y1={px(cy)} x2={px(bx)} y2={px(by)} stroke="#a9b4d8" stroke-width="4" stroke-linecap="round" />
                <circle cx={px(bx)} cy={px(by)} r={px(radius)} fill={bob_color(props.bob_index)} stroke="#eef2ff" stroke-width="2" />
                <circle cx={px(cx)} cy={px(cy)} r="9" fill="#ffcf70" />
                <rect x={px(cx - 45.0)} y="14" width="90" height="9" rx="4" fill="#34406a" />
            </svg>
        </div>
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
}
