import assert from "node:assert/strict";
import test from "node:test";

import {
  conversationRefFromUrl,
  hasStableConversationTarget,
} from "../extension-src/common/conversation-ref.mjs";

test("provider home and new-chat URLs remain provisional", () => {
  assert.equal(hasStableConversationTarget("gpt", { url: "https://chatgpt.com/" }), false);
  assert.equal(hasStableConversationTarget("gemini", { url: "https://gemini.google.com/app" }), false);
  assert.equal(hasStableConversationTarget("grok", { url: "https://grok.com/?q=" }), false);
  assert.equal(hasStableConversationTarget("claude", { url: "https://claude.ai/new" }), false);
});

test("provider conversation URLs are stable targets", () => {
  assert.equal(hasStableConversationTarget("gpt", { url: "https://chatgpt.com/c/chat-123" }), true);
  assert.equal(hasStableConversationTarget("gemini", { url: "https://gemini.google.com/app/chat-123" }), true);
  assert.equal(hasStableConversationTarget("grok", { url: "https://grok.com/c/chat-123" }), true);
  assert.equal(hasStableConversationTarget("claude", { url: "https://claude.ai/chat/chat-123" }), true);
});

test("conversation references preserve provider-issued IDs", () => {
  assert.equal(conversationRefFromUrl("gpt", "https://chatgpt.com/c/chat-123").conversation_id, "chat-123");
  assert.equal(hasStableConversationTarget("gpt", { conversation_id: "chat-456" }), true);
});
