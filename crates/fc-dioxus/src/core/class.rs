//! Class composition using upstream's BEM vocabulary.

use std::fmt;

/// A component's single declaration of its upstream class prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bem(&'static str);

impl Bem {
    /// Declare a block that exists in upstream's CSS.
    #[must_use]
    pub const fn new(block: &'static str) -> Self {
        Self(block)
    }
    /// The block class.
    #[must_use]
    pub const fn block(self) -> &'static str {
        self.0
    }
    /// A part belonging to this block.
    #[must_use]
    pub fn element(self, element: &str) -> String {
        format!("{}__{element}", self.0)
    }
    /// A modifier with no enum of its own, for a component composing another's block.
    #[must_use]
    pub(crate) fn modifier_raw(self, suffix: &str) -> String {
        format!("{}--{suffix}", self.0)
    }
    /// The class for a typed modifier, or none for the bare block.
    #[must_use]
    pub fn modifier<M: Modifier>(self, modifier: M) -> Option<String> {
        modifier
            .suffix()
            .map(|suffix| format!("{}--{suffix}", self.0))
    }
}

/// An enum whose variants are exactly the CSS modifiers for one block.
pub trait Modifier: Copy + Eq + Default + fmt::Debug + 'static {
    /// Every variant, including the bare default.
    const ALL: &'static [Self];
    /// The upstream block to which these modifiers belong.
    const BEM: Bem;
    /// The portion after `--`, absent for the default.
    fn suffix(self) -> Option<&'static str>;
}

/// A class attribute built in one string allocation.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ClassList(String);

impl ClassList {
    /// Start with a base class.
    #[must_use]
    pub fn new(base: &str) -> Self {
        Self(base.to_owned())
    }
    /// Append whitespace-separated classes.
    #[must_use]
    pub fn with(mut self, class: impl AsRef<str>) -> Self {
        let class = class.as_ref().trim();
        if !class.is_empty() {
            if !self.0.is_empty() {
                self.0.push(' ');
            }
            self.0.push_str(class);
        }
        self
    }
    /// Append a class when a condition holds.
    #[must_use]
    pub fn with_if(self, condition: bool, class: impl AsRef<str>) -> Self {
        if condition {
            self.with(class)
        } else {
            self
        }
    }
    /// Append the non-default modifier without allocating an intermediate string.
    #[must_use]
    pub fn with_modifier<M: Modifier>(mut self, bem: Bem, modifier: M) -> Self {
        if let Some(suffix) = modifier.suffix() {
            if !self.0.is_empty() {
                self.0.push(' ');
            }
            self.0.push_str(bem.block());
            self.0.push_str("--");
            self.0.push_str(suffix);
        }
        self
    }
    /// Append consumer classes after the component's classes.
    #[must_use]
    pub fn extend(self, extra: Option<&str>) -> Self {
        match extra {
            Some(class) => self.with(class),
            None => self,
        }
    }
}

impl fmt::Display for ClassList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A string did not name a declared modifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseModifierError;
impl fmt::Display for ParseModifierError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("unknown CSS modifier")
    }
}
impl std::error::Error for ParseModifierError {}

/// Declare a modifier vocabulary once, including parsing and coverage enumeration.
///
/// The `for` argument is the component's BEM constant, so its prefix is not duplicated.
/// Empty input parses as the bare variant.
#[macro_export]
macro_rules! fc_modifier {
    (
        $(#[$meta:meta])* $vis:vis enum $name:ident for $bem:ident {
            $(#[$default_doc:meta])* $default:ident => None,
            $($(#[$variant_doc:meta])* $variant:ident => $suffix:literal),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
        #[non_exhaustive]
        $vis enum $name {
            $(#[$default_doc])*
            #[default]
            $default,
            $($(#[$variant_doc])* $variant),*
        }
        impl $crate::core::class::Modifier for $name {
            const ALL: &'static [Self] = &[Self::$default, $(Self::$variant),*];
            const BEM: $crate::core::class::Bem = $bem;
            fn suffix(self) -> Option<&'static str> {
                match self { Self::$default => None, $(Self::$variant => Some($suffix)),* }
            }
        }
        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                f.write_str($crate::core::class::Modifier::suffix(*self).unwrap_or(""))
            }
        }
        impl ::std::str::FromStr for $name {
            type Err = $crate::core::class::ParseModifierError;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value { "" => Ok(Self::$default), $($suffix => Ok(Self::$variant),)* _ => Err($crate::core::class::ParseModifierError) }
            }
        }
    };
}
