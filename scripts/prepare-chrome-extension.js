#!/usr/bin/env node
"use strict";

const path = require("node:path");
const { spawnSync } = require("node:child_process");

const REPO_ROOT = path.resolve(__dirname, "..");

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: REPO_ROOT,
    env: process.env,
    stdio: "inherit",
  });
  if (result.error) {
    throw new Error(
      "could not run " + command + ": " + result.error.message + "; install the required build tool and retry"
    );
  }
  if (result.status !== 0) {
    throw new Error(
      command + " " + args.join(" ") + " failed with exit code " + result.status
    );
  }
}

function prepare() {
  run("just", ["dist-chrome"]);
  run("just", ["verify-dist-chrome"]);
}

if (require.main === module) {
  try {
    prepare();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

module.exports = { prepare, run };
