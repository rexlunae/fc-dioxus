//! Modal and non-modal dialogs, side sheets, and bottom sheets, from dialog.css.
use crate::{
    core::{
        assets,
        attrs::{component_attributes, FcData, Kind},
        platform::{DomBridge, DomError},
    },
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-dialog");
fc_modifier! {
    /// Dialog widths.
    pub enum DialogSize for BEM {
        /// Default.
        Md => None,
        /// Narrow, for a short confirmation.
        Sm => "sm",
        /// Wide, for forms and settings.
        Lg => "lg",
        /// The whole viewport.
        Fullscreen => "fullscreen",
    }
}
fc_modifier! {
    /// Where a dialog sits.
    pub enum DialogPlacement for BEM {
        /// Centred.
        Center => None,
        /// Docked to the inline end, full height.
        Sheet => "sheet",
        /// Docked to the block end.
        Bottom => "bottom",
    }
}

/// A native `<dialog>`, controlled by `open`.
///
/// Modal by default: when `open` turns true it is shown with `showModal()` after mount, so
/// the platform supplies the top layer, inert background, focus, and Escape. Escape and the
/// close button call `onclose`; the owner sets `open` to false. Hosts without
/// `showModal` render it open in place instead. With `modal: false` it is always the
/// in-place, non-modal form.
#[component]
pub fn Dialog(
    open: bool,
    onclose: EventHandler<()>,
    title: Element,
    #[props(default = HeadingLevel::H2)] heading: HeadingLevel,
    #[props(default)] description: Option<Element>,
    #[props(default)] size: DialogSize,
    #[props(default)] placement: DialogPlacement,
    #[props(default = true)] modal: bool,
    /// Names the close button; "Close" when omitted.
    #[props(default)]
    close_label: Option<AccessibleName>,
    #[props(default)] footer: Option<Element>,
    /// The drag handle drawn on a bottom sheet. Decorative.
    #[props(default)]
    handle: bool,
    /// Make a body that may scroll reachable by keyboard.
    #[props(default)]
    scrollable: bool,
    /// The runtime id for `data-fc-id`; it also marks the dialog `data-fc-kind="dialog"`.
    #[props(default)]
    fc_id: Option<Id>,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let id = use_id();
    let title_id = id.suffixed("title");
    let description_id = description.is_some().then(|| id.suffixed("description"));
    let mut inline = use_signal(|| !modal);
    let effect_id = id.clone();
    use_effect(use_reactive!(|open, modal| {
        let id = effect_id.clone();
        spawn(async move {
            if !modal {
                inline.set(true);
                return;
            }
            let result = if open {
                DomBridge.show_modal(&id).await
            } else {
                DomBridge.close_dialog(&id).await
            };
            // Without showModal the dialog still has to appear: open it in place.
            inline.set(matches!(result, Err(DomError::Unsupported)));
        });
    }));
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(BEM.block())
            .with_modifier(BEM, size)
            .with_modifier(BEM, placement)
            .extend(class.as_deref()),
        &["id", "open", "aria-labelledby", "aria-describedby"],
    );
    if let Some(fc_id) = fc_id {
        attributes.extend(
            FcData {
                kind: Some(Kind::Dialog),
                id: Some(fc_id),
                ..FcData::default()
            }
            .attributes(),
        );
    }
    let close_label = close_label.map_or_else(|| "Close".to_owned(), |l| l.to_string());
    rsx! {
        if use_stylesheets() {
            document::Stylesheet { href: assets::COMPONENTS_BUTTON_CSS }
            document::Stylesheet { href: assets::COMPONENTS_DIALOG_CSS }
        }
        dialog { id: "{id}", open: (open && inline()).then_some(true), aria_labelledby: "{title_id}",
            aria_describedby: description_id.as_ref().map(ToString::to_string),
            oncancel: move |event| { event.prevent_default(); onclose.call(()); },
            ..attributes,
            if handle { div { class: BEM.element("handle"), aria_hidden: "true" } }
            header { class: BEM.element("header"),
                {super::heading_with_id(heading, BEM.element("title"), &title_id, title)}
                button { r#type: "button", class: BEM.element("close"), aria_label: close_label, onclick: move |_| onclose.call(()), "✕" }
            }
            div { class: BEM.element("body"), tabindex: scrollable.then_some("0"),
                if let (Some(description), Some(id)) = (description, description_id.clone()) {
                    p { class: BEM.element("description"), id: "{id}", {description} }
                }
                {children}
            }
            if let Some(footer) = footer { footer { class: BEM.element("footer"), {footer} } }
        }
    }
}
