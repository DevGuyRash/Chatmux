const CHATGPT_URL_RE = /^https:\/\/(chatgpt\.com|chat\.openai\.com)\//i;
const DEFAULT_CHATGPT_URL = process.env.CHATMUX_E2E_CHATGPT_URL || "https://chatgpt.com/";

const CHATGPT_SELECTORS = {
  input: ["#prompt-textarea", "textarea", "[contenteditable='true']"],
  send: [
    "button[data-testid='send-button']",
    "button[aria-label='Send prompt']",
    "button[aria-label*='Send']",
  ],
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

async function collectChatGptState(page) {
  const surface = await detectChatGptSurface(page);
  const userCount = await userMessages(page).count().catch(() => 0);
  const assistantCount = await assistantMessages(page).count().catch(() => 0);
  const lastUserText =
    userCount > 0
      ? normalizeText(await userMessages(page).nth(userCount - 1).innerText().catch(() => ""))
      : "";
  const lastAssistantText =
    assistantCount > 0
      ? normalizeText(
          await assistantMessages(page).nth(assistantCount - 1).innerText().catch(() => "")
        )
      : "";

  const pageFacts = await page.evaluate(() => {
    const composer = document.querySelector("#prompt-textarea");
    const form = composer?.closest("form");
    const pathname = location.pathname;
    const currentProjectId =
      pathname.split("/").find((segment) => segment.startsWith("g-p-")) ?? null;
    const currentConversationId = (() => {
      const parts = pathname.split("/");
      const index = parts.indexOf("c");
      return index >= 0 ? parts[index + 1] ?? null : null;
    })();
    const thoughtSummaries = Array.from(
      document.querySelectorAll(".agent-turn button[disabled]")
    )
      .map((node) => (node.textContent || "").trim())
      .filter(Boolean)
      .slice(-5);
    const assistantTurns = Array.from(
      document.querySelectorAll("[data-message-author-role='assistant']")
    )
      .map((node) => ({
        messageId: node.getAttribute("data-message-id"),
        modelSlug: node.getAttribute("data-message-model-slug"),
        text: (node.textContent || "").trim().replace(/\s+/g, " ").slice(0, 500),
      }))
      .slice(-3);
    const projects = Array.from(document.querySelectorAll("a[href*='/project']"))
      .map((node) => ({
        title: (node.textContent || "").trim().replace(/\s+/g, " "),
        href: node.getAttribute("href"),
      }))
      .filter((item) => item.title && item.href)
      .slice(0, 20);
    const conversations = Array.from(document.querySelectorAll("a[href*='/c/']"))
      .map((node) => ({
        title:
          node.getAttribute("aria-label")?.split(",")[0]?.trim() ||
          (node.textContent || "").trim().replace(/\s+/g, " "),
        href: node.getAttribute("href"),
      }))
      .filter((item) => item.title && item.href)
      .slice(0, 25);

    return {
      url: location.href,
      title: document.title,
      pathname,
      currentProjectId,
      currentConversationId,
      composerText: (composer?.textContent || "").trim(),
      composerButtons: form
        ? Array.from(form.querySelectorAll("button")).map((button) => ({
            text: (button.innerText || "").trim().replace(/\s+/g, " "),
            ariaLabel: button.getAttribute("aria-label"),
            testid: button.getAttribute("data-testid"),
            expanded: button.getAttribute("aria-expanded"),
            popup: button.getAttribute("aria-haspopup"),
            disabled: button.disabled,
          }))
        : [],
      thoughtSummaries,
      assistantTurns,
      projects,
      conversations,
    };
  });

  return {
    surface,
    userCount,
    assistantCount,
    lastUserText,
    lastAssistantText,
    generating: await isGenerating(page),
    ...pageFacts,
  };
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
  collectChatGptState,
  detectChatGptSurface,
  findChatGptPage,
  normalizeText,
  openChatGptPage,
  userMessages,
  waitForAssistantResponse,
  waitForChatGptReady,
  waitForPromptEcho,
};
