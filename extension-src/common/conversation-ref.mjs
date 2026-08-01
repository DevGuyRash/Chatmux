export function conversationRefFromUrl(providerId, url) {
  if (!url) {
    return {
      conversation_id: null,
      conversation_title: null,
      conversation_url: null,
    };
  }

  try {
    const parsed = new URL(url);
    const segments = parsed.pathname.split("/").filter(Boolean);
    const marker = providerId === "gemini"
      ? "app"
      : providerId === "claude"
        ? "chat"
        : providerId === "grok" && segments.includes("grok")
          ? "grok"
          : "c";
    const markerIndex = segments.indexOf(marker);
    return {
      conversation_id: markerIndex >= 0 ? segments[markerIndex + 1] ?? null : null,
      conversation_title: null,
      conversation_url: parsed.toString(),
    };
  } catch (_error) {
    return {
      conversation_id: null,
      conversation_title: null,
      conversation_url: null,
    };
  }
}

export function normalizeConversationUrl(url) {
  if (!url) {
    return null;
  }
  return String(url).split("#")[0].split("?")[0].replace(/\/+$/, "");
}

export function conversationRefMatchesTarget(currentRef, targetRef) {
  if (!currentRef || !targetRef) {
    return false;
  }
  if (currentRef.conversation_id && targetRef.conversation_id) {
    return currentRef.conversation_id === targetRef.conversation_id;
  }
  const currentUrl = normalizeConversationUrl(currentRef.url);
  const targetUrl = normalizeConversationUrl(targetRef.url);
  return Boolean(currentUrl && targetUrl && currentUrl === targetUrl);
}

export function hasStableConversationTarget(providerId, target) {
  if (!target) {
    return false;
  }
  if (String(target.conversation_id ?? "").trim()) {
    return true;
  }
  return Boolean(conversationRefFromUrl(providerId, target.url).conversation_id);
}
