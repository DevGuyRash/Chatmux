const {
  chromeExtensionBuildState,
  expect,
  test,
} = require("../support/chrome-extension");
const {
  createWorkspaceAndOpen,
  deleteWorkspace,
  dispatchUiCommandWithTimeout,
  requestWorkspaceSnapshot,
  uniqueRunToken,
} = require("../support/workspace");

const buildState = chromeExtensionBuildState();
test.skip(!buildState.readyForShellTests, buildState.blocker);

async function openActiveWorkspace(page) {
  await page
    .getByRole("navigation", { name: "Main navigation" })
    .getByRole("button", { name: "Active Workspace", exact: true })
    .click();
  await expect(page.getByRole("button", { name: "Back to workspace list", exact: true })).toBeVisible();
}

test.describe("Advanced operator workflows", () => {
  test("saved filters, pinned summaries, route presets, and recipes persist and apply", async ({ chatmux }) => {
    const page = chatmux.extensionPage;
    const nav = page.getByRole("navigation", { name: "Main navigation" });
    let workspaceId = null;
    try {
      const workspace = await createWorkspaceAndOpen(page, uniqueRunToken("Advanced workspace"));
      workspaceId = workspace.workspaceId;
      const needle = uniqueRunToken("round evidence");
      const prepared = await dispatchUiCommandWithTimeout(page, {
        type: "send_manual_message",
        workspace_id: workspaceId,
        targets: ["gpt"],
        text: needle,
        approval_mode: "draft_only",
        parent_message_id: null,
      });
      expect(prepared?.ok).toBeTruthy();
      await openActiveWorkspace(page);

      await test.step("an advanced filter can be named, saved, cleared, and restored", async () => {
        await page.getByRole("button", { name: "Search and filter messages", exact: true }).click();
        await page.getByPlaceholder("Search messages…").fill(needle);
        await page.getByRole("button", { name: "Toggle filters", exact: true }).click();
        await page.getByRole("combobox", { name: "Filter by provider" }).selectOption("user");
        await page.getByRole("combobox", { name: "Filter by role" }).selectOption("user");
        await page.getByLabel("Minimum round", { exact: true }).fill("1");
        await page.getByLabel("Saved filter name", { exact: true }).fill("Round one evidence");
        await page.getByRole("button", { name: "Save filter", exact: true }).click();
        await expect(page.getByRole("combobox", { name: "Saved filters" })).toContainText("Round one evidence");
        await page.getByRole("button", { name: "Clear filters", exact: true }).click();
        await page.getByRole("combobox", { name: "Saved filters" }).selectOption({ label: "Round one evidence" });
        await expect(page.getByRole("combobox", { name: "Filter by provider" })).toHaveValue("user");
        await expect(page.getByLabel("Minimum round", { exact: true })).toHaveValue("1");
      });

      let summaryId;
      await test.step("a pinned summary is created from the GUI and becomes routable", async () => {
        await nav.getByRole("button", { name: "Pinned Summaries", exact: true }).click();
        await page.getByRole("button", { name: "+ Create", exact: true }).click();
        await page.getByPlaceholder("Summary name").fill("Round one compact context");
        await page.getByPlaceholder("Write a concise summary of the conversation context…").fill("The providers agreed on the durable round-one evidence.");
        await page.getByRole("button", { name: "Save", exact: true }).click();
        await expect(page.getByText("Round one compact context", { exact: true })).toBeVisible();

        const snapshot = await requestWorkspaceSnapshot(page, workspaceId);
        const summary = snapshot.recent_messages.find((message) => message.tags.includes("pinned-summary"));
        expect(summary).toBeTruthy();
        summaryId = summary.id;
      });

      await test.step("an edge policy selects the summary and a graph preset is reusable", async () => {
        await nav.getByRole("button", { name: "Routing", exact: true }).click();
        await page.getByRole("button", { name: /Edit .* to .* edge/ }).first().click();
        await page.getByRole("combobox", { name: "Catch-up rule" }).selectOption("pinned_summary");
        await page.getByRole("combobox", { name: "Pinned summary" }).selectOption(summaryId);
        await page.getByRole("button", { name: "Save edge policy", exact: true }).click();
        await expect.poll(async () => {
          const snapshot = await requestWorkspaceSnapshot(page, workspaceId);
          return snapshot.edge_policies.some((policy) =>
            policy.catch_up_policy.type === "pinned_summary" &&
            policy.catch_up_policy.summary_message_id === summaryId
          );
        }).toBe(true);

        await page.getByLabel("Route preset name", { exact: true }).fill("Compact route");
        await page.getByRole("button", { name: "Save current graph", exact: true }).click();
        await expect(page.getByRole("combobox", { name: "Saved route preset" })).toContainText("Compact route");
        await page.getByLabel("Recipe name", { exact: true }).fill("Draft critique verify");
        await page.getByRole("button", { name: "Save 4-phase recipe", exact: true }).click();
        await expect(page.getByRole("combobox", { name: "Orchestration recipe" })).toContainText("Draft critique verify");
        await page.getByRole("combobox", { name: "Orchestration recipe" }).selectOption({ label: "Draft critique verify" });
        await expect(page.getByRole("button", { name: /Run phase 1: Independent drafts/ })).toBeVisible();
        await page.getByRole("button", { name: "Next phase", exact: true }).click();
        await expect(page.getByRole("button", { name: /Run phase 2: Conflict map/ })).toBeVisible();
      });
    } finally {
      if (workspaceId) await deleteWorkspace(page, workspaceId).catch(() => {});
    }
  });

  test("workspace archives export and import through the settings GUI", async ({ chatmux }) => {
    const page = chatmux.extensionPage;
    const nav = page.getByRole("navigation", { name: "Main navigation" });
    const createdIds = [];
    try {
      const workspace = await createWorkspaceAndOpen(page, uniqueRunToken("Portable workspace"));
      createdIds.push(workspace.workspaceId);
      const archiveResponse = await dispatchUiCommandWithTimeout(page, {
        type: "export_workspace_archive",
        workspace_id: workspace.workspaceId,
      });
      expect(archiveResponse?.ok).toBeTruthy();
      const archive = archiveResponse.events.find((event) => event.type === "export_rendered")?.body;
      expect(JSON.parse(archive).schema_version).toBe(1);

      await nav.getByRole("button", { name: "Settings", exact: true }).click();
      await page.getByLabel("Workspace archive JSON", { exact: true }).fill(archive);
      await page.getByRole("button", { name: "Import workspace", exact: true }).click();
      await expect(page.locator('[aria-label="Notifications"]')).toContainText("Workspace imported as a safe paused copy.");

      const list = await dispatchUiCommandWithTimeout(page, { type: "request_workspace_list" });
      const imported = list.events.find((event) => event.type === "workspace_list").workspaces
        .find((candidate) => candidate.name.endsWith("(Imported)"));
      expect(imported).toBeTruthy();
      createdIds.push(imported.id);
      const snapshot = await requestWorkspaceSnapshot(page, imported.id);
      expect(snapshot.bindings).toEqual([]);
    } finally {
      for (const id of createdIds) await deleteWorkspace(page, id).catch(() => {});
    }
  });
});
