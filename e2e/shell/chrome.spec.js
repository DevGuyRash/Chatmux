const {
  chromeExtensionBuildState,
  expect,
  test,
} = require("../support/chrome-extension");

const buildState = chromeExtensionBuildState();
test.skip(!buildState.readyForShellTests, buildState.blocker);

test.describe("Chatmux Chrome shell", () => {
  test("renders the packaged shell in a deterministic full-tab layout", async ({
    chatmux,
  }) => {
    const { extensionPage } = chatmux;
    const navigation = extensionPage.getByRole("navigation", {
      name: "Main navigation",
    });

    await test.step("verify the extension identity and primary navigation", async () => {
      await expect(extensionPage).toHaveTitle("Chatmux");
      await expect(navigation).toBeVisible();
      for (const destination of [
        "Workspaces",
        "Active Workspace",
        "Routing",
        "Templates",
        "Diagnostics",
        "Settings",
      ]) {
        await expect(
          navigation.getByRole("button", { name: destination, exact: true })
        ).toBeVisible();
      }
    });

    await test.step("verify the workspace-list controls", async () => {
      await expect(
        extensionPage.getByRole("button", { name: "+ New Workspace", exact: true })
      ).toBeVisible();
      await expect(
        extensionPage.getByRole("radiogroup", { name: "Filter workspaces" })
      ).toBeVisible();
      await expect(extensionPage.getByRole("radio", { name: "Active" })).toBeVisible();
      await expect(extensionPage.getByRole("radio", { name: "Archived" })).toBeVisible();
    });
  });
});
