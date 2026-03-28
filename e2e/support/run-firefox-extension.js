const { spawn } = require("node:child_process");
const { firefoxSupportStatus } = require("./firefox-launcher");

const support = firefoxSupportStatus();

if (!support.webExtInstalled) {
  console.error(`web-ext not found at ${support.webExtBin}`);
  process.exit(1);
}

if (!support.playwrightFirefoxBinaryPresent) {
  console.error(
    `Playwright Firefox binary not found at ${support.playwrightFirefoxBinary}`
  );
  process.exit(1);
}

const child = spawn(
  support.webExtBin,
  [
    "run",
    "--source-dir",
    support.extensionDir,
    `--firefox=${support.playwrightFirefoxBinary}`,
    "--start-url",
    "about:blank",
    "--no-input",
  ],
  {
    stdio: "inherit",
  }
);

child.on("exit", (code, signal) => {
  if (signal) {
    process.kill(process.pid, signal);
    return;
  }

  process.exit(code ?? 1);
});

