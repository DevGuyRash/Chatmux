//! User-visible command outcomes and observed dispatch state.

use std::collections::BTreeMap;

use crate::models::{Dispatch, DispatchId, DispatchOutcome, UiEvent};

/// Severity used when presenting the result of an explicit user action.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandOutcomeKind {
    /// The requested action completed successfully.
    Success,
    /// The requested action completed with a non-fatal warning.
    Warning,
    /// The requested action failed or was rejected.
    Error,
}

/// A sequenced, accessible result for the most recent explicit user action.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandOutcome {
    /// Monotonic identifier used to announce repeated messages independently.
    pub sequence: u64,
    /// Presentation severity.
    pub kind: CommandOutcomeKind,
    /// Concise human-readable outcome.
    pub message: String,
}

/// Result of evaluating command events for confirmed completion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandConfirmation {
    /// The bridge accepted the action and returned no failing dispatch.
    Confirmed,
    /// A dispatch reached a terminal failure state.
    Rejected(String),
}

/// Evaluate returned events for terminal dispatch failures.
pub fn command_confirmation(events: &[UiEvent]) -> CommandConfirmation {
    events
        .iter()
        .filter_map(|event| match event {
            UiEvent::DispatchUpdated { dispatch } => Some(dispatch),
            _ => None,
        })
        .find_map(|dispatch| match dispatch.outcome {
            DispatchOutcome::Error => Some(CommandConfirmation::Rejected(
                dispatch
                    .error_detail
                    .clone()
                    .unwrap_or_else(|| "The provider rejected the message.".to_owned()),
            )),
            DispatchOutcome::Timeout => Some(CommandConfirmation::Rejected(
                dispatch.error_detail.clone().unwrap_or_else(|| {
                    "The provider timed out before confirming delivery.".to_owned()
                }),
            )),
            DispatchOutcome::Pending | DispatchOutcome::Delivered | DispatchOutcome::Skipped => {
                None
            }
        })
        .unwrap_or(CommandConfirmation::Confirmed)
}

/// Insert every observed dispatch event into the UI dispatch registry.
pub fn collect_dispatches(registry: &mut BTreeMap<DispatchId, Dispatch>, events: &[UiEvent]) {
    for event in events {
        if let UiEvent::DispatchUpdated { dispatch } = event {
            registry.insert(dispatch.id, dispatch.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DispatchOutcome, ProviderId, RunId};

    fn dispatch(outcome: DispatchOutcome, error_detail: Option<&str>) -> Dispatch {
        Dispatch {
            id: DispatchId::new(),
            run_id: RunId::new(),
            round_id: None,
            round_number: 0,
            target_participant_id: ProviderId::Gpt,
            source_message_ids: Vec::new(),
            template_id: None,
            rendered_payload: "payload".to_owned(),
            sent_at: None,
            captured_at: None,
            outcome,
            error_detail: error_detail.map(str::to_owned),
            retry_count: 0,
        }
    }

    #[test]
    fn error_dispatch_rejects_command_with_specific_detail() {
        let events = vec![UiEvent::DispatchUpdated {
            dispatch: dispatch(DispatchOutcome::Error, Some("Provider rejected send")),
        }];

        assert_eq!(
            command_confirmation(&events),
            CommandConfirmation::Rejected("Provider rejected send".to_owned())
        );
    }

    #[test]
    fn timeout_dispatch_rejects_command_with_fallback_detail() {
        let events = vec![UiEvent::DispatchUpdated {
            dispatch: dispatch(DispatchOutcome::Timeout, None),
        }];

        assert_eq!(
            command_confirmation(&events),
            CommandConfirmation::Rejected(
                "The provider timed out before confirming delivery.".to_owned()
            )
        );
    }

    #[test]
    fn delivered_and_skipped_dispatches_are_confirmed() {
        for outcome in [
            DispatchOutcome::Pending,
            DispatchOutcome::Delivered,
            DispatchOutcome::Skipped,
        ] {
            let events = vec![UiEvent::DispatchUpdated {
                dispatch: dispatch(outcome, None),
            }];
            assert_eq!(
                command_confirmation(&events),
                CommandConfirmation::Confirmed
            );
        }
    }

    #[test]
    fn dispatch_events_are_retained_by_identifier() {
        let expected = dispatch(DispatchOutcome::Delivered, None);
        let events = vec![UiEvent::DispatchUpdated {
            dispatch: expected.clone(),
        }];
        let mut registry = BTreeMap::new();

        collect_dispatches(&mut registry, &events);

        assert_eq!(registry.get(&expected.id), Some(&expected));
    }
}
