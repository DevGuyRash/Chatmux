//! Truthful persisted dispatch lifecycle transitions.

use crate::storage::StorageError;
use chatmux_common::{ApprovalMode, Dispatch, DispatchOutcome};
use chrono::{DateTime, Utc};

/// Purpose: Resolve the initial persisted outcome for an outbound package.
///
/// # Arguments
/// * `approval_mode` - User-selected approval behavior for the package.
///
/// # Returns
/// `Pending` only for automatic sends; non-I/O modes resolve to `Skipped`.
///
/// # Panics
/// None.
///
/// # Examples
/// ```
/// use chatmux_common::{ApprovalMode, DispatchOutcome};
/// use chatmux_core::dispatch::prepared_outcome;
///
/// assert_eq!(prepared_outcome(ApprovalMode::AutoSend), DispatchOutcome::Pending);
/// ```
pub fn prepared_outcome(approval_mode: ApprovalMode) -> DispatchOutcome {
    match approval_mode {
        ApprovalMode::AutoSend => DispatchOutcome::Pending,
        ApprovalMode::RequireUserConfirmation
        | ApprovalMode::DraftOnly
        | ApprovalMode::CopyOnly
        | ApprovalMode::ManualSend => DispatchOutcome::Skipped,
    }
}

/// Purpose: Transition a pending dispatch to delivered after browser I/O succeeds.
///
/// # Arguments
/// * `dispatch` - Persisted dispatch to transition.
/// * `delivered_at` - Browser-I/O acknowledgement time.
///
/// # Returns
/// The transitioned dispatch, or a storage invariant error for an illegal transition.
///
/// # Panics
/// None.
///
/// # Examples
/// Call this through `BackgroundCoordinator` so the transition and cursor advancement are
/// persisted together by the orchestration layer.
pub fn mark_delivered(
    mut dispatch: Dispatch,
    delivered_at: DateTime<Utc>,
) -> Result<Dispatch, StorageError> {
    match dispatch.outcome {
        DispatchOutcome::Pending => {
            dispatch.outcome = DispatchOutcome::Delivered;
            dispatch.sent_at = Some(delivered_at);
            dispatch.error_detail = None;
            Ok(dispatch)
        }
        DispatchOutcome::Delivered => Ok(dispatch),
        DispatchOutcome::Timeout | DispatchOutcome::Error | DispatchOutcome::Skipped => {
            Err(invalid_transition(dispatch.outcome, "delivered"))
        }
    }
}

fn invalid_transition(outcome: DispatchOutcome, target: &str) -> StorageError {
    let current = match outcome {
        DispatchOutcome::Pending => "pending",
        DispatchOutcome::Delivered => "delivered",
        DispatchOutcome::Timeout => "timeout",
        DispatchOutcome::Error => "error",
        DispatchOutcome::Skipped => "skipped",
    };
    StorageError::Invariant(format!(
        "dispatch transition failed: {current} dispatch cannot transition to {target}; acknowledge only pending dispatches"
    ))
}

/// Purpose: Transition a pending dispatch to an error without advancing delivery cursors.
///
/// # Arguments
/// * `dispatch` - Persisted dispatch to transition.
/// * `detail` - Actionable provider-I/O failure detail.
///
/// # Returns
/// The transitioned dispatch, or a storage invariant error for an illegal transition.
///
/// # Panics
/// None.
///
/// # Examples
/// Call this through `BackgroundCoordinator` after adapter injection or send fails.
pub fn mark_failed(mut dispatch: Dispatch, detail: &str) -> Result<Dispatch, StorageError> {
    let detail = detail.trim();
    if detail.is_empty() {
        return Err(StorageError::Invariant(
            "dispatch failure acknowledgement failed: failure detail is empty; provide an actionable provider-I/O error"
                .to_owned(),
        ));
    }
    if dispatch.outcome == DispatchOutcome::Error
        && dispatch.error_detail.as_deref() == Some(detail)
    {
        return Ok(dispatch);
    }
    if !matches!(
        dispatch.outcome,
        DispatchOutcome::Pending | DispatchOutcome::Delivered
    ) {
        return Err(invalid_transition(dispatch.outcome, "error"));
    }

    dispatch.outcome = DispatchOutcome::Error;
    dispatch.error_detail = Some(detail.to_owned());
    dispatch.retry_count = dispatch.retry_count.saturating_add(1);
    Ok(dispatch)
}

