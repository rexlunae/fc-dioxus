//! Resizable split panes and edge handles on the WAI-ARIA window splitter pattern, from
//! splitter.css. The behaviour ports upstream's `splitter.ts`; the arithmetic is in
//! [`crate::behavior::splitter`].
use crate::{
    behavior::splitter::{drag_value, key_value, settle, KeyContext, SplitUnit},
    core::{
        assets,
        attrs::component_attributes,
        measure::{custom_property, CustomProperty},
        platform::DomBridge,
    },
    prelude::*,
    theme::use_stylesheets,
};
use std::rc::Rc;
const SPLIT: Bem = Bem::new("fc-split");
const PANE: Bem = Bem::new("fc-split__pane");
const BAR: Bem = Bem::new("fc-splitter");
const RESIZABLE: Bem = Bem::new("fc-resizable");
fc_modifier! { enum SplitFlow for SPLIT { Row => None, Stacked => "stacked" } }
fc_modifier! { enum PaneRole for PANE { Secondary => None, Primary => "primary" } }
fc_modifier! { enum BarKind for BAR { Between => None, Edge => "edge" } }
fc_modifier! { enum ResizeAxis for RESIZABLE { Inline => None, Block => "block" } }

/// A size change. `settled` is false while a drag is in progress, so an app can persist
/// only sizes the user let go of.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Resize {
    /// The new value, in the splitter's unit.
    pub value: f64,
    /// Whether the size has settled.
    pub settled: bool,
}

#[derive(Clone, Copy, PartialEq)]
struct Drag {
    start: f64,
    origin: f64,
    per_pixel: f64,
}

#[derive(Clone, Copy)]
struct Bar {
    unit: SplitUnit,
    inline: bool,
    growth: f64,
    min: f64,
    max: f64,
    step: f64,
    disabled: bool,
}

/// The separator itself, shared by [`Split`] and [`Resizable`].
#[allow(clippy::too_many_arguments)]
fn separator(
    bar: Bar,
    label: &AccessibleName,
    controls: &Id,
    mut value: Signal<f64>,
    mut restore: Signal<Option<f64>>,
    mut drag: Signal<Option<Drag>>,
    frame: Signal<Option<Rc<MountedData>>>,
    onresize: Option<EventHandler<Resize>>,
    handle: &Id,
    rtl: bool,
) -> Element {
    let Bar {
        unit,
        inline,
        growth,
        min,
        max,
        step,
        disabled,
    } = bar;
    let mut set = move |next: f64, settled: bool| {
        let next = settle(next, min, max, unit);
        // Values are rounded to tenths or whole pixels, so a small margin is exact.
        if (*value.peek() - next).abs() > f64::EPSILON || settled {
            value.set(next);
            if let Some(onresize) = onresize {
                onresize.call(Resize {
                    value: next,
                    settled,
                });
            }
        }
        next
    };
    let class = ClassList::new(BAR.block())
        .with_modifier(
            BAR,
            if unit == SplitUnit::Px {
                BarKind::Edge
            } else {
                BarKind::Between
            },
        )
        .to_string();
    let capture = handle.clone();
    rsx! {
        div { class, id: "{handle}", role: "separator", tabindex: "0",
            aria_orientation: if inline { "vertical" } else { "horizontal" },
            aria_controls: "{controls}", aria_label: label.to_string(),
            aria_valuenow: "{value}", aria_valuemin: "{min}", aria_valuemax: "{max}",
            aria_disabled: disabled.then_some("true"),
            onkeydown: move |event| {
                if disabled { return; }
                if event.key() == Key::Escape {
                    if let Some(start) = drag() {
                        event.prevent_default();
                        event.stop_propagation();
                        drag.set(None);
                        set(start.start, true);
                    }
                    return;
                }
                let context = KeyContext { inline, rtl, growth, current: value(), min, max, step, restore: restore() };
                let Some((next, remember)) = key_value(&event.key(), event.modifiers().shift(), context) else { return };
                event.prevent_default();
                if let Some(remember) = remember { restore.set(Some(remember)); }
                set(next, true);
            },
            onpointerdown: move |event| {
                if disabled || event.trigger_button() != Some(dioxus::html::input_data::MouseButton::Primary) { return; }
                event.prevent_default();
                let point = event.client_coordinates();
                let origin = if inline { point.x } else { point.y };
                let pointer = event.pointer_id();
                let start = value();
                let capture = capture.clone();
                spawn(async move {
                    let per_pixel = if unit == SplitUnit::Percent {
                        let Some(frame) = frame() else { return };
                        let Ok(rect) = frame.get_client_rect().await else { return };
                        let total = if inline { rect.width() } else { rect.height() };
                        if total <= 0.0 { return; }
                        100.0 / total
                    } else {
                        1.0
                    };
                    let _ = DomBridge.capture_pointer(&capture, pointer).await;
                    let _ = DomBridge.focus(&capture).await;
                    drag.set(Some(Drag { start, origin, per_pixel }));
                });
            },
            onpointermove: move |event| {
                let Some(start) = drag() else { return };
                let point = event.client_coordinates();
                let delta = (if inline { point.x } else { point.y }) - start.origin;
                set(drag_value(start.start, delta, start.per_pixel, growth, inline && rtl), false);
            },
            onpointerup: move |_| {
                if drag().is_some() {
                    drag.set(None);
                    let settled = set(value(), true);
                    if settled > min { restore.set(Some(settled)); }
                }
            },
            onpointercancel: move |_| {
                if drag().is_some() {
                    drag.set(None);
                    set(value(), true);
                }
            },
        }
    }
}

