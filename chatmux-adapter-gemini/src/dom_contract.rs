//! Pure Gemini DOM classification and identity rules.

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
    segment_after(path, "app")
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

pub(super) fn classify_role(
    tag_name: &str,
    test_id: Option<&str>,
    author_role: Option<&str>,
) -> Option<MessageRole> {
    if let Some(role) = explicit_role(author_role) {
        return Some(role);
    }

    let tag_name = tag_name.to_ascii_lowercase();
    let test_id = test_id.unwrap_or_default().to_ascii_lowercase();
    if tag_name == "user-query" || test_id.contains("user-message") {
        Some(MessageRole::User)
    } else if tag_name == "model-response"
        || tag_name == "dual-model-response"
        || tag_name == "response-container"
        || test_id.contains("assistant-message")
        || test_id.contains("model-response")
    {
        Some(MessageRole::Assistant)
    } else {
        None
    }
}

fn explicit_role(author_role: Option<&str>) -> Option<MessageRole> {
    match author_role?.to_ascii_lowercase().as_str() {
        "user" => Some(MessageRole::User),
        "assistant" | "model" => Some(MessageRole::Assistant),
        "system" => Some(MessageRole::System),
        _ => None,
    }
}

pub(super) fn text_indicates_rate_limit(text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    !normalized.is_empty()
        && [
            "rate limit",
            "too many requests",
            "try again later",
            "you've reached your limit",
            "usage limit",
        ]
        .iter()
        .any(|pattern| normalized.contains(pattern))
}

pub(super) fn stable_message_id(input: FingerprintInput<'_>) -> MessageId {
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
        "gemini:{}:{role}:{source}",
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
    fn parses_gemini_conversation_route() {
        assert_eq!(
            conversation_id_from_path("/app/abc-123"),
            Some("abc-123".to_owned())
        );
        assert_eq!(conversation_id_from_path("/app"), None);
        assert_eq!(conversation_id_from_path("/"), None);
    }

    #[test]
    fn classifies_provider_message_roles() {
        assert_eq!(
            classify_role("user-query", None, None),
            Some(MessageRole::User)
        );
        assert_eq!(
            classify_role("model-response", None, None),
            Some(MessageRole::Assistant)
        );
        assert_eq!(
            classify_role("response-container", None, None),
            Some(MessageRole::Assistant)
        );
        assert_eq!(
            classify_role("dual-model-response", None, None),
            Some(MessageRole::Assistant)
        );
        assert_eq!(
            classify_role("div", Some("assistant-message"), None),
            Some(MessageRole::Assistant)
        );
        assert_eq!(classify_role("div", Some("toolbar"), None), None);
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
            text: "completed response",
        });
        let other_source = stable_message_id(FingerprintInput {
            conversation_key: Some("conversation-1"),
            source_key: Some("message-2"),
            role: MessageRole::Assistant,
            dom_index: 2,
            text: "completed response",
        });

        assert_eq!(partial, completed);
        assert_ne!(completed, other_source);
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
        assert!(text_indicates_rate_limit("You've reached your limit"));
        assert!(!text_indicates_rate_limit("Gemini updated your draft"));
    }
}
