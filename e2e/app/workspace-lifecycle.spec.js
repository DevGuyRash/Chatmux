const {
  chromeExtensionBuildState,
  expect,
  test,
} = require("../support/chrome-extension");
const { workspaceItem, workspaceRow } = require("../support/app-locators");

const buildState = chromeExtensionBuildState();
test.skip(!buildState.readyForShellTests, buildState.blocker);

test.describe("Workspace lifecycle", () => {
  test("a user can create, rename, duplicate, archive, restore, and delete from the GUI", async ({
    chatmux,
  }) => {
    const page = chatmux.extensionPage;

    await test.step("create a workspace from the empty state", async () => {
      await expect(page.getByText("No workspaces yet", { exact: true })).toBeVisible();
      await page.getByRole("button", { name: "Create workspace", exact: true }).click();
      await expect(
        page.getByRole("button", { name: "Back to workspace list", exact: true })
      ).toBeVisible();
    });

    await test.step("rename and duplicate preserve distinct workspace identities", async () => {
      await page
        .getByRole("button", { name: "Back to workspace list", exact: true })
        .click();
      await page.getByRole("button", { name: "Rename workspace", exact: true }).click();
      const renameDialog = page.getByRole("dialog", { name: "Rename workspace", exact: true });
      await renameDialog.getByLabel("Workspace name", { exact: true }).fill("Research Council");
      await renameDialog.getByRole("button", { name: "Save name", exact: true }).click();
      await expect(workspaceRow(page, "Research Council")).toBeVisible();

      await page.getByRole("button", { name: "Duplicate workspace", exact: true }).click();
      await expect(workspaceRow(page, "Research Council Copy")).toBeVisible();
    });

    await test.step("archive and restore move the workspace between explicit filters", async () => {
      const originalRow = workspaceItem(page, "Research Council");
      await originalRow.getByRole("button", { name: "Archive workspace", exact: true }).click();
      await expect(originalRow).toBeHidden();

      await page.getByRole("radio", { name: "Archived", exact: true }).click();
      const archivedRow = workspaceItem(page, "Research Council");
      await expect(archivedRow).toBeVisible();
      await archivedRow.getByRole("button", { name: "Restore workspace", exact: true }).click();
      await expect(archivedRow).toBeHidden();
      await page.getByRole("radio", { name: "Active", exact: true }).click();
      await expect(workspaceRow(page, "Research Council")).toBeVisible();
    });

    await test.step("canceling deletion preserves the workspace", async () => {
      await expect(workspaceRow(page, "Research Council")).toBeVisible();
      await workspaceItem(page, "Research Council")
        .getByRole("button", { name: "Delete workspace", exact: true })
        .click();

      const dialog = page.getByRole("dialog", {
        name: "Delete workspace confirmation",
        exact: true,
      });
      await expect(dialog).toBeVisible();
      await dialog.getByRole("button", { name: "Cancel", exact: true }).click();
      await expect(dialog).toBeHidden();
      await expect(workspaceRow(page, "Research Council")).toBeVisible();
    });

    await test.step("confirming deletion removes exactly the chosen workspace", async () => {
      await workspaceItem(page, "Research Council")
        .getByRole("button", { name: "Delete workspace", exact: true })
        .click();
      const dialog = page.getByRole("dialog", {
        name: "Delete workspace confirmation",
        exact: true,
      });
      // The confirm button carries the same name as the action that opened the
      // dialog, so the label the user committed to is the label they clicked.
      await dialog
        .getByRole("button", { name: "Delete workspace", exact: true })
        .click();
      await expect(dialog).toBeHidden();
      await expect(workspaceRow(page, "Research Council")).toBeHidden();
      await expect(workspaceRow(page, "Research Council Copy")).toBeVisible();
    });
  });
});
