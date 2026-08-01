const {
  createProviderSupport,
  css,
  normalizeText,
  role,
  validateButton,
  validateEditable,
} = require("./provider-surface");

async function validateText(pattern, locator) {
  const text = normalizeText(await locator.innerText().catch(() => ""));
  return { valid: pattern.test(text), text: text.slice(0, 200) };
}

async function validateGeminiTranscript(locator) {
  const facts = await locator.evaluate((node) => ({
    tagName: node.tagName.toLowerCase(),
    role: node.getAttribute("role"),
  }));
  return {
    valid:
      facts.tagName === "main" ||
      facts.tagName === "chat-app" ||
      facts.role === "main",
    ...facts,
  };
}

module.exports = createProviderSupport({
  id: "gemini",
  providerId: "gemini",
  displayName: "Gemini",
  url: process.env.CHATMUX_E2E_GEMINI_URL || "https://gemini.google.com/",
  urlPatterns: [/^https:\/\/gemini\.google\.com\//i],
  targets: {
    composer: {
      candidates: [
        role("textbox", "Enter a prompt for Gemini", { exact: true }),
        css("rich-textarea [contenteditable='true']", "css=rich-textarea [contenteditable=true]"),
      ],
      validate: validateEditable,
    },
    sendButton: {
      requireVisible: false,
      candidates: [role("button", "Send message", { exact: true })],
      validate: validateButton,
    },
    transcript: {
      candidates: [role("main"), css("chat-app", "css=chat-app")],
      validate: validateGeminiTranscript,
    },
    assistantMessage: {
      candidates: [
        css("message-content", "css=message-content"),
        css(".model-response-text", "css=.model-response-text"),
      ],
    },
    generating: {
      candidates: [role("button", /stop/i), role("progressbar")],
    },
    loginRequired: {
      candidates: [role("button", "Sign in", { exact: true })],
    },
    permissionRequired: {
      candidates: [role("alert")],
      validate: (locator) => validateText(/permission|access denied/i, locator),
    },
    rateLimited: {
      candidates: [
        css("snackbar-container", "css=snackbar-container"),
        role("alert"),
      ],
      validate: (locator) =>
        validateText(/rate limit|too many|try again later|usage limit|quota/i, locator),
    },
    challenge: {
      candidates: [
        css("iframe[title*='challenge' i]", "css=iframe[title*=challenge]"),
        css("[name='cf-turnstile-response']", "css=[name=cf-turnstile-response]"),
      ],
    },
    notFound: {
      candidates: [role("heading", /not found/i)],
    },
    error: {
      candidates: [role("alert")],
      validate: (locator) =>
        validateText(/error|something went wrong|unable to/i, locator),
    },
    loading: {
      candidates: [role("progressbar"), css("[aria-busy='true']", "css=[aria-busy=true]")],
    },
  },
});
