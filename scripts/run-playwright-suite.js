#!/usr/bin/env node
"use strict";

const path = require("node:path");
const { spawnSync } = require("node:child_process");

const allowed = new Set([
  "app",
  "provider-canary",
  "provider-live",
  "qualification",
]);
const suite = process.argv[2] || "app";
if (!allowed.has(suite)) {
  console.error(
    "unknown Playwright suite " +
      suite +
      "; choose " +
      Array.from(allowed).join(", ")
  );
  process.exit(2);
}

const binary = path.resolve(
  __dirname,
  "..",
  "node_modules",
  ".bin",
  process.platform === "win32" ? "playwright.cmd" : "playwright"
);
const result = spawnSync(binary, ["test", ...process.argv.slice(3)], {
  cwd: path.resolve(__dirname, ".."),
  env: { ...process.env, CHATMUX_E2E_SUITE: suite },
  stdio: "inherit",
});
if (result.error) {
  console.error(
    "could not start Playwright: " +
      result.error.message +
      "; run npm ci and npx playwright install first"
  );
  process.exit(1);
}
process.exit(result.status ?? 1);
