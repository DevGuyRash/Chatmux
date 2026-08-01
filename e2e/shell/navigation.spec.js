const {
  chromeExtensionBuildState,
  dispatchUiCommand,
  expect,
  test,
} = require("../support/chrome-extension");
const {
  createWorkspaceAndOpen,
  deleteWorkspace,
  uniqueRunToken,
} = require("../support/workspace");
const { workspaceRow } = require("../support/app-locators");

const buildState = chromeExtensionBuildState();
test.skip(!buildState.readyForShellTests, buildState.blocker);

async function createWorkspaceThroughBridge(page, name) {
  const response = await dispatchUiCommand(page, {
    type: "create_workspace",
    name,
  });
  expect(response?.ok).toBeTruthy();
  return response?.events
    ?.find((event) => event?.type === "workspace_list")
    ?.workspaces?.find((workspace) => workspace.name === name)?.id;
}

test.describe("Chatmux shell navigation", () => {
  test("a bridge-arranged workspace can be filtered through the GUI", async ({
    chatmux,
  }) => {
    const page = chatmux.extensionPage;
    const name = uniqueRunToken("Filter workspace");

    await test.step("arrange one active workspace", async () => {
      await expect(page.getByText("No workspaces yet", { exact: true })).toBeVisible();
      const workspaceId = await createWorkspaceThroughBridge(page, name);
      expect(workspaceId).toBeTruthy();
      await expect(workspaceRow(page, name)).toBeVisible();
    });

    await test.step("switch between archived and active filters", async () => {
      await page.getByRole("radio", { name: "Archived", exact: true }).click();
      // The Archived tab has its own empty state. It must not claim there are
      // no workspaces (one exists, just not here) and must not offer a create
      // button, which would silently add an active workspace the user cannot
      // see from this tab.
      await expect(
        page.getByText("No archived workspaces", { exact: true })
      ).toBeVisible();
      await expect(
        page.getByRole("button", { name: "Create workspace", exact: true })
      ).toHaveCount(0);
      await page.getByRole("radio", { name: "Active", exact: true }).click();
      await expect(workspaceRow(page, name)).toBeVisible();
    });
  });

  test("the main navigation reaches each mounted product screen", async ({
    chatmux,
  }) => {
    const page = chatmux.extensionPage;
    const nav = page.getByRole("navigation", { name: "Main navigation" });
    let workspaceId = null;
    let workspaceName = null;

    try {
      await test.step("arrange an active workspace", async () => {
        const workspace = await createWorkspaceAndOpen(
          page,
          uniqueRunToken("Navigation workspace")
        );
        workspaceId = workspace.workspaceId;
        workspaceName = workspace.name;
        await expect(workspaceRow(page, workspaceName)).toBeVisible();
      });

      await test.step("open Routing", async () => {
        await nav.getByRole("button", { name: "Routing", exact: true }).click();
        await expect(
          page.getByText("Select an edge to configure its routing policy.", {
            exact: true,
          })
        ).toBeVisible();
      });

      await test.step("open Templates", async () => {
        await nav.getByRole("button", { name: "Templates", exact: true }).click();
        await expect(page.getByText("Built-in Templates", { exact: true })).toBeVisible();
        await expect(page.getByText("Custom Templates", { exact: true })).toBeVisible();
      });

      await test.step("open Diagnostics", async () => {
        await nav.getByRole("button", { name: "Diagnostics", exact: true }).click();
        await expect(page.getByText("All clear", { exact: true })).toBeVisible();
      });

      await test.step("open Settings", async () => {
        await nav.getByRole("button", { name: "Settings", exact: true }).click();
        await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
        for (const section of [
          "Appearance",
          "Timing Defaults",
          "Storage",
          "Automation Safety",
          "Keyboard Shortcuts",
        ]) {
          await expect(page.getByRole("heading", { name: section })).toBeVisible();
        }
      });

      await test.step("return to the active workspace", async () => {
        await nav
          .getByRole("button", { name: "Active Workspace", exact: true })
          .click();
        await expect(
          page.getByRole("button", { name: "Back to workspace list", exact: true })
        ).toBeVisible();
      });
    } finally {
      if (workspaceId) {
        await deleteWorkspace(page, workspaceId).catch(() => {});
      }
    }
  });

  test("creating a workspace from the full-tab shell opens it immediately", async ({
    chatmux,
  }) => {
    const page = chatmux.extensionPage;

    await test.step("create from the global action", async () => {
      await expect(page.getByText("No workspaces yet", { exact: true })).toBeVisible();
      await page.getByRole("button", { name: "+ New Workspace", exact: true }).click();
    });

    await test.step("verify the new active workspace", async () => {
      await expect(
        page.getByRole("button", { name: "Back to workspace list", exact: true })
      ).toBeVisible();
      await expect(page.getByRole("button", { name: "Providers" })).toBeVisible();
    });
  });
});
