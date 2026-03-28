const CHATGPT_URL_RE = /^https:\/\/(chatgpt\.com|chat\.openai\.com)\//i;

const CHATGPT_SELECTORS = {
  input: ["#prompt-textarea", "textarea", "[contenteditable='true']"],
  send: ["button[data-testid='send-button']", "button[aria-label*='Send']"],
  transcript: ["main", "[data-message-author-role]", "article"],
  login: ["form[data-provider='auth0']", "button[data-testid='login-button']"],
  rateLimit: ["[role='alert']", ".text-red-500"],
  challenge: [
    "iframe[title*='challenge']",
    "#challenge-stage",
    "[name='cf-turnstile-response']",
  ],
};

async function openChatGptPage(context) {
  const page = await context.newPage();
  await page.goto("https://chatgpt.com/", {
    waitUntil: "domcontentloaded",
    timeout: 45_000,
  });
  return page;
}

function findChatGptPage(context) {
  return context.pages().find((page) => CHATGPT_URL_RE.test(page.url())) ?? null;
}

async function firstVisibleLocator(page, selectors) {
  for (const selector of selectors) {
    const locator = page.locator(selector).first();
    if (await locator.isVisible().catch(() => false)) {
      return { selector, locator };
    }
  }
  return null;
}

async function detectChatGptSurface(page) {
  const input = await firstVisibleLocator(page, CHATGPT_SELECTORS.input);
  const transcript = await firstVisibleLocator(page, CHATGPT_SELECTORS.transcript);
  const login = await firstVisibleLocator(page, CHATGPT_SELECTORS.login);
  const rateLimit = await firstVisibleLocator(page, CHATGPT_SELECTORS.rateLimit);
  const challenge = await firstVisibleLocator(page, CHATGPT_SELECTORS.challenge);
  const send = await firstVisibleLocator(page, CHATGPT_SELECTORS.send);

  if (input) {
    return {
      kind: "ready",
      inputSelector: input.selector,
      sendSelector: send?.selector ?? null,
      transcriptSelector: transcript?.selector ?? null,
    };
  }
  if (login) {
    return { kind: "login-required", loginSelector: login.selector };
  }
  if (rateLimit) {
    return { kind: "rate-limited", rateLimitSelector: rateLimit.selector };
  }
  if (challenge) {
    return { kind: "challenge", challengeSelector: challenge.selector };
  }
  if (transcript) {
    return { kind: "transcript-only", transcriptSelector: transcript.selector };
  }

  return { kind: "unknown" };
}

module.exports = {
  detectChatGptSurface,
  findChatGptPage,
  openChatGptPage,
};
