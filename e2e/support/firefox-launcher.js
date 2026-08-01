const fs = require("node:fs/promises");
const syncFs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { spawn } = require("node:child_process");

const FIREFOX_EXTENSION_PATH = path.resolve(
  __dirname,
  "..",
  "..",
  "extension-dist",
  "firefox"
);
const FIREFOX_MANIFEST_PATH = path.join(FIREFOX_EXTENSION_PATH, "manifest.json");
const WEB_EXT_BIN = path.resolve(
  __dirname,
  "..",
  "..",
  "node_modules",
  ".bin",
  process.platform === "win32" ? "web-ext.cmd" : "web-ext"
);
const CHATMUX_FIREFOX_BINARY = process.env.CHATMUX_E2E_FIREFOX_BINARY;
const CHATMUX_FIREFOX_PROFILE = process.env.CHATMUX_E2E_FIREFOX_PROFILE;

function resolvePlaywrightFirefoxBinary() {
  if (CHATMUX_FIREFOX_BINARY) {
    return path.resolve(CHATMUX_FIREFOX_BINARY);
  }
  try {
    return require("playwright").firefox.executablePath();
  } catch {
    return path.resolve(
      os.homedir(),
      ".cache",
      "ms-playwright",
      "firefox-1509",
      "firefox",
      "firefox"
    );
  }
}

async function ensureFirefoxArtifacts() {
  await fs.access(FIREFOX_EXTENSION_PATH);
  await fs.access(FIREFOX_MANIFEST_PATH);
}

async function readFirefoxManifest() {
  const raw = await fs.readFile(FIREFOX_MANIFEST_PATH, "utf8");
  return JSON.parse(raw);
}

function firefoxSupportStatus() {
  const playwrightFirefoxBinary = resolvePlaywrightFirefoxBinary();
  const manifest = syncFs.existsSync(FIREFOX_MANIFEST_PATH)
    ? JSON.parse(syncFs.readFileSync(FIREFOX_MANIFEST_PATH, "utf8"))
    : null;
  const chatGptContentScript = manifest?.content_scripts?.find((entry) =>
    entry.matches?.includes("https://chatgpt.com/*")
  );

  return {
    extensionDir: FIREFOX_EXTENSION_PATH,
    manifestPath: FIREFOX_MANIFEST_PATH,
    webExtBin: WEB_EXT_BIN,
    webExtInstalled: syncFs.existsSync(WEB_EXT_BIN),
    playwrightFirefoxBinary,
    playwrightFirefoxBinaryPresent: syncFs.existsSync(playwrightFirefoxBinary),
    configuredFirefoxProfile: CHATMUX_FIREFOX_PROFILE
      ? path.resolve(CHATMUX_FIREFOX_PROFILE)
      : null,
    configuredFirefoxProfilePresent: CHATMUX_FIREFOX_PROFILE
      ? syncFs.existsSync(path.resolve(CHATMUX_FIREFOX_PROFILE))
      : false,
    chatGptContentScriptPresent: Boolean(chatGptContentScript),
    chatGptMatches: chatGptContentScript?.matches ?? [],
    chatGptScripts: chatGptContentScript?.js ?? [],
    blocker:
      "Firefox launching is available here through web-ext plus the bundled Playwright Firefox binary, but this harness does not yet have a stable Playwright attachment path after that launch.",
  };
}

async function smokeFirefoxExtension(timeoutMs = 12_000) {
  const support = firefoxSupportStatus();
  if (!support.webExtInstalled || !support.playwrightFirefoxBinaryPresent) {
    throw new Error(support.blocker);
  }
  await ensureFirefoxArtifacts();
  const profile = await fs.mkdtemp(path.join(os.tmpdir(), "chatmux-firefox-smoke-"));
  const output = [];
  const child = spawn(
    support.webExtBin,
    [
      "run",
      "--source-dir",
      support.extensionDir,
      `--firefox=${support.playwrightFirefoxBinary}`,
      "--firefox-profile",
      profile,
      "--profile-create-if-missing",
      "--keep-profile-changes",
      "--start-url",
      "about:debugging#/runtime/this-firefox",
      "--no-reload",
      "--no-input",
      "--verbose",
    ],
    {
      detached: process.platform !== "win32",
      env: { ...process.env, MOZ_HEADLESS: "1" },
      stdio: ["ignore", "pipe", "pipe"],
    }
  );
  child.stdout.on("data", (chunk) => output.push(String(chunk)));
  child.stderr.on("data", (chunk) => output.push(String(chunk)));

  let exit = null;
  child.once("exit", (code, signal) => {
    exit = { code, signal };
  });
  try {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline && exit === null) {
      await new Promise((resolve) => setTimeout(resolve, 200));
    }
    if (exit !== null) {
      throw new Error(
        `Firefox exited before the extension smoke window completed: ${JSON.stringify(exit)}\n${output.join("")}`
      );
    }
    const log = output.join("");
    if (!/install|addon|extension/i.test(log)) {
      throw new Error(`web-ext did not report an extension installation:\n${log}`);
    }
    return { log, profile };
  } finally {
    if (exit === null) {
      try {
        if (process.platform === "win32") {
          child.kill("SIGTERM");
        } else {
          process.kill(-child.pid, "SIGTERM");
        }
      } catch {}
    }
    await new Promise((resolve) => setTimeout(resolve, 500));
    await fs.rm(profile, { recursive: true, force: true }).catch(() => {});
  }
}

module.exports = {
  ensureFirefoxArtifacts,
  firefoxSupportStatus,
  readFirefoxManifest,
  resolvePlaywrightFirefoxBinary,
  smokeFirefoxExtension,
};
