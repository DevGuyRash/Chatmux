const {
  chromeExtensionBuildState,
  dispatchUiCommand,
  expect,
  test,
} = require("../support/chrome-extension");
const chatGpt = require("../support/providers/chatgpt");
const {
  createWorkspaceAndOpen,
  deleteWorkspace,
  dispatchUiCommandWithTimeout,
  requestWorkspaceSnapshot,
  uniqueRunToken,
} = require("../support/workspace");

const buildState = chromeExtensionBuildState();
const EXPECT_BLOCKED = process.env.CHATMUX_E2E_EXPECT_CHATGPT_BLOCKED === "1";
const BLOCKED_STATES = new Set([
  "blocked",
  "challenge",
  "login_required",
  "permission_required",
  "rate_limited",
]);

async function attachJson(testInfo, name, value) {
  await testInfo.attach(name, {
    body: Buffer.from(JSON.stringify(value, null, 2)),
    contentType: "application/json",
  });
}

test.describe("ChatGPT blocked-delivery contract", () => {
  test.skip(!buildState.readyForShellTests, buildState.blocker);
  test.skip(
    !EXPECT_BLOCKED,
    "Set CHATMUX_E2E_EXPECT_CHATGPT_BLOCKED=1 only with a deliberately blocked or logged-out ChatGPT surface."
  );

  test("a known blocked page cannot produce a delivered workspace message", async ({
    chatmux,
  }, testInfo) => {
    test.setTimeout(100_000);
    const { context, extensionPage } = chatmux;
    let workspaceId = null;

    try {
      await test.step("arrange an isolated workspace and ChatGPT page", async () => {
        const workspace = await createWorkspaceAndOpen(
          extensionPage,
          uniqueRunToken("Blocked workspace")
        );
        workspaceId = workspace.workspaceId;

        let page = chatGpt.findPage(context);
        if (!page) {
          page = await chatGpt.openPage(context);
        }
        await page.bringToFront();
        await page.waitForLoadState("domcontentloaded");

        const state = await chatGpt.classifyPageState(page);
        await attachJson(testInfo, "chatgpt-blocked-precondition.json", state);
        expect(
          BLOCKED_STATES.has(state.kind),
          `Expected a known blocked state, received ${state.kind}. Unknown and ready states are test setup failures.`
        ).toBeTruthy();
      });

      await test.step("bind the known ChatGPT tab", async () => {
        const response = await dispatchUiCommandWithTimeout(extensionPage, {
          type: "open_provider_tab",
          workspace_id: workspaceId,
          provider: "gpt",
          prefer_existing: true,
        });
        expect(response?.ok).toBeTruthy();
      });

      await test.step("attempt delivery and verify no canonical message is recorded", async () => {
        const promptToken = uniqueRunToken("CHATMUX_BLOCKED");
        const prompt = `Reply with exactly ${promptToken} and no other words.`;
        const beforeSnapshot = await requestWorkspaceSnapshot(extensionPage, workspaceId);
        const sendResponse = await dispatchUiCommand(extensionPage, {
          type: "send_manual_message",
          workspace_id: workspaceId,
          targets: ["gpt"],
          text: prompt,
          approval_mode: "auto_send",
        });
        const afterSnapshot = await requestWorkspaceSnapshot(extensionPage, workspaceId);

        await attachJson(testInfo, "blocked-send-response.json", sendResponse);
        await attachJson(testInfo, "blocked-workspace-before.json", beforeSnapshot);
        await attachJson(testInfo, "blocked-workspace-after.json", afterSnapshot);

        expect(sendResponse?.ok).toBeFalsy();
        expect(sendResponse?.error ?? "").toMatch(
          /blocked|login|required|challenge|permission|rate/i
        );
        expect(JSON.stringify(afterSnapshot)).not.toContain(promptToken);
      });
    } finally {
      if (workspaceId) {
        await deleteWorkspace(extensionPage, workspaceId).catch(() => {});
      }
    }
  });
});
