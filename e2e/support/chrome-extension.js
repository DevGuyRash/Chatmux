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
const CHATMUX_CHROME_CHANNEL = process.env.CHATMUX_E2E_CHROME_CHANNEL;
const CHATMUX_CHROME_EXECUTABLE_PATH = process.env.CHATMUX_E2E_CHROME_EXECUTABLE_PATH;
const CHATMUX_CHROME_PROFILE_DIRECTORY = process.env.CHATMUX_E2E_CHROME_PROFILE_DIRECTORY;
const CHATMUX_CHROME_USER_DATA_DIR = process.env.CHATMUX_E2E_CHROME_USER_DATA_DIR;

const PROFILE_LOCK_FILES = [
  "SingletonLock",
  "SingletonCookie",
  "SingletonSocket",
  "lock",
  ".parentlock",
];

async function ensureChromeExtensionArtifacts() {
  await fs.access(CHROME_EXTENSION_PATH);
  await fs.access(path.join(CHROME_EXTENSION_PATH, "manifest.json"));
  await fs.access(path.join(CHROME_EXTENSION_PATH, EXTENSION_ENTRY_PATH));
}

async function ensureProfileNotLocked(userDataDir, profileDirectory) {
  const roots = [userDataDir];
  if (profileDirectory) {
    roots.push(path.join(userDataDir, "Default"));
    roots.push(path.join(userDataDir, profileDirectory));
  }

  const lockedFiles = [];

  for (const root of roots) {
    for (const file of PROFILE_LOCK_FILES) {
      const candidate = path.join(root, file);
      if (await fs.stat(candidate).then(() => true).catch(() => false)) {
        lockedFiles.push(candidate);
      }
    }
  }

  if (lockedFiles.length > 0) {
    throw new Error(
      `The requested browser profile is already locked. Close the existing browser session first or choose a different profile. Locked files: ${lockedFiles.join(", ")}`
    );
  }
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

async function dispatchUiCommand(page, payload) {
  return await page.evaluate(async (messagePayload) => {
    const runtimeApi = globalThis.browser ?? globalThis.chrome;
    return await runtimeApi.runtime.sendMessage({
      channel: "chatmux_ui_command",
      payload: messagePayload,
    });
  }, payload);
}

const test = base.extend({
  chatmux: async ({}, use, testInfo) => {
    await ensureChromeExtensionArtifacts();

    const shouldReuseProfile = Boolean(CHATMUX_CHROME_USER_DATA_DIR);
    const userDataDir = shouldReuseProfile
      ? path.resolve(CHATMUX_CHROME_USER_DATA_DIR)
      : await fs.mkdtemp(path.join(os.tmpdir(), "chatmux-playwright-"));

    if (shouldReuseProfile) {
      await ensureProfileNotLocked(userDataDir, CHATMUX_CHROME_PROFILE_DIRECTORY);
    }

    const context = await chromium.launchPersistentContext(userDataDir, {
      headless: process.env.CHATMUX_HEADLESS === "1",
      viewport: DEFAULT_VIEWPORT,
      channel: CHATMUX_CHROME_CHANNEL || undefined,
      executablePath: CHATMUX_CHROME_EXECUTABLE_PATH || undefined,
      args: [
        `--disable-extensions-except=${CHROME_EXTENSION_PATH}`,
        `--load-extension=${CHROME_EXTENSION_PATH}`,
        ...(CHATMUX_CHROME_PROFILE_DIRECTORY
          ? [`--profile-directory=${CHATMUX_CHROME_PROFILE_DIRECTORY}`]
          : []),
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

    if (!shouldReuseProfile && process.env.CHATMUX_KEEP_PROFILE !== "1") {
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
  dispatchUiCommand,
  expect,
  openExtensionPage,
  test,
};
