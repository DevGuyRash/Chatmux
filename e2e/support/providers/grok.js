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

async function validateSubmitButton(locator) {
  const base = await validateButton(locator);
  return { ...base, valid: base.valid && base.type === "submit" };
}

module.exports = createProviderSupport({
  id: "grok",
  providerId: "grok",
  displayName: "Grok",
  url: process.env.CHATMUX_E2E_GROK_URL || "https://grok.com/",
  urlPatterns: [
    /^https:\/\/(?:grok\.com(?:\/|$)|x\.com\/i\/grok(?:[/?#]|$))/i,
  ],
  clearComposer: async (composer, page) => {
    await composer.focus();
    await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
    await page.keyboard.press("Backspace");
  },
  targets: {
    composer: {
      candidates: [
        role("textbox", "Ask Grok anything", { exact: true }),
        css("textarea[placeholder]", "css=textarea[placeholder]"),
      ],
      validate: validateEditable,
    },
    sendButton: {
      requireVisible: false,
      candidates: [role("button", "Submit", { exact: true })],
      validate: validateSubmitButton,
    },
    transcript: {
      candidates: [role("main")],
      validate: validateTranscript,
    },
    assistantMessage: {
      candidates: [
        css("[data-testid*='assistant-message']", "css=[data-testid*=assistant-message]"),
      ],
    },
    generating: {
      candidates: [
        role("button", /stop/i),
        css("[data-testid*='stop']", "css=[data-testid*=stop]"),
      ],
    },
    loginRequired: {
      candidates: [
        role("button", /sign in|log in/i),
        css(
          "button[data-testid='LoginForm_Login_Button']",
          "css=button[data-testid=LoginForm_Login_Button]"
        ),
      ],
    },
    permissionRequired: {
      candidates: [role("alert")],
      validate: (locator) => validateText(/permission|access denied/i, locator),
    },
    rateLimited: {
      candidates: [
        css("[data-testid='toast']", "css=[data-testid=toast]"),
        role("alert"),
      ],
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
