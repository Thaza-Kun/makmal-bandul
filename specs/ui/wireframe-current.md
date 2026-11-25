# Current Wireframe

> Historical snapshot of the pre-wireframe UI. `wireframe.md` is authoritative; the guided wizard, setup panel and pivot HUD described below have been replaced.

Snapshot of the implemented UI (`src/main.rs`, `src/components/`, `style.css`). Desktop (≥900px) places Setup beside the phase screen; mobile stacks them in the same order. The guided wizard is a bottom sheet below 700px and a centered modal above it.

## Mobile Behaviour

```plantuml
@startsalt
{
  {== Sticky topbar (wraps to available width) ==}
  { Makmal Bandul | Works offline | Stage | EN/MS }
  .
  {== Setup card (inline, above main; default expanded) ==}
  { [−] | [Change bob] | [Change length] }
  { Bob 100g | Length 0.500 m | Release 30° | Threshold 5° }
  { [▶ Start] }
  .
  {== Main: viz fills the width ==}
  { Stopwatch 00:00.00 | Count 0 (top centre) }
  { pendulum centred below }
  .
  {== Sticky bottom controls (Running/Paused) ==}
  { [Confirm readings] | [Reset] }
  {       [ Stop / Resume ]       }
}
@endsalt
```

- Below 900px the workspace is one column: the setup panel is a normal inline card above the phase screen, not a sticky sidebar.
- Below 700px modals are bottom sheets — full width, rounded top corners, up to 92vh with internal scrolling; above 700px they are centred with padding.
- The topbar wraps on narrow screens. Primary buttons are 50–56px tall, the language select 44px, and the compact setup-menu buttons 38px; the tap highlight is disabled.
- Ruler, protractor and pendulum SVGs use `touch-action: none` with pointer capture so dragging does not scroll the page; keyboard arrows and Home/End (plus PageUp/PageDown on ruler/protractor) also change values.
- Bob cards and result summary cards reflow to the available width; the readings table scrolls horizontally inside its modal.
- `prefers-reduced-motion` removes the Start/Stop pulse and pendulum glow.

## Persistent Page

```plantuml
@startsalt
{
  {== Sticky topbar (top) ==}
  { Makmal Bandul | Pendulum experiment lab | Works offline | Stage: Setup | EN | MS }
  .
  {== Setup panel (sidebar >=900px / inline card mobile; default expanded) ==}
  { Bob | Length (m) | Release angle | Swing threshold }
  { 100g | 0.500 | 30° | 5° }
  { [−] | [Change bob] | [Change length] | [▶ Start] }
  { Collapsed: [ 100g | 0.500 m | 30° ] -> expand }
  .
  {== Phase screen (main) ==}
  { Setup: hint only | Running/Paused: pendulum view | Confirming: run view + modal | Results: summary }
  .
  {== User interaction (sticky bottom while Running/Paused) ==}
  { [Confirm readings] | [Reset] }
  {       [ Stop (red) / Resume (green) ]       }
}
@endsalt
```

`Start` lives in the setup panel; the bottom action bar appears only once a run exists.

## First-Load Setup Wizard

```plantuml
@startsalt
{
  {== Modal overlay, one dialog at a time ==}
  { 1 Bob: [100g] [30g] [20g] }
  { 2 Length: [−] <ruler, 1 mm divisions> [+] | Length (m) [0.500] }
  { 3 Release: highlighted pendulum (drag bob) | protractor | Angle (°) [30] }
  { 4 Threshold: highlighted pendulum | protractor | Angle (°) [5] }
  { [Select] -> next dialog; after 4 -> Setup with Start enabled }
}
@endsalt
```

Editing bob or length while running pauses the run and reopens this wizard; `Select` returns to Setup for a fresh Start.

## Running / Paused

```plantuml
@startsalt
{
  {== Main (top): stage title + hint ==}
  {== Values (center-x, top of viz) ==}
  { Stopwatch 00:00.00 | Count 0 }
  .
  {== Pendulum (center-x, center-y of viz) ==}
  { faded retort stand | rod + bob | dashed one-sided threshold + handle }
  .
  {== User interaction (bottom, sticky) ==}
  { [Confirm readings] | [Reset] }
  {       [ Stop (red) / Resume (green) ]       }
}
@endsalt
```

Paused freezes physics and the stopwatch; `Confirming` keeps this view under the readings modal. Numeric readouts use tool precision (length 3 decimals, angles and time 2/0 decimals as applicable).

## Confirming

```plantuml
@startsalt
{
  {== Modal overlay over the paused run view ==}
  { Trial | Bob | Length (m) | Release (°) | Time (s) | Swings }
  { 1 | 100g | 0.500 | 30 | 10.00 | 20 }
  { [Back] | [Add reading] | [Save & continue] }
}
@endsalt
```

## Results

```plantuml
@startsalt
{
  {== Main (top): Results ==}
  { Mean period (s) | Estimated g (m/s²) | Error vs 9.81 m/s² | Trials analysed }
  { [Copy JSON] | [Download CSV] | [New experiment] }
}
@endsalt
```

`New experiment` clears the run and readings, reopens the wizard at Bob, and returns to Setup.
