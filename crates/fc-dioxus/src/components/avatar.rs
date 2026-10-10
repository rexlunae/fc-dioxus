//! Initials or a picture standing in for a person, from avatar.css.
use crate::{
    core::{assets, attrs::component_attributes},
    prelude::*,
    theme::use_stylesheets,
};
const BEM: Bem = Bem::new("fc-avatar");
const GROUP: Bem = Bem::new("fc-avatar-group");
fc_modifier! {
    /// Avatar sizes.
    pub enum AvatarSize for BEM {
        /// The default 2rem.
        Md => None,
        /// Extra small; shows a single initial.
        Xs => "xs",
        /// Small.
        Sm => "sm",
        /// Large.
        Lg => "lg",
        /// Extra large.
        Xl => "xl",
    }
}
fc_modifier! { enum AvatarShape for BEM { Round => None, Square => "square" } }
fc_modifier! { enum AvatarTone for BEM { Accent => None, Neutral => "neutral" } }

/// Initials from the first and last words, at most `count` of them.
///
/// Splits on characters, never bytes, so a name in any script cannot be cut mid-character.
/// A combining mark that follows the first character is dropped; the initial stays legible.
#[must_use]
pub fn initials(name: &str, count: usize) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    let first = words.first().and_then(|w| w.chars().next());
    let last = if words.len() > 1 {
        words.last().and_then(|w| w.chars().next())
    } else {
        None
    };
    [first, last]
        .into_iter()
        .flatten()
        .take(count)
        .flat_map(char::to_uppercase)
        .collect()
}

/// A person's avatar.
///
/// `alt` defaults to the name: the avatar is announced as that person. Pass
/// `Alt::Decorative` when the name is already visible beside it, as in a message header,
/// so it is not read twice.
#[component]
pub fn Avatar(
    name: String,
    #[props(default)] src: Option<String>,
    #[props(default)] alt: Option<Alt>,
    /// Replaces the derived initials, e.g. a symbol for a bot.
    #[props(default)]
    initials: Option<String>,
    #[props(default)] size: AvatarSize,
    #[props(default)] square: bool,
    #[props(default)] neutral: bool,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
) -> Element {
    let classes = ClassList::new(BEM.block())
        .with_modifier(BEM, size)
        .with_modifier(
            BEM,
            if square {
                AvatarShape::Square
            } else {
                AvatarShape::Round
            },
        )
        .with_modifier(
            BEM,
            if neutral {
                AvatarTone::Neutral
            } else {
                AvatarTone::Accent
            },
        )
        .extend(class.as_deref());
    let mut attributes =
        component_attributes(attributes, classes, &["role", "aria-label", "aria-hidden"]);
    let label = match alt {
        Some(Alt::Text(name)) => Some(name.to_string()),
        Some(Alt::Decorative) => None,
        None => AccessibleName::try_from(name.as_str())
            .ok()
            .map(|n| n.to_string()),
    };
    let text = initials
        .unwrap_or_else(|| self::initials(&name, if size == AvatarSize::Xs { 1 } else { 2 }));
    match (&src, &label) {
        // The image carries the name itself; the wrapper adds nothing.
        (Some(_), _) => {}
        (None, Some(label)) => {
            attributes.push(Attribute::new("role", "img", None, false));
            attributes.push(Attribute::new("aria-label", label.clone(), None, false));
        }
        (None, None) => attributes.push(Attribute::new("aria-hidden", "true", None, false)),
    }
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_AVATAR_CSS } }
        span { ..attributes,
            if let Some(src) = src {
                img { class: BEM.element("image"), src, alt: label.unwrap_or_default() }
            } else {
                "{text}"
            }
        }
    }
}

/// Overlapping avatars, with an optional "+n" for the people not shown.
///
/// The "+n" is decoration, as in upstream's reference markup: say who and how many in
/// visible text beside the group, or in its `label`.
#[component]
pub fn AvatarGroup(
    /// Names the group, e.g. "Reviewers". Without one, the group is presentational.
    #[props(default)]
    label: Option<AccessibleName>,
    /// How many more people there are than avatars shown.
    #[props(default)]
    overflow: u32,
    /// Size of the overflow avatar, to match its siblings.
    #[props(default)]
    size: AvatarSize,
    #[props(default)] class: Option<String>,
    #[props(extends = GlobalAttributes)] attributes: Vec<Attribute>,
    children: Element,
) -> Element {
    let mut attributes = component_attributes(
        attributes,
        ClassList::new(GROUP.block()).extend(class.as_deref()),
        &["role", "aria-label"],
    );
    if let Some(label) = label {
        attributes.push(Attribute::new("role", "group", None, false));
        attributes.push(Attribute::new("aria-label", label.to_string(), None, false));
    }
    let more = format!("{overflow} more");
    rsx! {
        if use_stylesheets() { document::Stylesheet { href: assets::COMPONENTS_AVATAR_CSS } }
        span { ..attributes,
            {children}
            if overflow > 0 {
                Avatar { name: more, alt: Alt::Decorative, initials: format!("+{overflow}"), size, neutral: true }
            }
        }
    }
}
