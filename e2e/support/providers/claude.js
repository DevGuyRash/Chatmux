const {
  createProviderSupport,
  css,
  normalizeText,
  role,
  validateButton,
  validateEditable,
  validateTranscript,
} = require("./provider-surface");

async function validateText(pattern, locator) {
  const text = normalizeText(await locator.innerText().catch(() => ""));
  return { valid: pattern.test(text), text: text.slice(0, 200) };
}

module.exports = createProviderSupport({
  id: "claude",
  providerId: "claude",
  displayName: "Claude",
  url: process.env.CHATMUX_E2E_CLAUDE_URL || "https://claude.ai/",
  urlPatterns: [/^https:\/\/claude\.ai\//i],
  clearComposer: async (composer, page) => {
    await composer.focus();
    await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
    await page.keyboard.press("Backspace");
  },
  targets: {
    composer: {
      candidates: [
        role("textbox", "Write your prompt to Claude", { exact: true }),
        css(
          "div[contenteditable='true'][role='textbox']",
          "css=div[contenteditable=true][role=textbox]"
        ),
      ],
      validate: validateEditable,
    },
    sendButton: {
      requireVisible: false,
      candidates: [role("button", "Send message", { exact: true })],
      validate: validateButton,
    },
    transcript: {
      candidates: [role("main")],
      validate: validateTranscript,
    },
    assistantMessage: {
      candidates: [
        css("[data-testid*='assistant']", "css=[data-testid*=assistant]"),
      ],
    },
    generating: {
      candidates: [
        role("button", /stop/i),
        css("[data-state='streaming']", "css=[data-state=streaming]"),
      ],
    },
    loginRequired: {
      candidates: [
        role("textbox", /email/i),
        role("button", /continue with google|log in|sign in/i),
      ],
    },
    permissionRequired: {
      candidates: [role("alert")],
      validate: (locator) => validateText(/permission|access denied/i, locator),
    },
    rateLimited: {
      candidates: [role("alert")],
      validate: (locator) =>
        validateText(/rate limit|too many|try again later|usage limit/i, locator),
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
