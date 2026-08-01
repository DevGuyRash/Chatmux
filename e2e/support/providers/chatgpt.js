const { expect } = require("playwright/test");
const {
  createProviderSupport,
  css,
  normalizeText,
  role,
  validateButton,
  validateEditable,
  validateTranscript,
} = require("./provider-surface");

const RATE_LIMIT_PATTERN =
  /rate limit|too many requests|too many messages|try again later|unusual activity|systems are a bit busy|you(?:'|’)ve reached/i;

async function validateText(pattern, locator) {
  const text = normalizeText(await locator.innerText().catch(() => ""));
  return { valid: pattern.test(text), text: text.slice(0, 200) };
}

const support = createProviderSupport({
  id: "chatgpt",
  providerId: "gpt",
  displayName: "ChatGPT",
  url: process.env.CHATMUX_E2E_CHATGPT_URL || "https://chatgpt.com/",
  urlPatterns: [/^https:\/\/(chatgpt\.com|chat\.openai\.com)\//i],
  targets: {
    composer: {
      candidates: [
        role("textbox", "Chat with ChatGPT", { exact: true }),
        css("#prompt-textarea", "css=#prompt-textarea"),
      ],
      validate: validateEditable,
    },
    sendButton: {
      requireVisible: false,
      candidates: [
        role("button", "Send prompt", { exact: true }),
        css("button[data-testid='send-button']", "css=button[data-testid='send-button']"),
      ],
      validate: validateButton,
    },
    transcript: {
      candidates: [role("main")],
      validate: validateTranscript,
    },
    userMessage: {
      candidates: [
        css(
          "[data-message-author-role='user']",
          "css=[data-message-author-role='user']"
        ),
      ],
    },
    assistantMessage: {
      candidates: [
        css(
          "[data-message-author-role='assistant']",
          "css=[data-message-author-role='assistant']"
        ),
      ],
    },
    generating: {
      candidates: [
        role("button", /stop/i),
        css("[data-testid='stop-button']", "css=[data-testid='stop-button']"),
      ],
    },
    loginRequired: {
      candidates: [
        role("button", /log in|sign in/i),
        css("form[data-provider='auth0']", "css=form[data-provider='auth0']"),
      ],
    },
    rateLimited: {
      candidates: [role("alert")],
      validate: (locator) => validateText(RATE_LIMIT_PATTERN, locator),
    },
    challenge: {
      candidates: [
        css("iframe[title*='challenge' i]", "css=iframe[title*=challenge]"),
        css("#challenge-stage", "css=#challenge-stage"),
        css("[name='cf-turnstile-response']", "css=[name=cf-turnstile-response]"),
      ],
    },
    error: {
      candidates: [
        css("[data-testid='conversation-error']", "css=[data-testid=conversation-error]"),
      ],
    },
    loading: {
      candidates: [css("main [aria-busy='true']", "css=main [aria-busy=true]")],
    },
  },
});

async function semanticLocators(page, targetName) {
  const result = await support.findSemanticTarget(page, targetName, {
    requireVisible: false,
  });
  if (!result.matched) {
    return [];
  }

  const definition = support.targets[targetName];
  for (const candidate of definition.candidates) {
    const locator = candidate.kind === "role"
      ? page.getByRole(candidate.role, {
          name: candidate.name,
          exact: candidate.exact ?? false,
          includeHidden: candidate.includeHidden ?? false,
        })
      : page.locator(candidate.selector);
    const matches = await locator.all();
    if (matches.length > 0) {
      return matches;
    }
  }
  return [];
}

async function isGenerating(page) {
  const result = await support.findSemanticTarget(page, "generating");
  return result.matched;
}

async function collectChatGptState(page) {
  const surface = await support.classifyPageState(page);
  const userTurns = await semanticLocators(page, "userMessage");
  const assistantTurns = await semanticLocators(page, "assistantMessage");
  const lastUser = userTurns.at(-1);
  const lastAssistant = assistantTurns.at(-1);

  const pageFacts = await page.evaluate(() => {
    const pathname = location.pathname;
    const parts = pathname.split("/");
    const conversationIndex = parts.indexOf("c");
    const currentProjectId =
      parts.find((segment) => segment.startsWith("g-p-")) ?? null;
    const currentConversationId =
      conversationIndex >= 0 ? parts[conversationIndex + 1] ?? null : null;
    const projects = Array.from(document.querySelectorAll("a[href*='/project']"))
      .map((node) => ({
        title: (node.textContent || "").trim().replace(/\s+/g, " "),
        href: node.getAttribute("href"),
      }))
      .filter((item) => item.title && item.href)
      .slice(0, 50);
    const conversations = Array.from(document.querySelectorAll("a[href*='/c/']"))
      .map((node) => ({
        title:
          node.getAttribute("aria-label")?.split(",")[0]?.trim() ||
          (node.textContent || "").trim().replace(/\s+/g, " "),
        href: node.getAttribute("href"),
      }))
      .filter((item) => item.title && item.href)
      .slice(0, 50);

    return {
      pathname,
      currentProjectId,
      currentConversationId,
      projects,
      conversations,
    };
  });

  return {
    surface,
    url: support.safeUrl(page.url()),
    title: support.safeText(await page.title().catch(() => ""), 200),
    userCount: userTurns.length,
    assistantCount: assistantTurns.length,
    lastUserText: lastUser
      ? support.safeText(await lastUser.innerText().catch(() => ""))
      : "",
    lastAssistantText: lastAssistant
      ? support.safeText(await lastAssistant.innerText().catch(() => ""))
      : "",
    generating: await isGenerating(page),
    ...pageFacts,
  };
}

async function pollUntil(read, predicate, timeout, label) {
  let lastValue;
  try {
    await expect
      .poll(
        async () => {
          lastValue = await read();
          return Boolean(predicate(lastValue));
        },
        { message: label, timeout }
      )
      .toBe(true);
  } catch (error) {
    throw new Error(
      `${label} did not reach the expected outcome within ${timeout}ms. ` +
        `Last observed state: ${JSON.stringify(lastValue)}`,
      { cause: error }
    );
  }
  return lastValue;
}

async function waitForChatGptReady(page, timeout = 45_000) {
  return pollUntil(
    () => support.classifyPageState(page),
    (state) => state.kind === "ready",
    timeout,
    "ChatGPT ready surface"
  );
}

async function waitForPromptEcho(page, prompt, previousUserCount = 0, timeout = 45_000) {
  const expectedText = normalizeText(prompt);
  return pollUntil(
    async () => {
      const turns = await semanticLocators(page, "userMessage");
      const matching = [];
      for (const turn of turns) {
        const text = normalizeText(await turn.innerText().catch(() => ""));
        if (text === expectedText) {
          matching.push(text);
        }
      }
      return { count: turns.length, matching };
    },
    (state) => state.count > previousUserCount && state.matching.length === 1,
    timeout,
    "ChatGPT exact user-prompt echo"
  );
}

async function waitForAssistantResponse(
  page,
  expectedToken,
  previousAssistantCount = 0,
  timeout = 90_000
) {
  const expectedText = normalizeText(expectedToken);
  return pollUntil(
    async () => {
      const turns = await semanticLocators(page, "assistantMessage");
      const last = turns.at(-1);
      return {
        count: turns.length,
        generating: await isGenerating(page),
        text: last ? normalizeText(await last.innerText().catch(() => "")) : "",
      };
    },
    (state) =>
      state.count > previousAssistantCount &&
      state.generating === false &&
      state.text === expectedText,
    timeout,
    "ChatGPT completed exact assistant response"
  );
}

module.exports = {
  ...support,
  collectChatGptState,
  normalizeText,
  waitForAssistantResponse,
  waitForChatGptReady,
  waitForPromptEcho,
};