/// Two panes and a bar between them. The value is the primary pane's share of the split,
/// in percent, so the layout survives a window resize.
///
/// Arrows move the bar (Shift for ten times as far), Home and End go to the bounds, and
/// Enter collapses the primary pane to the minimum and restores it. A pane at zero is
/// `hidden`, so Tab cannot walk into controls nobody can see. Drag with any pointer;
/// Escape during a drag puts the bar back.
#[component]
pub fn Split(
    label: AccessibleName,
    primary: Element,
    secondary: Element,
    /// Put the secondary pane first, e.g. a context panel at the inline end.
    #[props(default)]
    primary_last: bool,
    /// Stack the panes and drag a horizontal bar.
    #[props(default)]
    stacked: bool,
    /// The split's block size when stacked, as a CSS length.
    #[props(default)]
    block_size: Option<String>,
    #[props(default)] value: Option<ReadSignal<f64>>,
    #[props(default = 50.0)] default_value: f64,
    #[props(default = 0.0)] min: f64,
    #[props(default = 100.0)] max: f64,
    #[props(default = 5.0)] step: f64,
    #[props(default)] onresize: Option<EventHandler<Resize>>,
    /// Name the primary pane as a focusable region, for a long, scrollable pane.
    #[props(default)]
    primary_label: Option<AccessibleName>,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let mut current = use_signal(|| settle(default_value, min, max, SplitUnit::Percent));
    if let Some(value) = value {
        let outer = settle(*value.read(), min, max, SplitUnit::Percent);
        if (*current.peek() - outer).abs() > f64::EPSILON {
            current.set(outer);
        }
    }
    let restore = use_signal(|| None);
    let drag = use_signal(|| None);
    let mut frame = use_signal(|| None);
    let pane = use_id();
    let handle = use_id();
    let rtl = use_direction() == Direction::Rtl;
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(SPLIT.block())
            .with_modifier(
                SPLIT,
                if stacked {
                    SplitFlow::Stacked
                } else {
                    SplitFlow::Row
                },
            )
            .extend(class.as_deref()),
        &["style"],
    );
    if let (true, Some(block_size)) = (stacked, &block_size) {
        attributes.push(custom_property(CustomProperty::SplitBlockSize, block_size));
    }
    let size = current();
    let primary_view = rsx! {
        div { class: ClassList::new(PANE.block()).with_modifier(PANE, PaneRole::Primary).to_string(), id: "{pane}",
            hidden: (size <= 0.0).then_some(true),
            role: primary_label.as_ref().map(|_| "region"), aria_label: primary_label.as_ref().map(ToString::to_string),
            tabindex: primary_label.as_ref().map(|_| "0"),
            ..crate::spread![custom_property(CustomProperty::SplitSize, format!("{size}%"))],
            {primary}
        }
    };
    let bar = Bar {
        unit: SplitUnit::Percent,
        inline: !stacked,
        // Moving the bar toward the end grows a pane that comes before it.
        growth: if primary_last { -1.0 } else { 1.0 },
        min,
        max,
        step,
        disabled: availability != Availability::Enabled,
    };
    let bar = separator(
        bar, &label, &pane, current, restore, drag, frame, onresize, &handle, rtl,
    );
    let secondary_view = rsx! { div { class: PANE.block(), {secondary} } };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_SPLITTER_CSS } }
        div { onmounted: move |event| frame.set(Some(event.data())), ..attributes,
            if primary_last { {secondary_view} {bar} {primary_view} } else { {primary_view} {bar} {secondary_view} }
        }
    }
}

/// An element with a handle on its inline or block end, sized in pixels.
#[component]
pub fn Resizable(
    label: AccessibleName,
    /// Resize the block size (a handle on the bottom) instead of the inline size.
    #[props(default)]
    block: bool,
    #[props(default)] value: Option<ReadSignal<f64>>,
    default_value: f64,
    #[props(default = 0.0)] min: f64,
    max: f64,
    #[props(default = 16.0)] step: f64,
    #[props(default)] onresize: Option<EventHandler<Resize>>,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut current = use_signal(|| settle(default_value, min, max, SplitUnit::Px));
    if let Some(value) = value {
        let outer = settle(*value.read(), min, max, SplitUnit::Px);
        if (*current.peek() - outer).abs() > f64::EPSILON {
            current.set(outer);
        }
    }
    let restore = use_signal(|| None);
    let drag = use_signal(|| None);
    let frame = use_signal(|| None);
    let id = use_id();
    let handle = use_id();
    let rtl = use_direction() == Direction::Rtl;
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(RESIZABLE.block())
            .with_modifier(
                RESIZABLE,
                if block {
                    ResizeAxis::Block
                } else {
                    ResizeAxis::Inline
                },
            )
            .extend(class.as_deref()),
        &["id", "style"],
    );
    attributes.push(custom_property(
        CustomProperty::SplitSize,
        format!("{}px", current()),
    ));
    let bar = Bar {
        unit: SplitUnit::Px,
        inline: !block,
        growth: 1.0,
        min,
        max,
        step,
        disabled: availability != Availability::Enabled,
    };
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_SPLITTER_CSS } }
        div { id: "{id}", ..attributes,
            {children}
            {separator(bar, &label, &id, current, restore, drag, frame, onresize, &handle, rtl)}
        }
    }
}
