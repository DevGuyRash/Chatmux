const {
  dispatchUiCommand,
  expect,
  test,
} = require("../support/chrome-extension");
const {
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
} = require("../support/providers/chatgpt");

const MANUAL_LOGIN_MODE = process.env.CHATMUX_E2E_MANUAL_LOGIN === "1";
const MANUAL_LOGIN_TIMEOUT_MS =
  Number(process.env.CHATMUX_E2E_MANUAL_LOGIN_TIMEOUT_SECS || "600") * 1000;
const CHATMUX_E2E_PROJECT_NAME =
  process.env.CHATMUX_E2E_PROJECT_NAME || "Chatmux E2E";

async function attachJson(testInfo, name, value) {
  await testInfo.attach(name, {
    body: Buffer.from(JSON.stringify(value, null, 2)),
    contentType: "application/json",
  });
}

async function createWorkspaceAndOpen(page) {
  const createResponse = await dispatchUiCommand(page, {
    type: "create_workspace",
    name: "Workspace 1",
  });
  expect(createResponse?.ok).toBeTruthy();

  const workspaces = createResponse?.events?.find(
    (event) => event?.type === "workspace_list"
  )?.workspaces;
  const workspaceId = workspaces?.[workspaces.length - 1]?.id;
  expect(workspaceId).toBeTruthy();

  const openResponse = await dispatchUiCommand(page, {
    type: "open_workspace",
    workspace_id: workspaceId,
  });
  expect(openResponse?.ok).toBeTruthy();
  return workspaceId;
}

async function leaveOnlyChatGptSelected(page) {
  await expect(page.getByRole("button", { name: "ChatGPT" })).toBeVisible();

  for (const provider of ["Gemini", "Grok", "Claude"]) {
    const chip = page.getByRole("button", { name: provider });
    if (await chip.isVisible().catch(() => false)) {
      await chip.click();
    }
  }
}

async function waitForExtensionAssistantMessage(page, timeout = 30_000) {
  const deadline = Date.now() + timeout;
  const assistantCards = page.locator("[role='article'][aria-label*='ChatGPT at']");
  let lastText = "";

  while (Date.now() < deadline) {
    const count = await assistantCards.count();
    if (count > 0) {
      lastText = normalizeText(await assistantCards.nth(count - 1).innerText());
      if (lastText) {
        return lastText;
      }
    }

    await page.waitForTimeout(500);
  }

  throw new Error(
    `Extension did not ingest a ChatGPT assistant message within ${timeout}ms. Last text: ${JSON.stringify(lastText)}`
  );
}

async function requestWorkspaceSnapshot(page, workspaceId) {
  const response = await dispatchUiCommand(page, {
    type: "request_workspace_snapshot",
    workspace_id: workspaceId,
  });
  expect(response?.ok).toBeTruthy();
  return response?.events?.find((event) => event?.type === "workspace_snapshot")?.snapshot ?? null;
}

async function waitForWorkspaceMessage(page, workspaceId, predicate, timeout = 30_000) {
  const deadline = Date.now() + timeout;
  let lastSnapshot = null;

  while (Date.now() < deadline) {
    lastSnapshot = await requestWorkspaceSnapshot(page, workspaceId);
    const messages = lastSnapshot?.recent_messages ?? [];
    const match = messages.find(predicate);
    if (match) {
      return { snapshot: lastSnapshot, message: match };
    }
    await page.waitForTimeout(500);
  }

  throw new Error(
    `Workspace message did not appear within ${timeout}ms. Last snapshot: ${JSON.stringify(lastSnapshot)}`
  );
}

async function collectExtensionState(page) {
  const messageLog = page.getByRole("log", { name: "Message log" });
  const articles = page.locator("[role='article']");
  const articleCount = await articles.count().catch(() => 0);
  const lastArticleText =
    articleCount > 0
      ? normalizeText(await articles.nth(articleCount - 1).innerText().catch(() => ""))
      : "";

  const selectedProviders = await Promise.all(
    ["ChatGPT", "Gemini", "Grok", "Claude"].map(async (provider) => {
      const chip = page.getByRole("button", { name: provider });
      const visible = await chip.isVisible().catch(() => false);
      if (!visible) {
        return { provider, visible: false, pressed: null };
      }
      return {
        provider,
        visible: true,
        pressed: await chip.getAttribute("aria-pressed").catch(() => null),
      };
    })
  );

  return {
    url: page.url(),
    articleCount,
    lastArticleText,
    selectedProviders,
    composerValue: await page.getByPlaceholder("Type a message…").inputValue().catch(() => null),
    messageLogText: normalizeText(await messageLog.innerText().catch(() => "")),
  };
}

