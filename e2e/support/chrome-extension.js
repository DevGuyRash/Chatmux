const fs = require("node:fs/promises");
const os = require("node:os");
const path = require("node:path");
const { test: base, expect, chromium } = require("playwright/test");

const CHROME_EXTENSION_PATH = path.resolve(
  __dirname,
  "..",
  "..",
  "extension-dist",
  "chrome"
);
const EXTENSION_ENTRY_PATH = "ui/index.html";
const DEFAULT_VIEWPORT = { width: 1600, height: 1000 };

async function ensureChromeExtensionArtifacts() {
  await fs.access(CHROME_EXTENSION_PATH);
  await fs.access(path.join(CHROME_EXTENSION_PATH, "manifest.json"));
  await fs.access(path.join(CHROME_EXTENSION_PATH, EXTENSION_ENTRY_PATH));
}

async function waitForServiceWorker(context) {
  const existing = context.serviceWorkers()[0];
  if (existing) {
    return existing;
  }
  return context.waitForEvent("serviceworker", { timeout: 30_000 });
}

function extensionIdFromWorker(worker) {
  return new URL(worker.url()).host;
}

async function openExtensionPage(context, extensionId) {
  const extensionUrl = `chrome-extension://${extensionId}/${EXTENSION_ENTRY_PATH}`;
  let page = context.pages().find((candidate) => candidate.url() === extensionUrl);

  if (!page) {
    page = await context.newPage();
    await page.goto(extensionUrl, { waitUntil: "networkidle" });
  }

  await page.setViewportSize(DEFAULT_VIEWPORT);
  await page.bringToFront();
  return page;
}

const test = base.extend({
  chatmux: async ({}, use, testInfo) => {
    await ensureChromeExtensionArtifacts();

    const userDataDir = await fs.mkdtemp(
      path.join(os.tmpdir(), "chatmux-playwright-")
    );

    const context = await chromium.launchPersistentContext(userDataDir, {
      headless: process.env.CHATMUX_HEADLESS === "1",
      viewport: DEFAULT_VIEWPORT,
      args: [
        `--disable-extensions-except=${CHROME_EXTENSION_PATH}`,
        `--load-extension=${CHROME_EXTENSION_PATH}`,
      ],
    });

    const worker = await waitForServiceWorker(context);
    const extensionId = extensionIdFromWorker(worker);
    const extensionPage = await openExtensionPage(context, extensionId);

    await use({
      context,
      extensionId,
      extensionPage,
      userDataDir,
    });

    if (process.env.CHATMUX_KEEP_BROWSER !== "1") {
      await context.close().catch(() => {});
    }

    if (process.env.CHATMUX_KEEP_PROFILE !== "1") {
      await fs.rm(userDataDir, { recursive: true, force: true }).catch(() => {});
    } else {
      testInfo.annotations.push({
        type: "chatmux-profile",
        description: userDataDir,
      });
    }
  },
});

module.exports = {
  CHROME_EXTENSION_PATH,
  DEFAULT_VIEWPORT,
  ensureChromeExtensionArtifacts,
  expect,
  openExtensionPage,
  test,
};
