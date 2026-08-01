const {
  chromeExtensionBuildState,
  expect,
  test,
} = require("../support/chrome-extension");
const {
  createWorkspaceAndOpen,
  deleteWorkspace,
  uniqueRunToken,
} = require("../support/workspace");

const buildState = chromeExtensionBuildState();
test.skip(!buildState.readyForShellTests, buildState.blocker);

test.describe("Run controls", () => {
  test("start, pause, resume, step, and stop expose the expected GUI state", async ({
    chatmux,
  }) => {
    const page = chatmux.extensionPage;
    let workspaceId = null;

    try {
      await test.step("arrange an isolated active workspace", async () => {
        const workspace = await createWorkspaceAndOpen(
          page,
          uniqueRunToken("Run workspace")
        );
        workspaceId = workspace.workspaceId;
        await page
          .getByRole("navigation", { name: "Main navigation" })
          .getByRole("button", { name: "Active Workspace", exact: true })
          .click();
        await expect(
          page.getByRole("button", { name: "Back to workspace list", exact: true })
        ).toBeVisible();
      });

      await test.step("start the run", async () => {
        await page.getByRole("button", { name: /start run/i }).click();
        const dialog = page.getByRole("dialog");
        await expect(dialog.getByText("Configure run", { exact: true })).toBeVisible();
        await dialog.getByRole("button", { name: /^Roundtable / }).click();
        await dialog.getByRole("button", { name: "Start run", exact: true }).click();
        await expect(page.getByText("Running", { exact: true })).toBeVisible();
        await expect(page.getByRole("button", { name: "Pause run", exact: true })).toBeVisible();
        await expect(page.getByText("Round 1 / 3", { exact: true })).toBeVisible();
      });

      await test.step("pause and resume the run", async () => {
        await page.getByRole("button", { name: "Pause run", exact: true }).click();
        await expect(page.getByText("Paused", { exact: true })).toBeVisible();
        await page.getByRole("button", { name: "Resume run", exact: true }).click();
        await expect(page.getByText("Running", { exact: true })).toBeVisible();
      });

      await test.step("advance one round and stop", async () => {
        await page
          .getByRole("button", { name: "Advance one run step", exact: true })
          .click();
        await expect(page.getByText("Round 2 / 3", { exact: true })).toBeVisible();
        await page.getByRole("button", { name: "Stop run", exact: true }).click();
        await expect(page.getByText("Completed", { exact: true })).toBeVisible();
        await expect(
          page.getByRole("button", { name: "Start a new run", exact: true })
        ).toBeVisible();
      });
    } finally {
      if (workspaceId) {
        await deleteWorkspace(page, workspaceId).catch(() => {});
      }
    }
  });
});
