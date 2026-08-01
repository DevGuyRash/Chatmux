"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
const path = require("node:path");

const {
  assertFreshAllowed,
  isPathInside,
  parseOptions,
  resolveCdpPort,
  resolveProfile,
  validateManifestAndMetadata,
} = require("./launch-chrome-extension");

test("default launch profile is isolated under .local/e2e", () => {
  const profile = resolveProfile({});
  assert.equal(profile.dedicated, true);
  assert.equal(isPathInside(path.join(".local", "e2e"), profile.userDataDir), true);
  assert.match(profile.userDataDir, /\.local[/\\]e2e[/\\]chrome-profile$/);
});

test("fresh launch rejects an explicit user profile", () => {
  const profile = resolveProfile({
    CHATMUX_E2E_CHROME_USER_DATA_DIR: "/tmp/user-owned-chrome",
  });
  assert.equal(profile.dedicated, false);
  assert.throws(() => assertFreshAllowed(profile), /refuses to delete/);
});

test("launcher options reject unknown switches", () => {
  assert.deepEqual(parseOptions(["--fresh"]), {
    relaunch: true,
    fresh: true,
    smoke: false,
  });
  assert.throws(() => parseOptions(["--user-data-dir=/tmp"]), /unknown launcher option/);
});

test("manifest and metadata must identify the same qualified Chrome build", () => {
  const manifest = {
    name: "Chatmux",
    manifest_version: 3,
    version: "0.1.0",
    background: { service_worker: "background.js", type: "module" },
  };
  const metadata = {
    browser: "chrome",
    version: "0.1.0",
    source_fingerprint: "fnv1a64:1234",
    artifact_fingerprint: "fnv1a64:5678",
  };
  assert.doesNotThrow(() => validateManifestAndMetadata(manifest, metadata));
  assert.throws(
    () => validateManifestAndMetadata(manifest, { ...metadata, browser: "firefox" }),
    /does not identify/
  );
});

test("CDP port validation accepts a safe explicit port", () => {
  assert.equal(resolveCdpPort({}), null);
  assert.equal(resolveCdpPort({ CHATMUX_E2E_CDP_PORT: "9223" }), 9223);
  assert.throws(
    () => resolveCdpPort({ CHATMUX_E2E_CDP_PORT: "80" }),
    /1024 through 65535/
  );
});
