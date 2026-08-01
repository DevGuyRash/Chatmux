const { expect, test } = require("playwright/test");
const {
  ensureFirefoxArtifacts,
  firefoxSupportStatus,
  readFirefoxManifest,
  smokeFirefoxExtension,
} = require("../support/firefox-launcher");

const PROVIDER_CONTENT_SCRIPTS = [
  { origin: "https://chatgpt.com/*", script: "content-gpt.js" },
  { origin: "https://gemini.google.com/*", script: "content-gemini.js" },
  { origin: "https://grok.com/*", script: "content-grok.js" },
  { origin: "https://claude.ai/*", script: "content-claude.js" },
];

test.describe("Chatmux Firefox package contract", () => {
  test("the staged sidebar package declares all four provider content scripts", async ({}, testInfo) => {
    await ensureFirefoxArtifacts();
    const manifest = await readFirefoxManifest();
    const support = firefoxSupportStatus();

    await testInfo.attach("firefox-package-contract.json", {
      body: Buffer.from(JSON.stringify({ manifest, support }, null, 2)),
      contentType: "application/json",
    });

    expect(manifest.browser_specific_settings.gecko.id).toBe(
      "chatmux@example.invalid"
    );
    expect(manifest.sidebar_action.default_panel).toBe("ui/index.html");

    for (const expected of PROVIDER_CONTENT_SCRIPTS) {
      const entry = manifest.content_scripts.find((candidate) =>
        candidate.matches?.includes(expected.origin)
      );
      expect(
        entry,
        `Firefox manifest is missing ${expected.origin}`
      ).toBeTruthy();
      expect(entry.js).toContain(expected.script);
    }
  });

  test("the optional local Firefox launcher prerequisites are available", async ({}, testInfo) => {
    const support = firefoxSupportStatus();
    await testInfo.attach("firefox-launcher-support.json", {
      body: Buffer.from(JSON.stringify(support, null, 2)),
      contentType: "application/json",
    });

    test.skip(
      process.env.CHATMUX_E2E_FIREFOX !== "1",
      `${support.blocker} Set CHATMUX_E2E_FIREFOX=1 to require launcher prerequisites.`
    );

    if (support.configuredFirefoxProfile) {
      expect(support.configuredFirefoxProfilePresent).toBeTruthy();
    }
    expect(support.webExtInstalled).toBeTruthy();
    expect(support.playwrightFirefoxBinaryPresent).toBeTruthy();
    const smoke = await smokeFirefoxExtension();
    await testInfo.attach("firefox-temporary-install.log", {
      body: Buffer.from(smoke.log),
      contentType: "text/plain",
    });
  });
});
