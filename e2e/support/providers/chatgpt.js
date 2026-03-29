const CHATGPT_URL_RE = /^https:\/\/(chatgpt\.com|chat\.openai\.com)\//i;
const DEFAULT_CHATGPT_URL = process.env.CHATMUX_E2E_CHATGPT_URL || "https://chatgpt.com/";

const CHATGPT_SELECTORS = {
  input: ["#prompt-textarea", "textarea", "[contenteditable='true']"],
  send: ["button[data-testid='send-button']", "button[aria-label*='Send']"],
  transcript: ["main", "[data-message-author-role]", "article"],
  assistantMessage: [
    "[data-message-author-role='assistant']",
    "article[data-testid*='conversation-turn']",
  ],
  userMessage: ["[data-message-author-role='user']"],
  generating: ["button[aria-label*='Stop']", "[data-testid='stop-button']"],
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
  await page.goto(DEFAULT_CHATGPT_URL, {
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

async function pollUntil(read, predicate, timeout, label) {
  const deadline = Date.now() + timeout;
  let lastValue;

  while (Date.now() < deadline) {
    lastValue = await read();
    if (predicate(lastValue)) {
      return lastValue;
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
  }

  throw new Error(
    `${label} did not satisfy the expected condition within ${timeout}ms. Last value: ${JSON.stringify(lastValue)}`
  );
}

function joinedSelector(selectors) {
  return selectors.join(", ");
}

function normalizeText(text) {
  return String(text || "").replace(/\s+/g, " ").trim();
}

function assistantMessages(page) {
  return page.locator(joinedSelector(CHATGPT_SELECTORS.assistantMessage));
}

function userMessages(page) {
  return page.locator(joinedSelector(CHATGPT_SELECTORS.userMessage));
}

function isGenerating(page) {
  return firstVisibleLocator(page, CHATGPT_SELECTORS.generating).then(Boolean);
}

async function waitForChatGptReady(page, timeout = 45_000) {
  return await pollUntil(
    async () => await detectChatGptSurface(page),
    (surface) => surface?.kind === "ready",
    timeout,
    "ChatGPT ready surface"
  );
}

async function waitForPromptEcho(page, prompt, previousUserCount = 0, timeout = 45_000) {
  await pollUntil(
    async () => {
      const count = await userMessages(page).count();
      if (count <= previousUserCount) {
        return "";
      }
      return normalizeText(await userMessages(page).nth(count - 1).innerText());
    },
    (text) => text.includes(normalizeText(prompt)),
    timeout,
    "ChatGPT user prompt echo"
  );
}

async function waitForAssistantResponse(page, previousAssistantCount = 0, timeout = 90_000) {
  return await pollUntil(
    async () => {
      const count = await assistantMessages(page).count();
      if (count <= previousAssistantCount) {
        return null;
      }

      const text = normalizeText(
        await assistantMessages(page).nth(count - 1).innerText()
      );
      if (!text) {
        return null;
      }

      return {
        count,
        generating: await isGenerating(page),
        text,
      };
    },
    (result) => Boolean(result && typeof result.text === "string" && result.text.length > 0),
    timeout,
    "ChatGPT assistant response"
  );
}

module.exports = {
  assistantMessages,
  detectChatGptSurface,
  findChatGptPage,
  normalizeText,
  openChatGptPage,
  userMessages,
  waitForAssistantResponse,
  waitForChatGptReady,
  waitForPromptEcho,
};