async function attachRoundtripState(testInfo, extensionPage, chatGptPage, stage, workspaceId = null) {
  await attachJson(testInfo, `roundtrip-${stage}-extension.json`, await collectExtensionState(extensionPage));
  await attachJson(testInfo, `roundtrip-${stage}-chatgpt.json`, await collectChatGptState(chatGptPage));
  if (workspaceId) {
    await attachJson(
      testInfo,
      `roundtrip-${stage}-workspace.json`,
      await requestWorkspaceSnapshot(extensionPage, workspaceId)
    );
  }
}

async function dispatchProviderCommand(page, command) {
  const response = await dispatchUiCommand(page, command);
  expect(response?.ok).toBeTruthy();
  return response;
}

async function waitForProjectPresence(chatGptPage, projectTitle, timeout = 20_000) {
  const deadline = Date.now() + timeout;
  let lastState = null;

  while (Date.now() < deadline) {
    lastState = await collectChatGptState(chatGptPage);
    if (lastState.projects?.some((project) => project.title === projectTitle)) {
      return lastState;
    }
    await chatGptPage.waitForTimeout(500);
  }

  throw new Error(
    `Project ${JSON.stringify(projectTitle)} was not visible in ChatGPT within ${timeout}ms. Last state: ${JSON.stringify(lastState)}`
  );
}

function projectIdFromHref(href) {
  return String(href || "")
    .split("/")
    .find((segment) => segment.startsWith("g-p-")) || null;
}

async function waitForProjectSelected(chatGptPage, projectId, timeout = 30_000) {
  const deadline = Date.now() + timeout;
  let lastState = null;

  while (Date.now() < deadline) {
    lastState = await collectChatGptState(chatGptPage);
    if (lastState.currentProjectId === projectId) {
      return lastState;
    }
    await chatGptPage.waitForTimeout(500);
  }

  throw new Error(
    `Project ${JSON.stringify(projectId)} was not selected within ${timeout}ms. Last state: ${JSON.stringify(lastState)}`
  );
}

async function waitForReasoningMode(chatGptPage, expectedLabel, timeout = 20_000) {
  const deadline = Date.now() + timeout;
  let lastState = null;

  while (Date.now() < deadline) {
    lastState = await collectChatGptState(chatGptPage);
    const body = JSON.stringify(lastState).toLowerCase();
    if (body.includes(String(expectedLabel).toLowerCase())) {
      return lastState;
    }
    await chatGptPage.waitForTimeout(500);
  }

  return lastState;
}

