//! Tabs on the APG keyboard contract, from tabs.css. A port of upstream's `tabs.ts`.
use crate::{
    behavior::roving::{self, use_member, Roster},
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-tabs");
const LIST: Bem = Bem::new("fc-tabs__list");
fc_modifier! { enum ListStyle for LIST { Underline => None, Segmented => "segmented" } }

/// Whether arrowing to a tab also selects it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Activation {
    /// Arrowing selects; right when panels are cheap to show.
    #[default]
    Automatic,
    /// Arrowing moves focus only; Enter or Space selects. For panels that fetch.
    Manual,
}

#[derive(Clone, Copy)]
struct TabsContext {
    roster: Roster,
    selected: Controllable<Option<String>>,
    focus: Signal<Option<u64>>,
    base: Signal<Id>,
    orientation: Orientation,
    activation: Activation,
}
impl TabsContext {
    /// The selected value, or the first enabled tab's before anything is chosen.
    fn current(&self) -> Option<String> {
        self.selected.get().or_else(|| {
            self.roster
                .read()
                .into_iter()
                .find(|m| m.enabled)
                .map(|m| m.value)
        })
    }
}

/// A tab list and its panels, paired by value. Ids, `aria-controls`, and
/// `aria-labelledby` are generated; only the selected panel is rendered.
///
/// Without a value or `default_value`, the first enabled tab is selected once the tabs
/// have rendered; give `default_value` so server-rendered markup includes its panel.
#[component]
pub fn Tabs(
    label: AccessibleName,
    #[props(default)] orientation: Orientation,
    #[props(default)] segmented: bool,
    #[props(default)] activation: Activation,
    #[props(default)] value: Option<ReadSignal<Option<String>>>,
    #[props(default)] default_value: Option<String>,
    #[props(default)] onchange: Option<EventHandler<String>>,
    /// [`Tab`]s.
    tabs: Element,
    /// [`TabPanel`]s.
    children: Element,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let selected = use_controllable(
        value,
        || default_value,
        onchange.map(|onchange| {
            EventHandler::new(move |value: Option<String>| {
                if let Some(value) = value {
                    onchange.call(value);
                }
            })
        }),
    );
    let base = use_id();
    let roster = use_context_provider(Roster::new);
    use_context_provider(|| TabsContext {
        roster,
        selected,
        focus: Signal::new(None),
        base: Signal::new(base),
        orientation,
        activation,
    });
    let attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block()).extend(class.as_deref()),
        &[],
    );
    let list_class = ClassList::new(LIST.block())
        .with_modifier(
            LIST,
            if segmented {
                ListStyle::Segmented
            } else {
                ListStyle::Underline
            },
        )
        .to_string();
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_TABS_CSS } }
        div { ..attributes,
            div { class: list_class, role: "tablist", aria_label: label.to_string(),
                aria_orientation: (orientation == Orientation::Vertical).then_some("vertical"),
                {tabs}
            }
            {children}
        }
    }
}

fn tab_id(base: &Id, value: &str) -> String {
    // Values are author text; keep only id-safe characters so the reference resolves.
    let safe: String = value
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{base}-tab-{safe}")
}

/// One tab. `count` adds a badge such as the number of problems.
#[component]
pub fn Tab(
    value: String,
    #[props(default)] count: Option<u32>,
    #[props(default)] availability: Availability,
    children: Element,
) -> Element {
    let enabled = availability == Availability::Enabled;
    let joined = use_member(&value, availability != Availability::Disabled, enabled);
    let direction = use_direction();
    let Some(mut context) = try_consume_context::<TabsContext>() else {
        return rsx! {};
    };
    let Some((roster, key)) = joined else {
        return rsx! {};
    };
    let current = context.current();
    let selected = current.as_deref() == Some(value.as_str());
    // The focus stop is the selected tab, or wherever manual activation moved it.
    let stop = context.focus.read().map_or(selected, |focus| focus == key);
    let base = context.base.read().clone();
    let id = tab_id(&base, &value);
    let panel = format!("{id}-panel");
    let state = availability_attributes(&availability);
    let own = value.clone();
    let mut choose = move |value: String| {
        context.focus.set(None);
        context.selected.set(Some(value));
    };
    rsx! {
        button { r#type: "button", class: BEM.element("tab"), role: "tab", id,
            aria_selected: selected.to_string(), aria_controls: selected.then_some(panel),
            tabindex: if stop { "0" } else { "-1" },
            onmounted: move |event| roster.mount(key, event.data()),
            onclick: move |_| if enabled { choose(own.clone()); },
            onkeydown: move |event| {
                let key_pressed = event.key();
                let activates = key_pressed == Key::Enter || key_pressed == Key::Character(" ".into());
                if context.activation == Activation::Manual && activates {
                    if enabled {
                        event.prevent_default();
                        choose(value.clone());
                    }
                    return;
                }
                let Some(step) = roving::intent(&key_pressed, context.orientation, direction) else { return };
                let members = roster.peek();
                let Some(index) = members.iter().position(|m| m.key == key) else { return };
                let Some(next) = roving::target(members.len(), index, step, |i| members.get(i).is_some_and(|m| m.enabled)).and_then(|i| members.get(i)) else { return };
                event.prevent_default();
                if context.activation == Activation::Manual {
                    context.focus.set(Some(next.key));
                } else {
                    choose(next.value.clone());
                }
                roving::focus(next);
            },
            ..state,
            {children}
            if let Some(count) = count { span { class: BEM.element("count"), "{count}" } }
        }
    }
}

/// The content for one tab. Rendered only while its tab is selected.
#[component]
pub fn TabPanel(value: String, children: Element) -> Element {
    let Some(context) = try_consume_context::<TabsContext>() else {
        return rsx! {};
    };
    if context.current().as_deref() != Some(value.as_str()) {
        return rsx! {};
    }
    let base = context.base.read().clone();
    let tab = tab_id(&base, &value);
    rsx! {
        div { class: BEM.element("panel"), role: "tabpanel", id: "{tab}-panel", aria_labelledby: "{tab}", tabindex: "0",
            {children}
        }
    }
}
