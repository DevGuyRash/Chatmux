const { expect, test } = require("playwright/test");
const {
  ensureFirefoxArtifacts,
  firefoxSupportStatus,
  readFirefoxManifest,
} = require("../support/firefox-launcher");

test.describe("Chatmux Firefox launcher", () => {
  test("documents the guarded Firefox launcher path", async ({}, testInfo) => {
    await ensureFirefoxArtifacts();

    const manifest = await readFirefoxManifest();
    const support = firefoxSupportStatus();

    await testInfo.attach("firefox-support.json", {
      body: Buffer.from(JSON.stringify({ manifest, support }, null, 2)),
      contentType: "application/json",
    });

    expect(manifest.browser_specific_settings.gecko.id).toBe(
      "chatmux@example.invalid"
    );
    expect(manifest.sidebar_action.default_panel).toBe("ui/index.html");
    expect(support.chatGptContentScriptPresent).toBeTruthy();
    expect(support.chatGptMatches).toContain("https://chatgpt.com/*");
    expect(support.chatGptScripts).toContain("content-gpt.js");
    if (support.configuredFirefoxProfile) {
      expect(support.configuredFirefoxProfilePresent).toBeTruthy();
    }
    expect(
      support.manifestPath.endsWith("extension-dist/firefox/manifest.json")
    ).toBeTruthy();

    test.skip(
      process.env.CHATMUX_E2E_FIREFOX !== "1",
      `${support.blocker} Set CHATMUX_E2E_FIREFOX=1 after installing dependencies if you want this spec to validate launcher prerequisites in your environment.`
    );

    expect(support.webExtInstalled).toBeTruthy();
    expect(support.playwrightFirefoxBinaryPresent).toBeTruthy();
  });
});
