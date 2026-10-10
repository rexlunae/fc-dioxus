//! The context strip's state, ported from upstream's `context-strip.ts`.
//!
//! Provider filters model, and effort applies only to reasoning models. Both rules
//! belong to the group, not to any one select: selects that each owned their own state
//! would disagree, showing a model from one provider selected under another.

use std::collections::HashMap;

/// One model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelChoice {
    /// Stable id.
    pub id: String,
    /// What people read.
    pub name: String,
    /// Reasoning models accept an effort level; others do not.
    pub reasoning: bool,
    /// Read-only facts shown as chips, e.g. ("context", "200k"), in order.
    pub parameters: Vec<(String, String)>,
}

/// One provider and the models it offers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderChoice {
    /// Stable id.
    pub id: String,
    /// What people read.
    pub name: String,
    /// Its models, in the order offered.
    pub models: Vec<ModelChoice>,
}

/// An effort level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Effort {
    /// Low.
    Low,
    /// Medium, the default when a reasoning model is chosen.
    Medium,
    /// High.
    High,
}
impl Effort {
    /// All levels, lowest first.
    pub const ALL: [Self; 3] = [Self::Low, Self::Medium, Self::High];
    /// The value written to `data-fc-effort`.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
    /// What people read.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }
    /// Parse a value written by [`Effort::as_str`].
    #[must_use]
    pub fn from_value(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.as_str() == value)
    }
}

/// The resolved state of the whole strip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextState {
    /// Provider id.
    pub provider: String,
    /// Model id.
    pub model: String,
    /// `None` when the current model has no effort control.
    pub effort: Option<Effort>,
    /// The current model's parameters.
    pub parameters: Vec<(String, String)>,
}

/// The strip's model.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ContextModel {
    providers: Vec<ProviderChoice>,
    provider: String,
    model: String,
    effort: Option<Effort>,
    /// The model last chosen under each provider, so switching back restores it.
    remembered: HashMap<String, String>,
}

impl ContextModel {
    /// Start on the first provider, so the strip is never half-set.
    #[must_use]
    pub fn new(providers: Vec<ProviderChoice>) -> Self {
        let first = providers.first().map(|p| p.id.clone()).unwrap_or_default();
        let mut model = Self {
            providers,
            ..Self::default()
        };
        model.set_provider(&first);
        model
    }
    /// The providers.
    #[must_use]
    pub fn providers(&self) -> &[ProviderChoice] {
        &self.providers
    }
    /// The current provider.
    #[must_use]
    pub fn provider(&self) -> Option<&ProviderChoice> {
        self.providers.iter().find(|p| p.id == self.provider)
    }
    /// The current model.
    #[must_use]
    pub fn model(&self) -> Option<&ModelChoice> {
        self.provider()?.models.iter().find(|m| m.id == self.model)
    }
    /// The effort the user set, even when the current model ignores it.
    #[must_use]
    pub fn stored_effort(&self) -> Option<Effort> {
        self.effort
    }
    /// The resolved state.
    #[must_use]
    pub fn state(&self) -> ContextState {
        let model = self.model();
        ContextState {
            provider: self.provider.clone(),
            model: self.model.clone(),
            effort: model.filter(|m| m.reasoning).and(self.effort),
            parameters: model.map(|m| m.parameters.clone()).unwrap_or_default(),
        }
    }
    /// Choose a provider and resolve the model against it: the model this provider had
    /// last time, else the one just chosen if it is offered here too, else its first.
    pub fn set_provider(&mut self, provider: &str) {
        if !self.providers.iter().any(|p| p.id == provider) {
            return;
        }
        if !self.provider.is_empty() {
            self.remembered
                .insert(self.provider.clone(), self.model.clone());
        }
        let previous = std::mem::take(&mut self.model);
        provider.clone_into(&mut self.provider);
        let available: Vec<String> = self
            .provider()
            .map(|p| p.models.iter().map(|m| m.id.clone()).collect())
            .unwrap_or_default();
        let candidates = [self.remembered.get(provider).cloned(), Some(previous)];
        self.model = candidates
            .into_iter()
            .flatten()
            .find(|id| !id.is_empty() && available.contains(id))
            .or_else(|| available.first().cloned())
            .unwrap_or_default();
        self.default_effort();
    }
    /// Choose a model the current provider offers.
    pub fn set_model(&mut self, model: &str) {
        if self
            .provider()
            .is_some_and(|p| p.models.iter().any(|m| m.id == model))
        {
            model.clone_into(&mut self.model);
            self.default_effort();
        }
    }
    /// Set the effort. Ignored for a model with no effort control.
    pub fn set_effort(&mut self, effort: Effort) {
        if self.model().is_some_and(|m| m.reasoning) {
            self.effort = Some(effort);
        }
    }
    /// Medium when a reasoning model is chosen and nothing is set yet, so switching
    /// between reasoning models keeps the user's choice.
    fn default_effort(&mut self) {
        if self.model().is_some_and(|m| m.reasoning) && self.effort.is_none() {
            self.effort = Some(Effort::Medium);
        }
    }
    /// What to say when the effort control does not apply, or `None` when it does.
    #[must_use]
    pub fn notice(&self) -> Option<&'static str> {
        if self.model().is_some_and(|m| m.reasoning) {
            None
        } else if self.effort.is_some() {
            Some("Effort is not used by this model")
        } else {
            Some("This model has no effort setting")
        }
    }
}
