//! Labelled form fields, inputs, input groups, choice cards, and the error summary, from
//! field.css and form-variants.css.
use crate::{
    core::{
        assets,
        attrs::{availability_attributes, component_attributes},
    },
    prelude::*,
    theme::use_stylesheets,
};
const FIELD: Bem = Bem::new("fc-field");
const INPUT: Bem = Bem::new("fc-input");
const GROUP: Bem = Bem::new("fc-input-group");
const SUMMARY: Bem = Bem::new("fc-error-summary");
const CHOICE: Bem = Bem::new("fc-choice");
const CHECK: Bem = Bem::new("fc-check");
fc_modifier! { enum FieldFlow for FIELD { Stacked => None, Inline => "inline" } }
fc_modifier! {
    /// Input sizes.
    pub enum InputSize for INPUT {
        /// Default.
        Md => None,
        /// Small.
        Sm => "sm",
        /// Large.
        Lg => "lg",
    }
}
fc_modifier! {
    /// Input surface treatments.
    pub enum InputVariant for INPUT {
        /// A bordered box.
        Outlined => None,
        /// A tinted fill.
        Filled => "filled",
        /// A soft, borderless tint.
        Soft => "soft",
        /// A single underline.
        Underlined => "underlined",
    }
}
fc_modifier! { enum InputGrowth for INPUT { Fixed => None, Autosize => "autosize" } }

fn stylesheets() -> Element {
    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_FIELD_CSS }
            document::Stylesheet { href: assets::COMPONENTS_FORM_VARIANTS_CSS }
        }
    }
}

/// Whether a field is marked as required or optional.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Necessity {
    /// No marker.
    #[default]
    Unmarked,
    /// Marked with an asterisk, and the control gets `required`.
    Required,
    /// Marked "(optional)".
    Optional,
}

/// What a [`Field`] tells the control inside it, so labels and descriptions are wired
/// by construction rather than by hand.
#[derive(Debug, Clone, PartialEq)]
struct FieldContext {
    control: Id,
    hint: Option<Id>,
    error: Option<Id>,
    required: bool,
}

/// Control wiring from the enclosing field, or none outside a field.
struct Wiring {
    id: Option<String>,
    describedby: Option<String>,
    invalid: bool,
    required: bool,
}
fn use_wiring(invalid: bool) -> Wiring {
    let context = try_consume_context::<Signal<FieldContext>>().map(|c| c.read().clone());
    let describedby = context.as_ref().and_then(|c| {
        let ids: Vec<&str> = [c.hint.as_ref(), c.error.as_ref()]
            .into_iter()
            .flatten()
            .map(Id::as_str)
            .collect();
        (!ids.is_empty()).then(|| ids.join(" "))
    });
    Wiring {
        id: context.as_ref().map(|c| c.control.to_string()),
        describedby,
        invalid: invalid || context.as_ref().is_some_and(|c| c.error.is_some()),
        required: context.is_some_and(|c| c.required),
    }
}

/// A visible label, an optional hint and error, and one control, wired together.
///
/// The control inside ([`Input`], [`Textarea`], [`NativeSelect`]) takes its `id`,
/// `aria-describedby`, and `aria-invalid` from the field, so the label is always
/// associated and the error is always announced with it.
#[component]
pub fn Field(
    label: Element,
    #[props(default)] hint: Option<Element>,
    #[props(default)] error: Option<Element>,
    #[props(default)] necessity: Necessity,
    /// Label beside the control instead of above it.
    #[props(default)]
    inline: bool,
    /// The control's id, when something else links to it, such as an [`ErrorSummary`].
    /// Generated when omitted.
    #[props(default)]
    control: Option<Id>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let generated = use_id();
    let control = control.unwrap_or(generated);
    let context = FieldContext {
        hint: hint.is_some().then(|| control.suffixed("hint")),
        error: error.is_some().then(|| control.suffixed("error")),
        required: necessity == Necessity::Required,
        control,
    };
    let mut shared = use_context_provider(|| Signal::new(context.clone()));
    if *shared.peek() != context {
        shared.set(context.clone());
    }
    let attributes = component_attributes(
        attributes,
        ClassList::new(FIELD.block())
            .with_modifier(
                FIELD,
                if inline {
                    FieldFlow::Inline
                } else {
                    FieldFlow::Stacked
                },
            )
            .extend(class.as_deref()),
        &[],
    );
    rsx! {
        {stylesheets()}
        div { ..attributes,
            label { class: FIELD.element("label"), r#for: "{context.control}",
                {label}
                match necessity {
                    Necessity::Required => rsx! { span { class: FIELD.element("required"), aria_hidden: "true", "*" } },
                    Necessity::Optional => rsx! { span { class: FIELD.element("optional"), "(optional)" } },
                    Necessity::Unmarked => rsx! {},
                }
            }
            if let (Some(hint), Some(id)) = (hint, context.hint.clone()) { p { class: FIELD.element("hint"), id: "{id}", {hint} } }
            {children}
            if let (Some(error), Some(id)) = (error, context.error.clone()) { p { class: FIELD.element("error"), id: "{id}", {error} } }
        }
    }
}

