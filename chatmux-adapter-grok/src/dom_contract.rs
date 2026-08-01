//! Pure Grok DOM classification and identity rules.

use chatmux_common::{MessageId, MessageRole};

#[derive(Debug, Clone, Copy)]
pub(super) struct FingerprintInput<'a> {
    pub conversation_key: Option<&'a str>,
    pub source_key: Option<&'a str>,
    pub role: MessageRole,
    pub dom_index: usize,
    pub text: &'a str,
}

pub(super) fn conversation_id_from_path(path: &str) -> Option<String> {
    segment_after(path, "c").or_else(|| segment_after(path, "grok"))
}

pub(super) fn classify_role(
    test_id: Option<&str>,
    author_role: Option<&str>,
    aria_label: Option<&str>,
) -> Option<MessageRole> {
    if let Some(role) = explicit_role(author_role) {
        return Some(role);
    }
    let semantic_text = format!(
        "{} {}",
        test_id.unwrap_or_default(),
        aria_label.unwrap_or_default()
    )
    .to_ascii_lowercase();
    if semantic_text.contains("user-message") || semantic_text.contains("user message") {
        Some(MessageRole::User)
    } else if semantic_text.contains("assistant-message")
        || semantic_text.contains("assistant message")
        || semantic_text.contains("grok response")
    {
        Some(MessageRole::Assistant)
    } else {
        None
    }
}

pub(super) fn text_indicates_rate_limit(text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    !normalized.is_empty()
        && [
            "rate limit",
            "too many requests",
            "try again later",
            "usage limit",
            "limit reached",
        ]
        .iter()
        .any(|pattern| normalized.contains(pattern))
}

pub(super) fn stable_message_id(input: FingerprintInput<'_>) -> MessageId {
    stable_id("grok", input)
}

fn explicit_role(author_role: Option<&str>) -> Option<MessageRole> {
    match author_role?.to_ascii_lowercase().as_str() {
        "user" => Some(MessageRole::User),
        "assistant" | "model" => Some(MessageRole::Assistant),
        "system" => Some(MessageRole::System),
        _ => None,
    }
}

fn segment_after(path: &str, marker: &str) -> Option<String> {
    let mut segments = path.split('/').filter(|segment| !segment.is_empty());
    while let Some(segment) = segments.next() {
        if segment == marker {
            return segments
                .next()
                .filter(|candidate| !candidate.is_empty())
                .map(ToOwned::to_owned);
        }
    }
    None
}

fn stable_id(provider: &str, input: FingerprintInput<'_>) -> MessageId {
    // Streaming content is intentionally excluded from identity. A turn keeps one id as it grows.
    let _streaming_text = input.text;
    let role = match input.role {
        MessageRole::User => "user",
        MessageRole::Assistant => "assistant",
        MessageRole::System => "system",
    };
    let fallback = format!("turn-{}", input.dom_index);
    let source = input.source_key.unwrap_or(&fallback);
    let fingerprint = format!(
        "{provider}:{}:{role}:{source}",
        input.conversation_key.unwrap_or("no-conversation")
    );
    let lower = fnv1a64(fingerprint.as_bytes(), 0xcbf29ce484222325);
    let upper = fnv1a64(fingerprint.as_bytes(), 0x84222325cbf29ce4);
    MessageId(uuid::Uuid::from_u128(
        ((upper as u128) << 64) | lower as u128,
    ))
}

fn fnv1a64(bytes: &[u8], offset: u64) -> u64 {
    bytes.iter().fold(offset, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_grok_conversation_route() {
        assert_eq!(
            conversation_id_from_path("/c/abc-123"),
            Some("abc-123".to_owned())
        );
        assert_eq!(
            conversation_id_from_path("/i/grok/abc-123"),
            Some("abc-123".to_owned())
        );
        assert_eq!(conversation_id_from_path("/i/grok"), None);
    }

    #[test]
    fn classifies_provider_message_roles() {
        assert_eq!(
            classify_role(Some("user-message"), None, None),
            Some(MessageRole::User)
        );
        assert_eq!(
            classify_role(Some("assistant-message"), None, None),
            Some(MessageRole::Assistant)
        );
        assert_eq!(classify_role(Some("toolbar"), None, None), None);
    }

    #[test]
    fn fingerprint_is_stable_and_source_sensitive() {
        let partial = stable_message_id(FingerprintInput {
            conversation_key: Some("conversation-1"),
            source_key: Some("message-1"),
            role: MessageRole::Assistant,
            dom_index: 2,
            text: "partial",
        });
        let completed = stable_message_id(FingerprintInput {
            conversation_key: Some("conversation-1"),
            source_key: Some("message-1"),
            role: MessageRole::Assistant,
            dom_index: 2,
            text: "completed",
        });
        let other = stable_message_id(FingerprintInput {
            conversation_key: Some("conversation-1"),
            source_key: Some("message-2"),
            role: MessageRole::Assistant,
            dom_index: 2,
            text: "completed",
        });
        assert_eq!(partial, completed);
        assert_ne!(completed, other);
    }

    #[test]
    fn fingerprint_without_source_stays_stable_while_text_streams() {
        let partial = stable_message_id(FingerprintInput {
            conversation_key: Some("conversation-1"),
            source_key: None,
            role: MessageRole::Assistant,
            dom_index: 2,
            text: "partial",
        });
        let completed = stable_message_id(FingerprintInput {
            conversation_key: Some("conversation-1"),
            source_key: None,
            role: MessageRole::Assistant,
            dom_index: 2,
            text: "completed response",
        });

        assert_eq!(partial, completed);
    }

    #[test]
    fn rate_limit_classification_requires_blocking_language() {
        assert!(text_indicates_rate_limit("Rate limit reached"));
        assert!(!text_indicates_rate_limit("Grok saved your draft"));
    }
}
