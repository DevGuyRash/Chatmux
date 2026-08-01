const fs = require("node:fs/promises");
const path = require("node:path");
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
const auditDir = path.resolve(__dirname, "..", "..", ".local", "audit", "chatmux-current");

async function capture(page, name) {
  const file = path.join(auditDir, name);
  // `animations: "disabled"` finishes in-flight CSS transitions before the
  // shutter fires. Without it a capture taken straight after a nav click lands
  // mid-transition, showing the outgoing destination still tinted and the
  // incoming one not yet arrived — which reads in the screenshot as two active
  // nav items and sends whoever reviews these images hunting a bug that is not
  // in the product.
  await page.screenshot({ path: file, fullPage: false, animations: "disabled" });
  return file;
}

async function assertNoProductErrors(chatmux, stage) {
  await chatmux.extensionPage.waitForTimeout(100);
  const errors = chatmux.browserDiagnostics
    .report()
    .entries
    .filter((entry) => entry.level === "error");
  expect(errors, `browser errors ${stage}`).toEqual([]);
}

test("capture the complete primary flow at full-tab and compact widths", async ({ chatmux }, testInfo) => {
  const page = chatmux.extensionPage;
  const nav = page.getByRole("navigation", { name: "Main navigation" });
  let workspaceId = null;
  await fs.rm(auditDir, { recursive: true, force: true });
  await fs.mkdir(auditDir, { recursive: true });

  try {
    await expect(page.getByText("No workspaces yet", { exact: true })).toBeVisible();
    await capture(page, "01-workspace-empty-dark.png");

    const workspace = await createWorkspaceAndOpen(
      page,
      uniqueRunToken("Visual audit")
    );
    workspaceId = workspace.workspaceId;
    await nav.getByRole("button", { name: "Active Workspace", exact: true }).click();
    await expect(page.getByRole("button", { name: "Back to workspace list", exact: true })).toBeVisible();
    await capture(page, "02-active-workspace-dark.png");

    await page.getByRole("button", { name: "Providers", exact: true }).click();
    await expect(page.getByText("Provider Settings", { exact: true })).toBeVisible();
    await capture(page, "03-provider-bindings-panel.png");
    await page.getByRole("button", { name: "Close panel", exact: true }).click();

    await page.getByRole("button", { name: "Start run", exact: true }).click();
    await expect(page.getByText("Configure run", { exact: true })).toBeVisible();
    await capture(page, "04-run-configuration.png");
    await page.getByRole("dialog").getByRole("button", { name: "Cancel", exact: true }).click();

    await page.getByRole("button", { name: "Export workspace", exact: true }).click();
    await expect(page.getByText("Export workspace", { exact: true })).toBeVisible();
    await capture(page, "05-export-dialog.png");
    await page.getByRole("dialog").press("Escape");

    await nav.getByRole("button", { name: "Routing", exact: true }).click();
    await capture(page, "06-routing.png");
    await nav.getByRole("button", { name: "Templates", exact: true }).click();
    await capture(page, "07-templates.png");
    await nav.getByRole("button", { name: "Pinned Summaries", exact: true }).click();
    await capture(page, "08-pinned-summaries.png");
    await nav.getByRole("button", { name: "Diagnostics", exact: true }).click();
    await capture(page, "09-diagnostics.png");
    await nav.getByRole("button", { name: "Settings", exact: true }).click();
    await capture(page, "10-settings-dark.png");

    await page.getByRole("radio", { name: "Light", exact: true }).click();
    await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
    await capture(page, "11-settings-light.png");
    await assertNoProductErrors(chatmux, "before compact resize");

    await page.setViewportSize({ width: 420, height: 820 });
    await expect(page.getByRole("navigation", { name: "Sidebar navigation" })).toBeVisible();
    await assertNoProductErrors(chatmux, "after compact resize");
    await expect(page.getByRole("heading", { name: "Settings", exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Workspaces", exact: true }).click();
    await page.getByText(workspace.name, { exact: true }).click();
    await expect(page.getByRole("button", { name: "Back to workspace list", exact: true })).toBeVisible();
    await capture(page, "12-active-workspace-compact-light.png");

    await page.setViewportSize({ width: 1600, height: 1000 });
    await expect(page.getByRole("navigation", { name: "Main navigation" })).toBeVisible();
    await assertNoProductErrors(chatmux, "after expanded resize");
    await expect(page.getByRole("button", { name: "Back to workspace list", exact: true })).toBeVisible();
    await capture(page, "13-active-workspace-expanded-light.png");

    await page.setViewportSize({ width: 420, height: 820 });
    await expect(page.getByRole("button", { name: "Back to workspace list", exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Back to workspace list", exact: true }).click();
    await expect(page.getByText(workspace.name, { exact: true })).toBeVisible();

    for (const filename of await fs.readdir(auditDir)) {
      await testInfo.attach(filename, {
        path: path.join(auditDir, filename),
        contentType: "image/png",
      });
    }
  } finally {
    if (workspaceId) {
      await deleteWorkspace(page, workspaceId).catch(() => {});
    }
  }
});
