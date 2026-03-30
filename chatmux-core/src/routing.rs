//! Routing graph compilation, edge policy evaluation, and delivery cursor management.

use chatmux_common::{
    BarrierPolicy, CatchUpPolicy, DeliveryCursor, EdgePolicy, IncrementalPolicy, Message,
    MessageId, OrchestrationMode, ProviderId, RouteEdge, RoutingGraph, StopPolicy,
    TruncationPolicy,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn compile_graph(mode: OrchestrationMode, participants: &BTreeSet<ProviderId>) -> RoutingGraph {
    let mut edges = Vec::new();
    match mode {
        OrchestrationMode::Broadcast
        | OrchestrationMode::RelayToMany
        | OrchestrationMode::Roundtable
        | OrchestrationMode::ModeratedAutonomous => {
            for source in participants {
                for target in participants {
                    if source != target {
                        edges.push(RouteEdge {
                            source: *source,
                            target: *target,
                            policy_id: None,
                        });
                    }
                }
            }
        }
        OrchestrationMode::Directed
        | OrchestrationMode::RelayToOne
        | OrchestrationMode::ModeratorJury
        | OrchestrationMode::RelayChain => {
            let ordered = participants.iter().copied().collect::<Vec<_>>();
            for pair in ordered.windows(2) {
                edges.push(RouteEdge {
                    source: pair[0],
                    target: pair[1],
                    policy_id: None,
                });
            }
        }
        OrchestrationMode::DraftOnly | OrchestrationMode::CopyOnly => {}
    }

    RoutingGraph {
        nodes: participants.clone(),
        edges,
    }
}

pub fn select_messages_for_edge(
    all_messages: &[Message],
    policy: &EdgePolicy,
    cursor: Option<&DeliveryCursor>,
) -> Vec<Message> {
    let mut selected = all_messages
        .iter()
        .filter(|message| message.participant_id == policy.source_participant_id)
        .filter(|message| {
            policy.include_user_turns || message.role != chatmux_common::MessageRole::User
        })
        .filter(|message| {
            policy.include_system_notes || message.role != chatmux_common::MessageRole::System
        })
        .cloned()
        .collect::<Vec<_>>();

    if policy.self_exclusion {
        selected.retain(|message| message.participant_id != policy.target_participant_id);
    }

    match &policy.catch_up_policy {
        CatchUpPolicy::FullHistory => {}
        CatchUpPolicy::LastN { count } => {
            let keep = selected.len().saturating_sub(*count);
            selected = selected.split_off(keep);
        }
        CatchUpPolicy::SelectedRange { start, end } => {
            selected.retain(|message| in_message_range(message.id, *start, *end, all_messages));
        }
        CatchUpPolicy::PinnedSummary { summary_message_id } => {
            selected.retain(|message| Some(message.id) == *summary_message_id);
        }
        CatchUpPolicy::None => {
            selected.clear();
        }
    }

    if let Some(cursor) = cursor {
        if let IncrementalPolicy::UnseenDeltaOnly = policy.incremental_policy {
            if let Some(last_delivered_message_id) = cursor.last_delivered_message_id {
                selected.retain(|message| message.id != last_delivered_message_id);
                if let Some(position) = selected
                    .iter()
                    .position(|message| message.id == last_delivered_message_id)
                {
                    selected = selected.split_off(position + 1);
                }
            }
        }
    }

    match &policy.incremental_policy {
        IncrementalPolicy::LastResponseOnly => {
            if let Some(last) = selected.last().cloned() {
                selected = vec![last];
            }
        }
        IncrementalPolicy::SlidingWindow { count } => {
            let keep = selected.len().saturating_sub(*count);
            selected = selected.split_off(keep);
        }
        IncrementalPolicy::ManualOnly => selected.clear(),
        IncrementalPolicy::UnseenDeltaOnly | IncrementalPolicy::FullHistoryEveryTime => {}
    }

    apply_truncation(selected, &policy.truncation_policy)
}

fn in_message_range(
    message_id: MessageId,
    start: Option<MessageId>,
    end: Option<MessageId>,
    messages: &[Message],
) -> bool {
    let positions = messages
        .iter()
        .enumerate()
        .map(|(index, message)| (message.id, index))
        .collect::<BTreeMap<_, _>>();
    let current = positions.get(&message_id).copied().unwrap_or_default();
    let start = start
        .and_then(|value| positions.get(&value).copied())
        .unwrap_or_default();
    let end = end
        .and_then(|value| positions.get(&value).copied())
        .unwrap_or(messages.len());
    current >= start && current <= end
}

fn apply_truncation(messages: Vec<Message>, policy: &TruncationPolicy) -> Vec<Message> {
    match policy {
        TruncationPolicy::None | TruncationPolicy::WarnOnly { .. } => messages,
        TruncationPolicy::TrimOldest {
            soft_character_limit,
        } => trim_messages_by_character_limit(messages, *soft_character_limit),
        TruncationPolicy::SwapForSummary {
            soft_character_limit,
            summary_message_id,
        } => {
            let mut trimmed =
                trim_messages_by_character_limit(messages.clone(), *soft_character_limit);
            if let Some(summary_message_id) = summary_message_id {
                if let Some(summary) = messages
                    .into_iter()
                    .find(|message| message.id == *summary_message_id)
                {
                    trimmed.insert(0, summary);
                }
            }
            trimmed
        }
    }
}

fn trim_messages_by_character_limit(messages: Vec<Message>, limit: usize) -> Vec<Message> {
    let mut running = 0usize;
    let mut output = Vec::new();
    for message in messages.into_iter().rev() {
        running += message.body_text.chars().count();
        if running <= limit {
            output.push(message);
        }
    }
    output.reverse();
    output
}

pub fn advance_cursor(cursor: &DeliveryCursor, delivered_messages: &[Message]) -> DeliveryCursor {
    let mut cursor = cursor.clone();
    if let Some(last_message) = delivered_messages.last() {
        cursor.last_delivered_message_id = Some(last_message.id);
        cursor.last_delivered_at = Some(last_message.timestamp);
    }
    cursor
}

pub fn barrier_satisfied(
    policy: &BarrierPolicy,
    responded: &BTreeSet<ProviderId>,
    active: &BTreeSet<ProviderId>,
) -> bool {
    match policy {
        BarrierPolicy::WaitForAll => responded == active,
        BarrierPolicy::Quorum { providers } => providers.is_subset(responded),
        BarrierPolicy::FirstFinisher => !responded.is_empty(),
        BarrierPolicy::ManualAdvance => false,
    }
}

pub fn should_stop_run(
    stop_policy: &StopPolicy,
    completed_rounds: u32,
    repeated_failures: u32,
    repeated_timeouts: u32,
    newest_message_bodies: &[String],
) -> bool {
    if stop_policy.stop_on_max_rounds
        && stop_policy
            .stagnation_window
            .is_some_and(|limit| completed_rounds >= limit)
        && stop_policy.require_approval_between_rounds
    {
        return true;
    }

    if stop_policy
        .repeated_provider_failure_limit
        .is_some_and(|limit| repeated_failures >= limit)
    {
        return true;
    }

    if stop_policy
        .repeated_timeout_limit
        .is_some_and(|limit| repeated_timeouts >= limit)
    {
        return true;
    }

    if let Some(phrase) = &stop_policy.stop_on_sentinel_phrase {
        newest_message_bodies
            .iter()
            .any(|body| body.contains(phrase))
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chatmux_common::{
        ApprovalMode, CaptureConfidence, EdgePolicy, MessageRole, TimingPolicy, WorkspaceId,
    };
    use chrono::Utc;

    fn sample_message(workspace_id: WorkspaceId, provider: ProviderId, body: &str) -> Message {
        Message {
            id: MessageId::new(),
            workspace_id,
            participant_id: provider,
            role: MessageRole::Assistant,
            round: Some(1),
            timestamp: Utc::now(),
            body_text: body.to_owned(),
            body_blocks: vec![],
            source_binding_id: None,
            dispatch_id: None,
            raw_response_text: None,
            network_capture: None,
            tags: vec![],
            capture_confidence: CaptureConfidence::Certain,
        }
    }

    fn edge_policy(workspace_id: WorkspaceId) -> EdgePolicy {
        EdgePolicy {
            id: chatmux_common::EdgePolicyId::new(),
            workspace_id,
            source_participant_id: ProviderId::Gpt,
            target_participant_id: ProviderId::Claude,
            enabled: true,
            catch_up_policy: CatchUpPolicy::FullHistory,
            incremental_policy: IncrementalPolicy::UnseenDeltaOnly,
            self_exclusion: true,
            include_user_turns: true,
            include_system_notes: true,
            include_pinned_summaries: true,
            include_moderator_annotations: true,
            include_target_prior_turns: false,
            truncation_policy: TruncationPolicy::None,
            priority: 0,
            approval_mode: ApprovalMode::AutoSend,
            template_id: None,
        }
    }

    #[test]
    fn full_mesh_graph_skips_self_edges() {
        let participants =
            BTreeSet::from([ProviderId::Gpt, ProviderId::Claude, ProviderId::Gemini]);
        let graph = compile_graph(OrchestrationMode::Roundtable, &participants);
        assert_eq!(graph.edges.len(), 6);
        assert!(graph.edges.iter().all(|edge| edge.source != edge.target));
    }

    #[test]
    fn selection_honors_source_and_self_exclusion() {
        let workspace_id = WorkspaceId::new();
        let messages = vec![
            sample_message(workspace_id, ProviderId::Gpt, "one"),
            sample_message(workspace_id, ProviderId::Claude, "two"),
            sample_message(workspace_id, ProviderId::Gpt, "three"),
        ];

        let selected = select_messages_for_edge(&messages, &edge_policy(workspace_id), None);
        assert_eq!(selected.len(), 2);
        assert!(
            selected
                .iter()
                .all(|message| message.participant_id == ProviderId::Gpt)
        );
    }

    #[test]
    fn cursor_advances_to_latest_delivered_message() {
        let workspace_id = WorkspaceId::new();
        let messages = vec![
            sample_message(workspace_id, ProviderId::Gpt, "one"),
            sample_message(workspace_id, ProviderId::Gpt, "two"),
        ];
        let cursor = DeliveryCursor {
            id: chatmux_common::DeliveryCursorId::new(),
            workspace_id,
            source_participant_id: ProviderId::Gpt,
            target_participant_id: ProviderId::Claude,
            last_delivered_message_id: None,
            last_delivered_at: None,
            frozen: false,
        };

        let advanced = advance_cursor(&cursor, &messages);
        assert_eq!(advanced.last_delivered_message_id, Some(messages[1].id));
    }

    #[test]
    fn stop_policy_detects_sentinel_phrase() {
        let stop_policy = StopPolicy {
            stop_on_max_rounds: false,
            stop_on_manual_pause: false,
            stop_on_sentinel_phrase: Some("HALT".to_owned()),
            repeated_provider_failure_limit: None,
            repeated_timeout_limit: None,
            stagnation_window: None,
            require_approval_between_rounds: false,
        };
        assert!(should_stop_run(
            &stop_policy,
            0,
            0,
            0,
            &["please HALT now".to_owned()]
        ));
    }

    #[test]
    fn timing_policy_is_available_to_routing_tests() {
        let policy = TimingPolicy::default();
        assert_eq!(policy.max_concurrent_sends, 4);
    }
}