/// Purpose: Record capture completion for an already-delivered dispatch.
///
/// # Arguments
/// * `dispatch` - Persisted delivered dispatch.
/// * `captured_at` - Response-capture acknowledgement time.
///
/// # Returns
/// The updated dispatch, or a storage invariant error if delivery was never acknowledged.
///
/// # Panics
/// None.
///
/// # Examples
/// Call this through `BackgroundCoordinator` while attaching captured messages.
pub fn mark_captured(
    mut dispatch: Dispatch,
    captured_at: DateTime<Utc>,
) -> Result<Dispatch, StorageError> {
    if dispatch.outcome != DispatchOutcome::Delivered {
        return Err(invalid_transition(dispatch.outcome, "captured"));
    }

    if dispatch.captured_at.is_none() {
        dispatch.captured_at = Some(captured_at);
    }
    Ok(dispatch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dispatch(outcome: DispatchOutcome) -> Dispatch {
        Dispatch {
            id: chatmux_common::DispatchId::new(),
            run_id: chatmux_common::RunId::new(),
            round_id: None,
            round_number: 1,
            target_participant_id: chatmux_common::ProviderId::Gpt,
            source_message_ids: Vec::new(),
            template_id: None,
            rendered_payload: "exact payload".to_owned(),
            sent_at: None,
            captured_at: None,
            outcome,
            error_detail: None,
            retry_count: 0,
        }
    }

    #[test]
    fn auto_send_is_prepared_as_pending() {
        assert_eq!(
            prepared_outcome(ApprovalMode::AutoSend),
            DispatchOutcome::Pending
        );
    }

    #[test]
    fn non_io_approval_modes_are_never_prepared_as_delivered() {
        for approval_mode in [
            ApprovalMode::RequireUserConfirmation,
            ApprovalMode::DraftOnly,
            ApprovalMode::CopyOnly,
            ApprovalMode::ManualSend,
        ] {
            assert_eq!(prepared_outcome(approval_mode), DispatchOutcome::Skipped);
        }
    }

    #[test]
    fn skipped_dispatch_cannot_be_marked_delivered() {
        let result = mark_delivered(dispatch(DispatchOutcome::Skipped), DateTime::UNIX_EPOCH);

        assert!(matches!(result, Err(StorageError::Invariant(detail)) if
            detail.contains("skipped dispatch cannot transition to delivered")));
    }

    #[test]
    fn pending_dispatch_can_be_marked_failed_without_send_timestamp() {
        let failed = mark_failed(dispatch(DispatchOutcome::Pending), "composer missing");

        assert!(matches!(failed, Ok(ref item) if
            item.outcome == DispatchOutcome::Error
                && item.sent_at.is_none()
                && item.error_detail.as_deref() == Some("composer missing")
                && item.retry_count == 1));
    }

    #[test]
    fn failure_ack_requires_actionable_detail() {
        let result = mark_failed(dispatch(DispatchOutcome::Pending), "   ");

        assert!(matches!(result, Err(StorageError::Invariant(detail)) if
            detail.contains("failure detail is empty")));
    }

    #[test]
    fn delivered_dispatch_can_record_capture_completion() {
        let captured_at = DateTime::UNIX_EPOCH;
        let mut delivered = dispatch(DispatchOutcome::Delivered);
        delivered.sent_at = Some(captured_at);

        let captured = mark_captured(delivered, captured_at);

        assert!(matches!(captured, Ok(ref item) if
            item.outcome == DispatchOutcome::Delivered
                && item.captured_at == Some(captured_at)));
    }
}
