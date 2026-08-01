#!/usr/bin/env node
"use strict";

const fs = require("node:fs/promises");
const syncFs = require("node:fs");
const path = require("node:path");
const { randomUUID } = require("node:crypto");
const { spawnSync } = require("node:child_process");
const { chromium } = require("playwright");

const REPO_ROOT = path.resolve(__dirname, "..");
const LOCAL_ROOT = path.join(REPO_ROOT, ".local", "e2e");
const DEFAULT_PROFILE = path.join(LOCAL_ROOT, "chrome-profile");
const OWNER_FILE = path.join(LOCAL_ROOT, "chrome-launcher.json");
const EXTENSION_DIR = path.join(REPO_ROOT, "extension-dist", "chrome");
const PROFILE_LOCK_FILES = [
  "SingletonLock",
  "SingletonCookie",
  "SingletonSocket",
  "lock",
  ".parentlock",
];
const PROVIDER_URLS = [
  "https://chatgpt.com/",
  "https://gemini.google.com/",
  "https://grok.com/",
  "https://claude.ai/",
];

function resolveCdpPort(env = process.env) {
  const raw = env.CHATMUX_E2E_CDP_PORT;
  if (!raw) {
    return null;
  }
  const port = Number(raw);
  if (!Number.isInteger(port) || port < 1024 || port > 65535) {
    throw new Error("CHATMUX_E2E_CDP_PORT must be an integer from 1024 through 65535");
  }
  return port;
}

function parseOptions(argv) {
  const allowed = new Set(["--relaunch", "--fresh", "--smoke"]);
  for (const argument of argv) {
    if (!allowed.has(argument)) {
      throw new Error(
        "unknown launcher option " + argument + "; choose --relaunch, --fresh, or --smoke"
      );
    }
  }
  return {
    relaunch: argv.includes("--relaunch") || argv.includes("--fresh"),
    fresh: argv.includes("--fresh"),
    smoke: argv.includes("--smoke"),
  };
}

