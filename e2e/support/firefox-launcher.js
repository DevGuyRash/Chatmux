const fs = require("node:fs/promises");
const syncFs = require("node:fs");
const os = require("node:os");
const path = require("node:path");

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

function resolvePlaywrightFirefoxBinary() {
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

  return {
    extensionDir: FIREFOX_EXTENSION_PATH,
    manifestPath: FIREFOX_MANIFEST_PATH,
    webExtBin: WEB_EXT_BIN,
    webExtInstalled: syncFs.existsSync(WEB_EXT_BIN),
    playwrightFirefoxBinary,
    playwrightFirefoxBinaryPresent: syncFs.existsSync(playwrightFirefoxBinary),
    blocker:
      "Firefox launching is available here through web-ext plus the bundled Playwright Firefox binary, but this harness does not yet have a stable Playwright attachment path after that launch.",
  };
}

module.exports = {
  ensureFirefoxArtifacts,
  firefoxSupportStatus,
  readFirefoxManifest,
  resolvePlaywrightFirefoxBinary,
};
