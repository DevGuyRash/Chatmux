const {
  dispatchUiCommand,
  expect,
  test,
} = require("../support/chrome-extension");
const chatGpt = require("../support/providers/chatgpt");
const { providerPageForCanary } = require("../support/provider-canary");
const {
  createWorkspaceAndOpen,
  deleteWorkspace,
  dispatchUiCommandWithTimeout,
  uniqueRunToken,
  waitForWorkspaceMessage,
} = require("../support/workspace");

async function attachJson(testInfo, name, value) {
  await testInfo.attach(name, {
    body: Buffer.from(JSON.stringify(value, null, 2)),
    contentType: "application/json",
  });
}

async function selectOnlyChatGpt(page) {
  for (const provider of ["ChatGPT", "Gemini", "Grok", "Claude"]) {
    const chip = page.getByRole("button", { name: provider, exact: true });
    await expect(chip).toBeVisible();
    const selected = await chip.getAttribute("aria-pressed");
    if (provider === "ChatGPT" && selected === "false") {
      await chip.click();
    }
    if (provider !== "ChatGPT" && selected !== "false") {
      await chip.click();
    }
  }

  await expect(
    page.getByRole("button", { name: "ChatGPT", exact: true })
  ).toHaveAttribute("aria-pressed", "true");
}

test.describe("ChatGPT GUI roundtrip", () => {
  test("the extension composer sends one prompt and automatically ingests the completed reply", async ({
    chatmux,
  }, testInfo) => {
    test.setTimeout(180_000);

    const { context, extensionPage } = chatmux;
    let workspaceId = null;
    let chatGptPage = null;

    expect(
      extensionPage,
      "Chatmux must be installed in the qualification browser before a live roundtrip."
    ).toBeTruthy();

    try {
      let baseline;
      const promptToken = uniqueRunToken("CHATMUX_E2E");
      const prompt = `Reply with exactly ${promptToken} and no other words.`;

      await test.step("verify a ready authenticated ChatGPT surface", async () => {
        chatGptPage = await providerPageForCanary(chatGpt, chatmux);
        const state = await chatGpt.classifyPageState(chatGptPage);
        await attachJson(testInfo, "chatgpt-roundtrip-precondition.json", state);
        expect(
          state.kind,
          `Live roundtrip requires ready ChatGPT; unknown is a selector failure, not a skip.`
        ).toBe("ready");
      });

      await test.step("arrange an isolated workspace and bind the existing ChatGPT tab", async () => {
        const workspace = await createWorkspaceAndOpen(
          extensionPage,
          uniqueRunToken("Roundtrip workspace")
        );
        workspaceId = workspace.workspaceId;

        const bindResponse = await dispatchUiCommandWithTimeout(extensionPage, {
          type: "open_provider_tab",
          workspace_id: workspaceId,
          provider: "gpt",
          prefer_existing: true,
        });
        expect(bindResponse?.ok).toBeTruthy();

        baseline = await chatGpt.collectChatGptState(chatGptPage);
        await attachJson(testInfo, "chatgpt-roundtrip-baseline.json", baseline);
      });

      await test.step("send from the product-owned GUI", async () => {
        await extensionPage.bringToFront();
        await selectOnlyChatGpt(extensionPage);

        const composer = extensionPage.getByPlaceholder("Type a message…");
        await expect(composer).toBeEditable();
        await composer.fill(prompt);
        await extensionPage.getByRole("button", { name: "Send", exact: true }).click();
        await expect(composer).toHaveValue("");
      });

      await test.step("observe the exact prompt and a completed exact provider response", async () => {
        await chatGptPage.bringToFront();
        await chatGpt.waitForPromptEcho(
          chatGptPage,
          prompt,
          baseline.userCount,
          45_000
        );
        const assistant = await chatGpt.waitForAssistantResponse(
          chatGptPage,
          promptToken,
          baseline.assistantCount,
          120_000
        );
        await attachJson(testInfo, "chatgpt-completed-response.json", assistant);
        expect(assistant.generating).toBe(false);
        expect(chatGpt.normalizeText(assistant.text)).toBe(promptToken);
      });

      await test.step("verify automatic canonical ingestion without an explicit sync command", async () => {
        const workspaceResult = await waitForWorkspaceMessage(
          extensionPage,
          workspaceId,
          (message) =>
            message?.participant_id === "gpt" &&
            chatGpt.normalizeText(message?.body_text) === promptToken,
          60_000,
          "automatic ChatGPT response ingestion"
        );
        await attachJson(
          testInfo,
          "chatgpt-ingested-workspace-message.json",
          workspaceResult.message
        );

        await extensionPage.bringToFront();
        await expect(
          extensionPage
            .getByRole("article", { name: /ChatGPT at/i })
            .filter({ hasText: promptToken })
        ).toHaveCount(1);
        expect(chatGpt.normalizeText(workspaceResult.message.body_text)).toBe(promptToken);
      });
    } finally {
      if (workspaceId && extensionPage) {
        await deleteWorkspace(extensionPage, workspaceId).catch(() => {});
      }
      if (chatGptPage) {
        await chatGptPage.bringToFront().catch(() => {});
      }
    }
  });
});
