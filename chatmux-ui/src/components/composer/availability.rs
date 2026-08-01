//! Pure composer target availability and selection rules.

use crate::components::provider::Provider;

/// Why a provider can or cannot receive the current composer submission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetAvailability {
    /// The provider is bound, healthy, permitted, and capable of sending.
    Available,
    /// No browser tab is bound to the provider.
    Unbound,
    /// The binding is stale and must be refreshed.
    StaleBinding,
    /// The tab no longer matches the conversation that was bound.
    ConversationChanged,
    /// Host permission is unavailable.
    PermissionMissing,
    /// The provider is not currently healthy enough to accept a message.
    Unhealthy,
    /// The provider adapter cannot safely auto-send.
    Unsupported,
    /// The provider is not enabled for this workspace.
    WorkspaceDisabled,
}

impl TargetAvailability {
    /// Whether the target may be selected and submitted.
    pub fn is_available(self) -> bool {
        matches!(self, Self::Available)
    }

    /// Short explanation suitable for a disabled-target tooltip.
    pub fn reason(self) -> &'static str {
        match self {
            Self::Available => "Ready to receive this message.",
            Self::Unbound => "Bind a provider tab first.",
            Self::StaleBinding => "Refresh this stale provider binding.",
            Self::ConversationChanged => "Refresh the binding to confirm this conversation.",
            Self::PermissionMissing => "Grant host permission for this provider.",
            Self::Unhealthy => "Wait for the provider to become ready.",
            Self::Unsupported => "This provider cannot safely auto-send.",
            Self::WorkspaceDisabled => "Enable this provider for the workspace first.",
        }
    }
}

/// A provider together with its trustworthy send availability.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ComposerTarget {
    /// Provider represented by the target chip.
    pub provider: Provider,
    /// Current reason the provider is available or blocked.
    pub availability: TargetAvailability,
}

/// Reconcile a selected provider list against currently available targets.
pub fn reconcile_selected_targets(
    selected: &[Provider],
    targets: &[ComposerTarget],
    initialize: bool,
) -> Vec<Provider> {
    targets
        .iter()
        .filter(|target| target.availability.is_available())
        .filter(|target| initialize || selected.contains(&target.provider))
        .map(|target| target.provider)
        .collect()
}

/// Determine whether the current composer state is safe to submit.
pub fn can_submit(
    text: &str,
    selected: &[Provider],
    targets: &[ComposerTarget],
    kill_switch_active: bool,
    command_in_flight: bool,
) -> bool {
    !text.trim().is_empty()
        && !selected.is_empty()
        && !kill_switch_active
        && !command_in_flight
        && selected.iter().all(|provider| {
            targets
                .iter()
                .any(|target| target.provider == *provider && target.availability.is_available())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(provider: Provider, availability: TargetAvailability) -> ComposerTarget {
        ComposerTarget {
            provider,
            availability,
        }
    }

    #[test]
    fn initial_selection_contains_only_available_targets() {
        let targets = [
            target(Provider::Gpt, TargetAvailability::Available),
            target(Provider::Gemini, TargetAvailability::Unbound),
            target(Provider::Grok, TargetAvailability::PermissionMissing),
            target(Provider::Claude, TargetAvailability::Available),
        ];

        assert_eq!(
            reconcile_selected_targets(&[], &targets, true),
            vec![Provider::Gpt, Provider::Claude]
        );
    }

    #[test]
    fn reconciliation_drops_blocked_targets_without_auto_selecting_new_targets() {
        let targets = [
            target(Provider::Gpt, TargetAvailability::Unhealthy),
            target(Provider::Gemini, TargetAvailability::Available),
            target(Provider::Claude, TargetAvailability::Available),
        ];

        assert_eq!(
            reconcile_selected_targets(&[Provider::Gpt, Provider::Gemini], &targets, false,),
            vec![Provider::Gemini]
        );
    }

    #[test]
    fn submission_requires_text_selected_available_targets_and_safe_global_state() {
        let targets = [target(Provider::Gpt, TargetAvailability::Available)];
        let selected = [Provider::Gpt];

        assert!(can_submit("hello", &selected, &targets, false, false));
        assert!(!can_submit("   ", &selected, &targets, false, false));
        assert!(!can_submit("hello", &[], &targets, false, false));
        assert!(!can_submit("hello", &selected, &targets, true, false));
        assert!(!can_submit("hello", &selected, &targets, false, true));
        assert!(!can_submit(
            "hello",
            &selected,
            &[target(Provider::Gpt, TargetAvailability::StaleBinding)],
            false,
            false,
        ));
    }

    #[test]
    fn disabled_reasons_are_specific_and_actionable() {
        assert_eq!(
            TargetAvailability::Unbound.reason(),
            "Bind a provider tab first."
        );
        assert_eq!(
            TargetAvailability::ConversationChanged.reason(),
            "Refresh the binding to confirm this conversation."
        );
        assert_eq!(
            TargetAvailability::Unsupported.reason(),
            "This provider cannot safely auto-send."
        );
    }
}
