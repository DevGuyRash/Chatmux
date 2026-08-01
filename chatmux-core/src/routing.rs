//! Routing graph compilation, edge policy evaluation, and delivery cursor management.

use chatmux_common::{
    BarrierPolicy, CatchUpPolicy, DeliveryCursor, EdgePolicy, IncrementalPolicy, Message,
    MessageId, OrchestrationMode, ProviderId, RouteEdge, RoutingGraph, RunConfiguration,
    StopPolicy, TimingPolicy, TruncationPolicy,
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

/// Compile an explicit run topology, including user-seed edges for autonomous modes.
pub fn compile_configured_graph(configuration: &RunConfiguration) -> RoutingGraph {
    let participants = &configuration.participants;
    let mut nodes = participants.clone();
    nodes.insert(ProviderId::User);
    let mut edges = Vec::new();
    let mut push_edge = |source, target| {
        if source != target
            && !edges
                .iter()
                .any(|edge: &RouteEdge| edge.source == source && edge.target == target)
        {
            edges.push(RouteEdge {
                source,
                target,
                policy_id: None,
            });
        }
    };

    match configuration.mode {
        OrchestrationMode::Broadcast | OrchestrationMode::RelayToMany => {
            for target in participants {
                push_edge(ProviderId::User, *target);
            }
        }
        OrchestrationMode::Directed | OrchestrationMode::RelayToOne => {
            let target = configuration
                .moderator
                .or_else(|| configuration.relay_order.last().copied())
                .or_else(|| participants.iter().next_back().copied());
            if let Some(target) = target {
                push_edge(ProviderId::User, target);
                for source in participants {
                    push_edge(*source, target);
                }
            }
        }
        OrchestrationMode::Roundtable | OrchestrationMode::ModeratedAutonomous => {
            for target in participants {
                push_edge(ProviderId::User, *target);
                for source in participants {
                    push_edge(*source, *target);
                }
            }
        }
        OrchestrationMode::ModeratorJury => {
            if let Some(moderator) = configuration
                .moderator
                .or_else(|| participants.iter().next_back().copied())
            {
                push_edge(ProviderId::User, moderator);
                for participant in participants {
                    push_edge(ProviderId::User, *participant);
                    push_edge(*participant, moderator);
                    push_edge(moderator, *participant);
                }
            }
        }
        OrchestrationMode::RelayChain => {
            let order = if configuration.relay_order.is_empty() {
                participants.iter().copied().collect::<Vec<_>>()
            } else {
                configuration.relay_order.clone()
            };
            if let Some(first) = order.first() {
                push_edge(ProviderId::User, *first);
            }
            for pair in order.windows(2) {
                push_edge(pair[0], pair[1]);
            }
        }
        OrchestrationMode::DraftOnly | OrchestrationMode::CopyOnly => {}
    }

    RoutingGraph { nodes, edges }
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

    let last_delivered_message_id = cursor.and_then(|item| item.last_delivered_message_id);
    if let Some(last_delivered_message_id) = last_delivered_message_id {
        match &policy.incremental_policy {
            IncrementalPolicy::UnseenDeltaOnly => {
                if let Some(position) = selected
                    .iter()
                    .position(|message| message.id == last_delivered_message_id)
                {
                    selected = selected.split_off(position + 1);
                } else {
                    selected.clear();
                }
            }
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
            IncrementalPolicy::FullHistoryEveryTime => {}
        }
    } else {
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
            CatchUpPolicy::None => selected.clear(),
        }
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
            if let Some(summary_message_id) = summary_message_id
                && let Some(summary) = messages
                    .into_iter()
                    .find(|message| message.id == *summary_message_id)
            {
                trimmed.insert(0, summary);
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
    if cursor.frozen {
        return cursor.clone();
    }

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
        BarrierPolicy::WaitForAll => active.is_subset(responded),
        BarrierPolicy::Quorum { providers } => providers.is_subset(responded),
        BarrierPolicy::FirstFinisher => !responded.is_empty(),
        BarrierPolicy::ManualAdvance => false,
    }
}

pub fn should_stop_run(
    timing_policy: &TimingPolicy,
    stop_policy: &StopPolicy,
    completed_rounds: u32,
    repeated_failures: u32,
    repeated_timeouts: u32,
    recent_round_message_bodies: &[Vec<String>],
) -> bool {
    if stop_policy.stop_on_max_rounds
        && timing_policy
            .max_rounds
            .is_some_and(|limit| completed_rounds >= limit)
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

    if let Some(phrase) = &stop_policy.stop_on_sentinel_phrase
        && recent_round_message_bodies
            .iter()
            .flatten()
            .any(|body| body.contains(phrase))
    {
        return true;
    }

    stop_policy
        .stagnation_window
        .is_some_and(|window| convergence_stagnated(recent_round_message_bodies, window as usize))
}

/// Detect repeated low-delta rounds without requiring semantic model access.
///
/// Each round is reduced to a normalized word set. A run is considered stagnant when every
/// adjacent pair in the configured window has Jaccard similarity of at least 0.92. This keeps
/// the heuristic deterministic, local-only, and insensitive to punctuation or provider order.
pub fn convergence_stagnated(recent_rounds: &[Vec<String>], window: usize) -> bool {
    if window < 2 || recent_rounds.len() < window {
        return false;
    }

    let start = recent_rounds.len() - window;
    let fingerprints = recent_rounds[start..]
        .iter()
        .map(|round| {
            round
                .iter()
                .flat_map(|body| body.split(|ch: char| !ch.is_alphanumeric()))
                .map(str::to_lowercase)
                .filter(|word| word.len() > 2)
                .collect::<BTreeSet<_>>()
        })
        .collect::<Vec<_>>();

    fingerprints.windows(2).all(|pair| {
        let union = pair[0].union(&pair[1]).count();
        if union == 0 {
            return false;
        }
        let intersection = pair[0].intersection(&pair[1]).count();
        intersection as f64 / union as f64 >= 0.92
    })
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
            parent_message_id: None,
            child_message_ids: Vec::new(),
            branch_index: None,
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
    fn configured_roundtable_has_user_seed_and_provider_full_mesh_edges() {
        let configuration = RunConfiguration {
            mode: OrchestrationMode::Roundtable,
            participants: BTreeSet::from([ProviderId::Gpt, ProviderId::Claude]),
            ..RunConfiguration::default()
        };

        let graph = compile_configured_graph(&configuration);

        assert!(graph.edges.contains(&RouteEdge {
            source: ProviderId::User,
            target: ProviderId::Gpt,
            policy_id: None,
        }));
        assert!(graph.edges.contains(&RouteEdge {
            source: ProviderId::Gpt,
            target: ProviderId::Claude,
            policy_id: None,
        }));
        assert!(graph.edges.iter().all(|edge| edge.source != edge.target));
    }

    #[test]
    fn configured_relay_chain_preserves_user_order() {
        let configuration = RunConfiguration {
            mode: OrchestrationMode::RelayChain,
            participants: BTreeSet::from([ProviderId::Gpt, ProviderId::Claude, ProviderId::Gemini]),
            relay_order: vec![ProviderId::Claude, ProviderId::Gpt, ProviderId::Gemini],
            ..RunConfiguration::default()
        };

        let graph = compile_configured_graph(&configuration);
        assert_eq!(
            graph.edges,
            vec![
                RouteEdge {
                    source: ProviderId::User,
                    target: ProviderId::Claude,
                    policy_id: None,
                },
                RouteEdge {
                    source: ProviderId::Claude,
                    target: ProviderId::Gpt,
                    policy_id: None,
                },
                RouteEdge {
                    source: ProviderId::Gpt,
                    target: ProviderId::Gemini,
                    policy_id: None,
                },
            ]
        );
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
    fn frozen_cursor_never_advances() {
        let workspace_id = WorkspaceId::new();
        let original_message = sample_message(workspace_id, ProviderId::Gpt, "original");
        let new_message = sample_message(workspace_id, ProviderId::Gpt, "new");
        let cursor = DeliveryCursor {
            id: chatmux_common::DeliveryCursorId::new(),
            workspace_id,
            source_participant_id: ProviderId::Gpt,
            target_participant_id: ProviderId::Claude,
            last_delivered_message_id: Some(original_message.id),
            last_delivered_at: Some(original_message.timestamp),
            frozen: true,
        };

        let advanced = advance_cursor(&cursor, &[new_message]);

        assert_eq!(advanced, cursor);
    }

    #[test]
    fn unseen_delta_returns_only_messages_after_cursor() {
        let workspace_id = WorkspaceId::new();
        let messages = vec![
            sample_message(workspace_id, ProviderId::Gpt, "already delivered one"),
            sample_message(workspace_id, ProviderId::Gpt, "already delivered two"),
            sample_message(workspace_id, ProviderId::Gpt, "new response"),
        ];
        let cursor = DeliveryCursor {
            id: chatmux_common::DeliveryCursorId::new(),
            workspace_id,
            source_participant_id: ProviderId::Gpt,
            target_participant_id: ProviderId::Claude,
            last_delivered_message_id: Some(messages[1].id),
            last_delivered_at: Some(messages[1].timestamp),
            frozen: false,
        };

        let selected =
            select_messages_for_edge(&messages, &edge_policy(workspace_id), Some(&cursor));

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, messages[2].id);
    }

    #[test]
    fn unseen_delta_at_latest_cursor_is_empty() {
        let workspace_id = WorkspaceId::new();
        let messages = vec![
            sample_message(workspace_id, ProviderId::Gpt, "one"),
            sample_message(workspace_id, ProviderId::Gpt, "latest"),
        ];
        let cursor = DeliveryCursor {
            id: chatmux_common::DeliveryCursorId::new(),
            workspace_id,
            source_participant_id: ProviderId::Gpt,
            target_participant_id: ProviderId::Claude,
            last_delivered_message_id: Some(messages[1].id),
            last_delivered_at: Some(messages[1].timestamp),
            frozen: false,
        };

        let selected =
            select_messages_for_edge(&messages, &edge_policy(workspace_id), Some(&cursor));

        assert!(selected.is_empty());
    }

    #[test]
    fn unknown_cursor_position_returns_empty_without_regression() {
        let workspace_id = WorkspaceId::new();
        let messages = vec![sample_message(
            workspace_id,
            ProviderId::Gpt,
            "retained source message",
        )];
        let cursor = DeliveryCursor {
            id: chatmux_common::DeliveryCursorId::new(),
            workspace_id,
            source_participant_id: ProviderId::Gpt,
            target_participant_id: ProviderId::Claude,
            last_delivered_message_id: Some(MessageId::new()),
            last_delivered_at: Some(Utc::now()),
            frozen: false,
        };

        let selected =
            select_messages_for_edge(&messages, &edge_policy(workspace_id), Some(&cursor));
        let unchanged = advance_cursor(&cursor, &selected);

        assert!(selected.is_empty());
        assert_eq!(unchanged, cursor);
    }

    #[test]
    fn catch_up_none_is_initial_only_and_later_delta_is_delivered() {
        let workspace_id = WorkspaceId::new();
        let initial = sample_message(workspace_id, ProviderId::Gpt, "existing");
        let later = sample_message(workspace_id, ProviderId::Gpt, "later");
        let mut policy = edge_policy(workspace_id);
        policy.catch_up_policy = CatchUpPolicy::None;

        let initial_selection =
            select_messages_for_edge(std::slice::from_ref(&initial), &policy, None);
        assert!(initial_selection.is_empty());

        let cursor = DeliveryCursor {
            id: chatmux_common::DeliveryCursorId::new(),
            workspace_id,
            source_participant_id: ProviderId::Gpt,
            target_participant_id: ProviderId::Claude,
            last_delivered_message_id: Some(initial.id),
            last_delivered_at: Some(initial.timestamp),
            frozen: false,
        };
        let later_selection =
            select_messages_for_edge(&[initial, later.clone()], &policy, Some(&cursor));

        assert!(matches!(later_selection.as_slice(), [message] if message.id == later.id));
    }

    #[test]
    fn initial_catch_up_is_not_reapplied_after_cursor_exists() {
        let workspace_id = WorkspaceId::new();
        let messages = vec![
            sample_message(workspace_id, ProviderId::Gpt, "old one"),
            sample_message(workspace_id, ProviderId::Gpt, "cursor"),
            sample_message(workspace_id, ProviderId::Gpt, "new one"),
            sample_message(workspace_id, ProviderId::Gpt, "new two"),
        ];
        let mut policy = edge_policy(workspace_id);
        policy.catch_up_policy = CatchUpPolicy::LastN { count: 1 };
        let cursor = DeliveryCursor {
            id: chatmux_common::DeliveryCursorId::new(),
            workspace_id,
            source_participant_id: ProviderId::Gpt,
            target_participant_id: ProviderId::Claude,
            last_delivered_message_id: Some(messages[1].id),
            last_delivered_at: Some(messages[1].timestamp),
            frozen: false,
        };

        let selected = select_messages_for_edge(&messages, &policy, Some(&cursor));

        assert_eq!(
            selected
                .iter()
                .map(|message| message.id)
                .collect::<Vec<_>>(),
            vec![messages[2].id, messages[3].id]
        );
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
            &TimingPolicy::default(),
            &stop_policy,
            0,
            0,
            0,
            &[vec!["please HALT now".to_owned()]]
        ));
    }

    #[test]
    fn stagnation_requires_the_configured_number_of_low_delta_rounds() {
        let rounds = vec![
            vec![
                "The answer is alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu"
                    .to_owned(),
            ],
            vec![
                "Answer: alpha beta gamma delta epsilon zeta eta theta iota kappa lambda mu"
                    .to_owned(),
            ],
        ];

        assert!(convergence_stagnated(&rounds, 2));
        assert!(!convergence_stagnated(&rounds[..1], 2));
    }

    #[test]
    fn stagnation_rejects_materially_different_rounds() {
        let rounds = vec![
            vec!["alpha beta gamma delta epsilon zeta eta theta".to_owned()],
            vec!["completely different critique with novel evidence and risks".to_owned()],
        ];

        assert!(!convergence_stagnated(&rounds, 2));
    }

    #[test]
    fn wait_for_all_ignores_terminal_results_from_inactive_providers() {
        let active = BTreeSet::from([ProviderId::Gpt]);
        let responded = BTreeSet::from([ProviderId::Gpt, ProviderId::System]);

        assert!(barrier_satisfied(
            &BarrierPolicy::WaitForAll,
            &responded,
            &active
        ));
    }

    #[test]
    fn timing_policy_is_available_to_routing_tests() {
        let policy = TimingPolicy::default();
        assert_eq!(policy.max_concurrent_sends, 4);
    }

    #[test]
    fn stop_policy_uses_timing_policy_max_rounds() {
        let timing_policy = TimingPolicy {
            max_rounds: Some(2),
            ..TimingPolicy::default()
        };
        let stop_policy = StopPolicy {
            stop_on_max_rounds: true,
            stop_on_manual_pause: false,
            stop_on_sentinel_phrase: None,
            repeated_provider_failure_limit: None,
            repeated_timeout_limit: None,
            stagnation_window: None,
            require_approval_between_rounds: false,
        };

        assert!(should_stop_run(&timing_policy, &stop_policy, 2, 0, 0, &[]));
    }
}
