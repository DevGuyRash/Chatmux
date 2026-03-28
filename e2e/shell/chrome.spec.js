const { expect, test } = require("../support/chrome-extension");

test.describe("Chatmux Chrome shell", () => {
  test("renders the packaged shell in a deterministic full-tab layout", async ({
    chatmux,
  }) => {
    const { extensionPage } = chatmux;
    const navigation = extensionPage.getByRole("navigation", {
      name: "Main navigation",
    });

    await expect(extensionPage).toHaveTitle("Chatmux");
    await expect(navigation).toBeVisible();
    await expect(navigation.getByRole("button", { name: "Workspaces" })).toBeVisible();
    await expect(
      navigation.getByRole("button", { name: "Active Workspace" })
    ).toBeVisible();
    await expect(navigation.getByRole("button", { name: "Routing" })).toBeVisible();
    await expect(navigation.getByRole("button", { name: "Templates" })).toBeVisible();
    await expect(navigation.getByRole("button", { name: "Diagnostics" })).toBeVisible();
    await expect(navigation.getByRole("button", { name: "Settings" })).toBeVisible();
    await expect(
      extensionPage.getByRole("button", { name: "+ New Workspace" })
    ).toBeVisible();
    await expect(
      extensionPage.getByRole("radiogroup", { name: "Filter workspaces" })
    ).toBeVisible();
    await expect(
      extensionPage.getByRole("radio", { name: "Active" })
    ).toBeVisible();
    await expect(
      extensionPage.getByRole("radio", { name: "Archived" })
    ).toBeVisible();

    const emptyState = extensionPage.getByText("No workspaces yet");
    if (await emptyState.isVisible().catch(() => false)) {
      await expect(emptyState).toBeVisible();
      await expect(
        extensionPage.getByRole("button", { name: "Create Workspace" })
      ).toBeVisible();
      return;
    }

    await expect(
      extensionPage.locator("button.workspace-row").first()
    ).toBeVisible();
  });
});
