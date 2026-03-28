const path = require("node:path");

module.exports = {
  testDir: path.join(__dirname, "e2e"),
  testMatch: ["**/*.spec.js"],
  fullyParallel: false,
  forbidOnly: Boolean(process.env.CI),
  retries: 0,
  workers: 1,
  timeout: 90_000,
  reporter: [["list"]],
  use: {
    trace: "on-first-retry",
  },
};
