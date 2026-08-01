const path = require("node:path");

const suite = process.env.CHATMUX_E2E_SUITE || "app";
const outputRoot = path.join(__dirname, ".local");

const projectCatalog = {
  app: {
    name: "app",
    testMatch: ["shell/**/*.spec.js", "app/**/*.spec.js"],
    retries: process.env.CI ? 1 : 0,
  },
  "firefox-contract": {
    name: "firefox-contract",
    testMatch: ["firefox/**/*.spec.js"],
    retries: 0,
  },
  "provider-canary": {
    name: "provider-canary",
    testMatch: [
      "provider-canary/**/*.spec.js",
      "chatgpt/dom-anchors.spec.js",
    ],
    retries: 0,
  },
  "provider-live": {
    name: "provider-live",
    testMatch: [
      "provider-live/**/*.spec.js",
      "chatgpt/blocking.spec.js",
      "chatgpt/roundtrip.spec.js",
    ],
    retries: 0,
  },
};

const suiteProjects = {
  app: ["app", "firefox-contract"],
  "provider-canary": ["provider-canary"],
  "provider-live": ["provider-live"],
  qualification: [
    "app",
    "firefox-contract",
    "provider-canary",
    "provider-live",
  ],
};

if (!suiteProjects[suite]) {
  throw new Error(
    `Unknown CHATMUX_E2E_SUITE=${JSON.stringify(suite)}. ` +
      `Expected one of: ${Object.keys(suiteProjects).join(", ")}.`
  );
}

module.exports = {
  testDir: path.join(__dirname, "e2e"),
  testMatch: ["**/*.spec.js"],
  fullyParallel: false,
  forbidOnly: Boolean(process.env.CI),
  retries: 0,
  workers: 1,
  timeout: 90_000,
  expect: {
    timeout: 10_000,
  },
  outputDir: path.join(outputRoot, "playwright-results"),
  reporter: [
    ["list"],
    ["html", { outputFolder: path.join(outputRoot, "playwright-report"), open: "never" }],
    ...(process.env.CI
      ? [["junit", { outputFile: path.join(outputRoot, "playwright-junit.xml") }]]
      : []),
  ],
  use: {
    actionTimeout: 10_000,
    navigationTimeout: 45_000,
    locale: "en-US",
    timezoneId: "UTC",
    reducedMotion: "reduce",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: suiteProjects[suite].map((name) => projectCatalog[name]),
};
