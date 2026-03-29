const {
  dispatchUiCommand,
  expect,
  test,
} = require("../support/chrome-extension");

async function createWorkspaceThroughBridge(page, name) {
  const response = await dispatchUiCommand(page, {
    type: "create_workspace",
    name,
  });
  expect(response?.ok).toBeTruthy();
}

test.describe("Chatmux shell navigation", () => {
  test("creates and filters workspaces through the extension bridge", async ({
    chatmux,
  }) => {
    const { extensionPage: page } = chatmux;
    const activeFilter = page.getByRole("radio", { name: "Active" });
    const archivedFilter = page.getByRole("radio", { name: "Archived" });
    const createdWorkspace = page.locator("button.workspace-row").first();

    await expect(
      page.getByRole("button", { name: "Create Workspace" })
    ).toBeVisible();
    await expect(page.getByText("No workspaces yet")).toBeVisible();

    await createWorkspaceThroughBridge(page, "Workspace 1");
    await expect(createdWorkspace).toBeVisible();

    await archivedFilter.click();
    await expect(page.getByText("No workspaces yet")).toBeVisible();

    await activeFilter.click();
    await expect(createdWorkspace).toBeVisible();
  });

  test("renders the mounted Routing, Templates, Diagnostics, Settings, and Active Workspace screens", async ({
    chatmux,
  }) => {
    const { extensionPage: page } = chatmux;
    const nav = page.getByRole("navigation", { name: "Main navigation" });

    await nav.getByRole("button", { name: "Routing" }).click();
    await expect(
      page.getByText("Select an edge to configure its routing policy.")
    ).toBeVisible();

    await nav.getByRole("button", { name: "Templates" }).click();
    await expect(page.getByText("Select a template to edit")).toBeVisible();
    await expect(page.getByText("Built-in Templates")).toBeVisible();
    await expect(page.getByText("Custom Templates")).toBeVisible();

    await nav.getByRole("button", { name: "Diagnostics" }).click();
    await expect(page.getByText("All clear")).toBeVisible();
    await expect(
      page.getByText("No diagnostic events have been recorded.")
    ).toBeVisible();

    await nav.getByRole("button", { name: "Settings" }).click();
    await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Timing Defaults" })
    ).toBeVisible();
    await expect(page.getByRole("heading", { name: "Storage" })).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Automation Safety" })
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Keyboard Shortcuts" })
    ).toBeVisible();

    await nav.getByRole("button", { name: "Active Workspace" }).click();
    await expect(
      page.getByText("Select a workspace to view its conversation state.")
    ).toBeVisible();
  });
});
