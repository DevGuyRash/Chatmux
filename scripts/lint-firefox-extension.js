#!/usr/bin/env node
"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const repoRoot = path.resolve(__dirname, "..");
const binary = path.join(
  repoRoot,
  "node_modules",
  ".bin",
  process.platform === "win32" ? "web-ext.cmd" : "web-ext"
);
const result = spawnSync(
  binary,
  [
    "lint",
    "--source-dir",
    path.join(repoRoot, "extension-dist", "firefox"),
    "--output=json",
  ],
  { cwd: repoRoot, encoding: "utf8" }
);
if (result.error) {
  console.error(
    "could not start web-ext: " + result.error.message + "; run npm ci and retry"
  );
  process.exit(1);
}

let report;
try {
  report = JSON.parse(result.stdout);
} catch {
  process.stderr.write(result.stderr || "");
  console.error("web-ext did not return a JSON validation report");
  process.exit(1);
}

const evidenceDir = path.join(repoRoot, ".local");
fs.mkdirSync(evidenceDir, { recursive: true });
fs.writeFileSync(
  path.join(evidenceDir, "firefox-lint.json"),
  JSON.stringify(report, null, 2)
);

console.log(
  "Firefox manifest: " +
    report.summary.errors +
    " errors, " +
    report.summary.warnings +
    " warnings, " +
    report.summary.notices +
    " notices"
);
for (const warning of report.warnings || []) {
  console.warn(
    warning.code +
      " " +
      warning.file +
      (warning.line ? ":" + warning.line : "") +
      " - " +
      warning.message
  );
}
if (report.summary.errors > 0 || result.status !== 0) {
  for (const error of report.errors || []) {
    console.error(error.code + " " + error.file + " - " + error.message);
  }
  process.exit(1);
}
