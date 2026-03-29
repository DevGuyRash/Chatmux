const {
  dispatchUiCommand,
  expect,
  test,
} = require("../support/chrome-extension");
const {
  assistantMessages,
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

  await page
    .getByRole("navigation", { name: "Main navigation" })
    .getByRole("button", { name: "Active Workspace" })
    .click();
  await expect(page.getByRole("log", { name: "Message log" })).toBeVisible({
    timeout: 15_000,
  });
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
      !process.env.CHATMUX_E2E_CHROME_USER_DATA_DIR,
      "Set CHATMUX_E2E_CHROME_USER_DATA_DIR to a signed-in Chrome or Chromium profile for the live ChatGPT roundtrip."
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
    await testInfo.attach("chatgpt-roundtrip-surface.json", {
      body: Buffer.from(JSON.stringify(surface, null, 2)),
      contentType: "application/json",
    });

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
    const prompt = `Reply with exactly ${promptToken} and no other words.`;

    const { extensionPage } = chatmux;
    await extensionPage.bringToFront();
    await createWorkspaceAndOpen(extensionPage);
    await leaveOnlyChatGptSelected(extensionPage);

    const composer = extensionPage.getByPlaceholder("Type a message…");
    await composer.fill(prompt);
    await extensionPage.getByRole("button", { name: "Send" }).click();

    await expect(extensionPage.getByRole("log", { name: "Message log" })).toContainText(prompt);

    await waitForPromptEcho(chatGptPage, prompt, initialUserCount);
    const assistantResult = await waitForAssistantResponse(
      chatGptPage,
      initialAssistantCount
    );
    const extensionAssistantText = await waitForExtensionAssistantMessage(extensionPage);

    await testInfo.attach("chatgpt-roundtrip.json", {
      body: Buffer.from(
        JSON.stringify(
          {
            prompt,
            promptToken,
            assistantOnPage: assistantResult,
            assistantInExtension: extensionAssistantText,
          },
          null,
          2
        )
      ),
      contentType: "application/json",
    });

    expect(
      normalizeText(assistantResult.text).includes(extensionAssistantText) ||
        extensionAssistantText.includes(normalizeText(assistantResult.text)) ||
        extensionAssistantText.includes(promptToken)
    ).toBeTruthy();
  });
});