test.describe("ChatGPT roundtrip", () => {
  test("sends from the extension, observes ChatGPT, and ingests the reply", async ({
    chatmux,
  }, testInfo) => {
    test.setTimeout(
      MANUAL_LOGIN_MODE
        ? Math.max(180_000, MANUAL_LOGIN_TIMEOUT_MS + 180_000)
        : 120_000
    );

    test.skip(
      !process.env.CHATMUX_E2E_CHROME_CDP_URL &&
        !process.env.CHATMUX_E2E_CHROME_USER_DATA_DIR,
      "Set CHATMUX_E2E_CHROME_CDP_URL to an attached Chrome session or CHATMUX_E2E_CHROME_USER_DATA_DIR to a signed-in Chrome profile for the live ChatGPT roundtrip."
    );

    const shouldOpenChatGpt = process.env.CHATMUX_E2E_OPEN_CHATGPT === "1";
    let chatGptPage = findChatGptPage(chatmux.context);

    if (!chatGptPage && shouldOpenChatGpt) {
      chatGptPage = await openChatGptPage(chatmux.context);
    }

    test.skip(
      !chatGptPage,
      "Open ChatGPT in the automation browser, or set CHATMUX_E2E_OPEN_CHATGPT=1 to open the configured ChatGPT URL."
    );

    await chatGptPage.bringToFront();
    await chatGptPage.waitForLoadState("domcontentloaded");

    let surface = await detectChatGptSurface(chatGptPage);
    await attachJson(testInfo, "chatgpt-roundtrip-surface.json", surface);

    if (MANUAL_LOGIN_MODE && surface.kind !== "ready") {
      surface = await waitForChatGptReady(chatGptPage, MANUAL_LOGIN_TIMEOUT_MS);
    }

    test.skip(
      surface.kind !== "ready",
      `ChatGPT must be signed in and ready for a roundtrip test. Detected surface: ${surface.kind}.`
    );

    await waitForChatGptReady(chatGptPage);

    const initialAssistantCount = await assistantMessages(chatGptPage).count();
    const initialUserCount = await userMessages(chatGptPage).count();
    const promptToken = `CHATMUX_E2E_${Date.now()}`;
    const timestamp = new Date().toISOString();
    const chatTitle = `Chatmux ${timestamp}`;
    const prompt = [
      `Chatmux run metadata: token=${promptToken}; timestamp=${timestamp}; project=${CHATMUX_E2E_PROJECT_NAME}; requested_title=${chatTitle}.`,
      `Reply with exactly ${promptToken} and no other words.`,
    ].join(" ");

    const { extensionPage } = chatmux;
    test.skip(
      !extensionPage,
      "Chatmux must already be installed in the attached browser session before the live ChatGPT roundtrip can run."
    );
    await extensionPage.bringToFront();
    const workspaceId = await createWorkspaceAndOpen(extensionPage);
    await attachRoundtripState(testInfo, extensionPage, chatGptPage, "pre-send", workspaceId);

    let chatState = await collectChatGptState(chatGptPage);
    await attachJson(testInfo, "chatgpt-provider-control-initial.json", chatState);

    const existingProject = chatState?.projects?.find(
      (project) => project?.title === CHATMUX_E2E_PROJECT_NAME
    );

    if (!existingProject) {
      await dispatchProviderCommand(extensionPage, {
        type: "create_provider_project",
        workspace_id: workspaceId,
        provider: "gpt",
        title: CHATMUX_E2E_PROJECT_NAME,
      });
      chatState = await waitForProjectPresence(chatGptPage, CHATMUX_E2E_PROJECT_NAME, 30_000);
    }

    const targetProject = chatState?.projects?.find(
      (project) => project?.title === CHATMUX_E2E_PROJECT_NAME
    );
    const targetProjectId = projectIdFromHref(targetProject?.href);
    expect(targetProjectId).toBeTruthy();

    await dispatchProviderCommand(extensionPage, {
      type: "select_provider_project",
      workspace_id: workspaceId,
      provider: "gpt",
      project_id: targetProjectId,
    });
    await waitForProjectSelected(chatGptPage, targetProjectId, 30_000);

    await dispatchProviderCommand(extensionPage, {
      type: "create_provider_conversation",
      workspace_id: workspaceId,
      provider: "gpt",
      project_id: targetProjectId,
      title: chatTitle,
    });
    await chatGptPage.waitForURL(/\/project$/, { timeout: 30_000 });

    await dispatchProviderCommand(extensionPage, {
      type: "set_provider_reasoning",
      workspace_id: workspaceId,
      provider: "gpt",
      reasoning_id: "thinking",
    }).catch(() => null);
    chatState = await waitForReasoningMode(chatGptPage, "thinking", 20_000);
    await attachJson(testInfo, "chatgpt-provider-control-pre-send.json", chatState);

    const sendResponse = await dispatchUiCommand(extensionPage, {
      type: "send_manual_message",
      workspace_id: workspaceId,
      targets: ["gpt"],
      text: prompt,
      approval_mode: "auto_send",
    });
    expect(sendResponse?.ok).toBeTruthy();
    await attachRoundtripState(testInfo, extensionPage, chatGptPage, "after-send-click", workspaceId);

    let assistantResult;
    let extensionAssistantText;

    try {
      await waitForPromptEcho(chatGptPage, prompt, initialUserCount);
      await attachRoundtripState(testInfo, extensionPage, chatGptPage, "after-provider-echo", workspaceId);

      assistantResult = await waitForAssistantResponse(
        chatGptPage,
        initialAssistantCount
      );
      await attachRoundtripState(testInfo, extensionPage, chatGptPage, "after-provider-response", workspaceId);

      await dispatchProviderCommand(extensionPage, {
        type: "sync_provider_conversation",
        workspace_id: workspaceId,
        provider: "gpt",
      });

      const workspaceAssistant = await waitForWorkspaceMessage(
        extensionPage,
        workspaceId,
        (message) =>
          message?.participant_id === "gpt" &&
          typeof message?.body_text === "string" &&
          message.body_text.length > 0,
        30_000
      );
      extensionAssistantText = normalizeText(workspaceAssistant.message.body_text);
      await attachRoundtripState(testInfo, extensionPage, chatGptPage, "after-workspace-ingest", workspaceId);
    } catch (error) {
      await attachRoundtripState(testInfo, extensionPage, chatGptPage, "failure", workspaceId);
      throw error;
    }

    await attachJson(testInfo, "chatgpt-roundtrip.json", {
      prompt,
      promptToken,
      timestamp,
      projectName: CHATMUX_E2E_PROJECT_NAME,
      chatTitle,
      workspaceId,
      initialUserCount,
      initialAssistantCount,
      assistantOnPage: assistantResult,
      assistantInExtension: extensionAssistantText,
    });

    expect(
      normalizeText(assistantResult.text).includes(extensionAssistantText) ||
        extensionAssistantText.includes(normalizeText(assistantResult.text)) ||
        extensionAssistantText.includes(promptToken)
    ).toBeTruthy();
  });
});