/// The HTML input types this library styles. Text is the default and is not written,
/// as in upstream's markup.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum InputType {
    /// Free text.
    #[default]
    Text,
    /// An email address; mobile keyboards adapt.
    Email,
    /// A password, masked.
    Password,
    /// A search query.
    Search,
    /// A URL.
    Url,
    /// A telephone number.
    Tel,
    /// A number.
    Number,
}
impl InputType {
    const fn attribute(self) -> Option<&'static str> {
        match self {
            Self::Text => None,
            Self::Email => Some("email"),
            Self::Password => Some("password"),
            Self::Search => Some("search"),
            Self::Url => Some("url"),
            Self::Tel => Some("tel"),
            Self::Number => Some("number"),
        }
    }
}

struct InGroup;

/// A single-line text input. Inside a [`Field`] it is labelled automatically; anywhere
/// else, give it an `aria_label`.
#[component]
pub fn Input(
    #[props(default)] r#type: InputType,
    #[props(default)] size: InputSize,
    #[props(default)] variant: InputVariant,
    #[props(default)] value: Option<ReadSignal<String>>,
    #[props(default)] default_value: String,
    #[props(default)] oninput: Option<EventHandler<String>>,
    #[props(default)] invalid: bool,
    #[props(default)] readonly: bool,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes, extends = input)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(value, || default_value, oninput);
    let wiring = use_wiring(invalid);
    let grouped = try_consume_context::<std::rc::Rc<InGroup>>().is_some();
    // In a group the group draws the box, so the control drops the input styling.
    let classes = if grouped {
        ClassList::new(&GROUP.element("control"))
    } else {
        ClassList::new(INPUT.block())
            .with_modifier(INPUT, variant)
            .with_modifier(INPUT, size)
    }
    .extend(class.as_deref());
    let mut attributes = component_attributes(
        attributes,
        classes,
        &["type", "id", "aria-invalid", "aria-describedby", "value"],
    );
    attributes.extend(availability_attributes(&availability));
    rsx! {
        {stylesheets()}
        input { r#type: r#type.attribute(), id: wiring.id, aria_describedby: wiring.describedby,
            aria_invalid: wiring.invalid.then_some("true"), required: wiring.required, readonly,
            value: state.get(), oninput: move |event| state.set(event.value()), ..attributes }
    }
}

/// A multi-line text input; `autosize` grows it with its content where supported.
#[component]
pub fn Textarea(
    #[props(default)] size: InputSize,
    #[props(default)] variant: InputVariant,
    #[props(default)] autosize: bool,
    #[props(default)] value: Option<ReadSignal<String>>,
    #[props(default)] default_value: String,
    #[props(default)] oninput: Option<EventHandler<String>>,
    #[props(default)] invalid: bool,
    #[props(default)] readonly: bool,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes, extends = textarea)] attributes: Vec<Attribute>,
) -> Element {
    let state = use_controllable(value, || default_value, oninput);
    let wiring = use_wiring(invalid);
    let classes = ClassList::new(INPUT.block())
        .with_modifier(INPUT, variant)
        .with_modifier(INPUT, size)
        .with_modifier(
            INPUT,
            if autosize {
                InputGrowth::Autosize
            } else {
                InputGrowth::Fixed
            },
        )
        .extend(class.as_deref());
    let mut attributes = component_attributes(
        attributes,
        classes,
        &["id", "aria-invalid", "aria-describedby", "value"],
    );
    attributes.extend(availability_attributes(&availability));
    rsx! {
        {stylesheets()}
        textarea { id: wiring.id, aria_describedby: wiring.describedby,
            aria_invalid: wiring.invalid.then_some("true"), required: wiring.required, readonly,
            value: state.get(), oninput: move |event| state.set(event.value()), ..attributes }
    }
}

