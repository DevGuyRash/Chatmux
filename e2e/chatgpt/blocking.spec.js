const {
  chromeExtensionBuildState,
  dispatchUiCommand,
  expect,
  test,
} = require("../support/chrome-extension");
const {
  collectChatGptState,
  findChatGptPage,
} = require("../support/providers/chatgpt");

const buildState = chromeExtensionBuildState();
const SHOULD_OPEN_CHATGPT = process.env.CHATMUX_E2E_OPEN_CHATGPT === "1";
const DEFAULT_CHATGPT_URL = process.env.CHATMUX_E2E_CHATGPT_URL || "https://chatgpt.com/";

async function attachJson(testInfo, name, value) {
  await testInfo.attach(name, {
    body: Buffer.from(JSON.stringify(value, null, 2)),
    contentType: "application/json",
  });
}

async function createWorkspaceAndOpen(page) {
  const createResponse = await dispatchUiCommandWithTimeout(page, {
    type: "create_workspace",
    name: "Workspace 1",
  });
  expect(createResponse?.ok).toBeTruthy();

  const workspaces = createResponse?.events?.find(
    (event) => event?.type === "workspace_list"
  )?.workspaces;
  const workspaceId = workspaces?.[workspaces.length - 1]?.id;
  expect(workspaceId).toBeTruthy();

  const openResponse = await dispatchUiCommandWithTimeout(page, {
    type: "open_workspace",
    workspace_id: workspaceId,
  });
  expect(openResponse?.ok).toBeTruthy();

  return workspaceId;
}

async function requestWorkspaceSnapshot(page, workspaceId) {
  const response = await dispatchUiCommandWithTimeout(page, {
    type: "request_workspace_snapshot",
    workspace_id: workspaceId,
  });
  expect(response?.ok).toBeTruthy();
  return response?.events?.find((event) => event?.type === "workspace_snapshot")?.snapshot ?? null;
}

async function dispatchUiCommandWithTimeout(page, payload, timeout = 30_000) {
  const commandType = payload?.type ?? "unknown";
  let timer;
  try {
    return await Promise.race([
      dispatchUiCommand(page, payload),
      new Promise((_, reject) => {
        timer = setTimeout(() => {
          reject(new Error(`Timed out waiting for ${commandType} after ${timeout}ms`));
        }, timeout);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

async function openChatGptPageForBlocking(context) {
  const page = await context.newPage();
  await withTimeout(
    "ChatGPT navigation",
    page.goto(DEFAULT_CHATGPT_URL, {
      waitUntil: "domcontentloaded",
      timeout: 25_000,
    }),
    30_000
  );
  return page;
}

async function withTimeout(label, promise, timeout) {
  let timer;
  try {
    return await Promise.race([
      promise,
      new Promise((_, reject) => {
        timer = setTimeout(() => {
          reject(new Error(`${label} timed out after ${timeout}ms`));
        }, timeout);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

test.describe("ChatGPT blocking states", () => {
  test.skip(!buildState.readyForShellTests, buildState.blocker);
  test.skip(
    !SHOULD_OPEN_CHATGPT,
    "Set CHATMUX_E2E_OPEN_CHATGPT=1 to open ChatGPT in the temp-profile browser."
  );

  test("does not report delivery when ChatGPT is blocked", async ({ chatmux }, testInfo) => {
    test.setTimeout(100_000);

    const { context, extensionPage } = chatmux;
    console.log("[chatgpt-blocking] creating workspace");
    const workspaceId = await createWorkspaceAndOpen(extensionPage);

    console.log("[chatgpt-blocking] opening ChatGPT");
    let chatGptPage = findChatGptPage(context);
    if (!chatGptPage) {
      chatGptPage = await openChatGptPageForBlocking(context);
    }

    await chatGptPage.bringToFront();
    await chatGptPage.waitForLoadState("domcontentloaded");
    console.log(`[chatgpt-blocking] ChatGPT page open: ${chatGptPage.url()}`);

    console.log("[chatgpt-blocking] binding ChatGPT tab");
    const openResponse = await dispatchUiCommandWithTimeout(extensionPage, {
      type: "open_provider_tab",
      workspace_id: workspaceId,
      provider: "gpt",
      prefer_existing: true,
    });
    expect(openResponse?.ok).toBeTruthy();

    const promptToken = `CHATMUX_BLOCKED_${Date.now()}`;
    const prompt = `Reply with exactly ${promptToken} and no other words.`;
    const beforeSnapshot = await requestWorkspaceSnapshot(extensionPage, workspaceId);

    console.log("[chatgpt-blocking] sending blocked prompt");
    const sendResponse = await dispatchUiCommandWithTimeout(extensionPage, {
      type: "send_manual_message",
      workspace_id: workspaceId,
      targets: ["gpt"],
      text: prompt,
      approval_mode: "auto_send",
    });
    console.log(`[chatgpt-blocking] send response: ${JSON.stringify(sendResponse)}`);
    const afterSnapshot = await requestWorkspaceSnapshot(extensionPage, workspaceId);

    const chatGptState = await withTimeout(
      "Collect ChatGPT state",
      collectChatGptState(chatGptPage),
      5_000
    ).catch((error) => ({ error: error?.message ?? String(error) }));
    await attachJson(testInfo, "chatgpt-state.json", chatGptState);
    await attachJson(testInfo, "send-response.json", sendResponse);
    await attachJson(testInfo, "workspace-before.json", beforeSnapshot);
    await attachJson(testInfo, "workspace-after.json", afterSnapshot);

    expect(sendResponse?.ok).toBeFalsy();
    expect(sendResponse?.error ?? "").toMatch(/blocked|login|required|challenge|rate/i);
    expect(JSON.stringify(afterSnapshot)).not.toContain(promptToken);
  });
});
