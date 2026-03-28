const { expect, test } = require("../support/chrome-extension");

test.describe("Chatmux shell navigation", () => {
  test("creates and filters workspaces from the empty state", async ({
    chatmux,
  }) => {
    const { extensionPage: page } = chatmux;
    const activeFilter = page.getByRole("radio", { name: "Active" });
    const archivedFilter = page.getByRole("radio", { name: "Archived" });
    const createEmptyStateWorkspace = page.getByRole("button", {
      name: "Create Workspace",
    });
    const createdWorkspace = page.getByRole("button", { name: "Workspace 1" });

    await expect(createEmptyStateWorkspace).toBeVisible();
    await expect(page.getByText("No workspaces yet")).toBeVisible();

    await createEmptyStateWorkspace.click();
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