/// A native `<select>` styled as an input, for short fixed lists. Children are `option`s.
#[component]
pub fn NativeSelect(
    #[props(default)] size: InputSize,
    #[props(default)] variant: InputVariant,
    #[props(default)] onchange: Option<EventHandler<String>>,
    #[props(default)] invalid: bool,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes, extends = select)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let wiring = use_wiring(invalid);
    let classes = ClassList::new(INPUT.block())
        .with_modifier(INPUT, variant)
        .with_modifier(INPUT, size)
        .extend(class.as_deref());
    let mut attributes = component_attributes(
        attributes,
        classes,
        &["id", "aria-invalid", "aria-describedby"],
    );
    attributes.extend(availability_attributes(&availability));
    rsx! {
        {stylesheets()}
        select { id: wiring.id, aria_describedby: wiring.describedby,
            aria_invalid: wiring.invalid.then_some("true"), required: wiring.required,
            onchange: move |event| if let Some(onchange) = onchange { onchange.call(event.value()); },
            ..attributes, {children}
        }
    }
}

/// One box holding an [`Input`] with [`InputAddon`]s before or after it.
#[component]
pub fn InputGroup(
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    use_context_provider(|| std::rc::Rc::new(InGroup));
    let attributes = component_attributes(
        attributes,
        ClassList::new(GROUP.block()).extend(class.as_deref()),
        &[],
    );
    rsx! {
        {stylesheets()}
        div { ..attributes, {children} }
    }
}

/// Fixed text beside an input, such as a unit or a URL scheme.
#[component]
pub fn InputAddon(
    /// Hide a symbol whose meaning the label already carries, such as "$" for "Budget".
    #[props(default)]
    decorative: bool,
    children: Element,
) -> Element {
    rsx! { span { class: GROUP.element("addon"), aria_hidden: decorative.then_some("true"), {children} } }
}

/// One problem listed in an [`ErrorSummary`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldError {
    /// The id of the control to jump to.
    pub field: Id,
    /// What is wrong, as an instruction.
    pub message: String,
}

/// The list of problems after a submit, linking to each field. It takes focus whenever it
/// appears with errors, so keyboard and screen reader users land on it.
#[component]
pub fn ErrorSummary(
    title: Element,
    #[props(default = HeadingLevel::H2)] heading: HeadingLevel,
    errors: Vec<FieldError>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let attributes = component_attributes(
        attributes,
        ClassList::new(SUMMARY.block()).extend(class.as_deref()),
        &["tabindex"],
    );
    let focus = !errors.is_empty();
    rsx! {
        {stylesheets()}
        div { tabindex: "-1",
            onmounted: move |event| async move { if focus { let _ = event.set_focus(true).await; } },
            ..attributes,
            {super::heading(heading, SUMMARY.element("title"), title)}
            ul { class: SUMMARY.element("list"),
                for error in errors {
                    li { key: "{error.field}", a { class: SUMMARY.element("link"), href: "#{error.field}", "{error.message}" } }
                }
            }
        }
    }
}

/// A radio drawn as a card with a title and description, inside a [`CheckGroup`] given
/// `radios`.
#[component]
pub fn Choice(
    title: Element,
    #[props(default)] description: Option<Element>,
    value: String,
    #[props(default)] availability: Availability,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let choice = try_consume_context::<Signal<Option<RadioChoice>>>()
        .and_then(|context| context.read().clone());
    let name = choice.as_ref().map(|c| c.name.to_string());
    let checked = choice
        .as_ref()
        .is_some_and(|c| c.value.as_deref() == Some(value.as_str()));
    let onchange = choice.and_then(|c| availability.guard(c.onchange));
    let blocked = availability != Availability::Enabled;
    let attributes = component_attributes(
        attributes,
        ClassList::new(CHOICE.block()).extend(class.as_deref()),
        &[],
    );
    let state_attrs = availability_attributes(&availability);
    rsx! {
        {stylesheets()}
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_CHECK_CSS } }
        label { ..attributes,
            input { class: CHECK.element("input"), r#type: "radio", name, value: value.clone(), checked,
                onclick: move |event| if blocked { event.prevent_default(); },
                onchange: move |_| if let Some(onchange) = onchange { onchange.call(value.clone()); },
                ..state_attrs,
            }
            span { class: CHOICE.element("content"),
                strong { {title} }
                if let Some(description) = description { span { class: CHOICE.element("description"), {description} } }
            }
        }
    }
}
