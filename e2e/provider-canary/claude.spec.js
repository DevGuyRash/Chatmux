const { expect, test } = require("../support/chrome-extension");
const support = require("../support/providers/claude");
const { providerPageForCanary } = require("../support/provider-canary");

test.describe("Claude selector canary", () => {
  test("ready composer, send action, and transcript targets resolve without sending", async ({
    chatmux,
  }, testInfo) => {
    let page;
    await test.step("locate the signed-in Claude surface", async () => {
      page = await providerPageForCanary(support, chatmux);
    });
    await test.step("classify the page and verify semantic targets", async () => {
      await support.assertReadyCanary(page, testInfo, expect);
    });
  });
});
