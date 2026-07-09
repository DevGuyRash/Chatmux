const { expect, test } = require("../support/chrome-extension");
const {
  detectChatGptSurface,
  findChatGptPage,
  openChatGptPage,
} = require("../support/providers/chatgpt");

test.describe("ChatGPT DOM anchors", () => {
  test("observes ChatGPT DOM anchors when a ChatGPT tab is available", async ({
    chatmux,
  }, testInfo) => {
    const shouldOpenChatGpt = process.env.CHATMUX_E2E_OPEN_CHATGPT === "1";
    let page = findChatGptPage(chatmux.context);

    if (!page && shouldOpenChatGpt) {
      page = await openChatGptPage(chatmux.context);
    }

    test.skip(
      !page,
      "Open ChatGPT in the automation browser, or set CHATMUX_E2E_OPEN_CHATGPT=1 to create a temp-profile tab."
    );

    await page.bringToFront();
    await page.waitForLoadState("domcontentloaded");

    const surface = await detectChatGptSurface(page);
    await testInfo.attach("chatgpt-surface.json", {
      body: Buffer.from(JSON.stringify(surface, null, 2)),
      contentType: "application/json",
    });

    expect(
      ["ready", "login-required", "rate-limited", "challenge", "transcript-only"]
        .includes(surface.kind)
    ).toBeTruthy();

    if (surface.kind === "ready") {
      const composer = page.locator(surface.inputSelector).filter({ visible: true }).first();
      await expect(composer).toBeVisible();

      await composer.click();
      await page.keyboard.type("CHATMUX_DOM_ANCHOR_PROBE");

      const sendSelector = surface.sendSelector ?? "button[data-testid='send-button']";
      await expect(page.locator(sendSelector).filter({ visible: true }).first()).toBeVisible();

      await page.keyboard.press("Control+A");
      await page.keyboard.press("Backspace");
      return;
    }

    if (surface.kind === "login-required") {
      await expect(page.locator(surface.loginSelector).filter({ visible: true }).first()).toBeVisible();
      return;
    }

    if (surface.kind === "rate-limited") {
      await expect(page.locator(surface.rateLimitSelector).filter({ visible: true }).first()).toBeVisible();
      return;
    }

    if (surface.kind === "challenge") {
      await expect(page.locator(surface.challengeSelector).filter({ visible: true }).first()).toBeVisible();
      return;
    }

    await expect(page.locator(surface.transcriptSelector).filter({ visible: true }).first()).toBeVisible();
  });
});