function isPathInside(root, candidate) {
  const relative = path.relative(path.resolve(root), path.resolve(candidate));
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

function resolveProfile(env = process.env) {
  const configured = env.CHATMUX_E2E_CHROME_USER_DATA_DIR;
  const userDataDir = configured ? path.resolve(configured) : DEFAULT_PROFILE;
  return {
    userDataDir,
    profileDirectory: env.CHATMUX_E2E_CHROME_PROFILE_DIRECTORY || null,
    dedicated: isPathInside(LOCAL_ROOT, userDataDir),
  };
}

function assertFreshAllowed(profile) {
  if (!profile.dedicated) {
    throw new Error(
      "fresh launch refuses to delete an explicit Chrome profile; unset CHATMUX_E2E_CHROME_USER_DATA_DIR or use relaunch"
    );
  }
}

function verifyQualifiedBuild() {
  const result = spawnSync(
    "cargo",
    ["run", "--locked", "-p", "xtask", "--", "verify-dist", "chrome"],
    { cwd: REPO_ROOT, env: process.env, stdio: "inherit" }
  );
  if (result.error || result.status !== 0) {
    throw new Error(
      "the staged Chrome extension is missing or stale; run npm run prelaunch and retry"
    );
  }
}

async function validateDiskBuild() {
  const manifest = JSON.parse(
    await fs.readFile(path.join(EXTENSION_DIR, "manifest.json"), "utf8")
  );
  const metadata = JSON.parse(
    await fs.readFile(path.join(EXTENSION_DIR, "build-metadata.json"), "utf8")
  );
  validateManifestAndMetadata(manifest, metadata);
  return { manifest, metadata };
}

function validateManifestAndMetadata(manifest, metadata) {
  if (
    manifest.name !== "Chatmux" ||
    manifest.manifest_version !== 3 ||
    manifest.background?.service_worker !== "background.js" ||
    manifest.background?.type !== "module"
  ) {
    throw new Error(
      "staged Chrome manifest is not the expected Chatmux MV3 service-worker package"
    );
  }
  if (
    metadata.browser !== "chrome" ||
    metadata.version !== manifest.version ||
    !String(metadata.source_fingerprint || "").startsWith("fnv1a64:") ||
    !String(metadata.artifact_fingerprint || "").startsWith("fnv1a64:")
  ) {
    throw new Error(
      "staged Chrome build metadata does not identify the manifest and qualified artifacts"
    );
  }
}

async function findProfileLocks(profile) {
  const roots = [profile.userDataDir];
  if (profile.profileDirectory) {
    roots.push(path.join(profile.userDataDir, profile.profileDirectory));
  }
  const found = [];
  for (const root of roots) {
    for (const filename of PROFILE_LOCK_FILES) {
      const candidate = path.join(root, filename);
      if (await fs.lstat(candidate).then(() => true).catch(() => false)) {
        found.push(candidate);
      }
    }
  }
  return found;
}

async function readOwner() {
  try {
    return JSON.parse(await fs.readFile(OWNER_FILE, "utf8"));
  } catch (error) {
    if (error.code === "ENOENT" || error.name === "SyntaxError") {
      return null;
    }
    throw error;
  }
}

function processAlive(pid) {
  if (!Number.isInteger(pid) || pid <= 0) {
    return false;
  }
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

async function ownerStillMatches(owner) {
  if (!owner || owner.repoRoot !== REPO_ROOT || !processAlive(owner.pid)) {
    return false;
  }
  const procCommand = "/proc/" + owner.pid + "/cmdline";
  if (!syncFs.existsSync(procCommand)) {
    return true;
  }
  const command = await fs.readFile(procCommand, "utf8").catch(() => "");
  return command.includes("launch-chrome-extension.js");
}

async function waitForExit(pid, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (!processAlive(pid)) {
      return;
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(
    "the previous Chatmux launcher did not exit; close its browser and retry"
  );
}

async function handlePreviousOwner(options) {
  const owner = await readOwner();
  if (!(await ownerStillMatches(owner))) {
    await fs.rm(OWNER_FILE, { force: true });
    return;
  }
  if (!options.relaunch) {
    throw new Error(
      "a Chatmux Chrome launcher is already running; use npm run relaunch or close it first"
    );
  }
  process.kill(owner.pid, "SIGTERM");
  await waitForExit(owner.pid, 10_000);
  await fs.rm(OWNER_FILE, { force: true });
}

async function claimOwner(profile) {
  await fs.mkdir(LOCAL_ROOT, { recursive: true });
  const record = {
    pid: process.pid,
    token: randomUUID(),
    repoRoot: REPO_ROOT,
    userDataDir: profile.userDataDir,
  };
  const handle = await fs.open(OWNER_FILE, "wx");
  await handle.writeFile(JSON.stringify(record, null, 2));
  await handle.close();
  return record;
}

async function releaseOwner(record) {
  const current = await readOwner();
  if (current?.token === record.token) {
    await fs.rm(OWNER_FILE, { force: true });
  }
}

async function readJsonPage(context, url) {
  const page = await context.newPage();
  try {
    await page.goto(url, { waitUntil: "domcontentloaded", timeout: 10_000 });
    return JSON.parse(await page.locator("body").innerText());
  } finally {
    await page.close().catch(() => {});
  }
}

async function discoverChatmuxWorker(context, expectedManifest) {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    for (const worker of context.serviceWorkers()) {
      const parsed = new URL(worker.url());
      if (
        parsed.protocol !== "chrome-extension:" ||
        !parsed.pathname.endsWith("/background.js")
      ) {
        continue;
      }
      const extensionId = parsed.host;
      const manifest = await readJsonPage(
        context,
        "chrome-extension://" + extensionId + "/manifest.json"
      ).catch(() => null);
      if (
        manifest?.name === "Chatmux" &&
        manifest.version === expectedManifest.version &&
        manifest.background?.service_worker === "background.js"
      ) {
        return { worker, extensionId, manifest };
      }
    }
    await context
      .waitForEvent("serviceworker", {
        timeout: Math.min(1_000, Math.max(1, deadline - Date.now())),
      })
      .catch(() => {});
  }
  throw new Error(
    "Chatmux service worker was not discovered within 30 seconds; inspect chrome://extensions for load errors"
  );
}

function attachDiagnostics(target, label) {
  target.on("console", (message) => {
    if (message.type() === "error" || message.type() === "warning") {
      console.error("[" + label + ":" + message.type() + "] " + message.text());
    }
  });
  if (typeof target.on === "function" && label === "ui") {
    target.on("pageerror", (error) => {
      console.error("[ui:pageerror] " + error.message);
    });
  }
}

async function openAndValidateUi(context, extensionId) {
  const page = await context.newPage();
  attachDiagnostics(page, "ui");
  const extensionUrl =
    "chrome-extension://" + extensionId + "/ui/index.html";
  await page.goto(extensionUrl, { waitUntil: "domcontentloaded", timeout: 20_000 });
  await page.setViewportSize({ width: 1600, height: 1000 });
  if ((await page.title()) !== "Chatmux") {
    throw new Error("Chatmux extension page loaded with an unexpected title");
  }
  await page
    .getByRole("navigation", { name: "Main navigation" })
    .waitFor({ state: "visible", timeout: 20_000 });
  await page.bringToFront();
  return page;
}

async function launch(argv = process.argv.slice(2)) {
  const options = parseOptions(argv);
  const profile = resolveProfile();
  const cdpPort = resolveCdpPort();
  if (options.fresh) {
    assertFreshAllowed(profile);
  }
  await handlePreviousOwner(options);
  if (options.fresh) {
    await fs.rm(profile.userDataDir, { recursive: true, force: true });
  }

  verifyQualifiedBuild();
  const diskBuild = await validateDiskBuild();
  const locks = await findProfileLocks(profile);
  if (locks.length > 0) {
    throw new Error(
      "the requested Chrome profile is locked; close Chrome or choose the dedicated profile. Locked files: " +
        locks.join(", ")
    );
  }

  await fs.mkdir(profile.userDataDir, { recursive: true });
  const owner = await claimOwner(profile);
  let context;
  let closing = false;
  const close = async () => {
    if (closing) {
      return;
    }
    closing = true;
    await context?.close().catch(() => {});
    await releaseOwner(owner).catch(() => {});
  };

  try {
    context = await chromium.launchPersistentContext(profile.userDataDir, {
      headless: process.env.CHATMUX_HEADLESS === "1",
      viewport: { width: 1600, height: 1000 },
      channel: process.env.CHATMUX_E2E_CHROME_CHANNEL || undefined,
      executablePath:
        process.env.CHATMUX_E2E_CHROME_EXECUTABLE_PATH || undefined,
      args: [
        "--disable-extensions-except=" + EXTENSION_DIR,
        "--load-extension=" + EXTENSION_DIR,
        ...(cdpPort
          ? [
              "--remote-debugging-port=" + cdpPort,
              "--remote-allow-origins=*",
            ]
          : []),
        ...(profile.profileDirectory
          ? ["--profile-directory=" + profile.profileDirectory]
          : []),
      ],
    });
    context.on("serviceworker", (worker) => attachDiagnostics(worker, "worker"));
    const discovered = await discoverChatmuxWorker(context, diskBuild.manifest);
    attachDiagnostics(discovered.worker, "worker");
    const page = await openAndValidateUi(context, discovered.extensionId);

    if (process.env.CHATMUX_E2E_OPEN_PROVIDERS === "1") {
      for (const url of PROVIDER_URLS) {
        await context.newPage().then((providerPage) =>
          providerPage.goto(url, { waitUntil: "domcontentloaded" }).catch(() => {})
        );
      }
      await page.bringToFront();
    }

    console.log(
      "Chatmux " +
        discovered.manifest.version +
        " loaded from " +
        EXTENSION_DIR +
        " with profile " +
        profile.userDataDir
    );
    console.log(
      "Extension URL: chrome-extension://" +
        discovered.extensionId +
        "/ui/index.html"
    );
    if (cdpPort) {
      console.log("CDP URL: http://127.0.0.1:" + cdpPort);
    }

    if (options.smoke) {
      await close();
      return;
    }
    process.once("SIGINT", () => close().finally(() => process.exit(0)));
    process.once("SIGTERM", () => close().finally(() => process.exit(0)));
    await new Promise((resolve) => context.once("close", resolve));
    await releaseOwner(owner);
  } catch (error) {
    await close();
    throw error;
  }
}

if (require.main === module) {
  launch().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}

module.exports = {
  assertFreshAllowed,
  discoverChatmuxWorker,
  findProfileLocks,
  isPathInside,
  launch,
  parseOptions,
  resolveCdpPort,
  resolveProfile,
  validateManifestAndMetadata,
};
