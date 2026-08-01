const fs = require("node:fs/promises");
const {
  chromeExtensionBuildState,
  expect,
  test,
} = require("../support/chrome-extension");
const {
  createWorkspaceAndOpen,
  deleteWorkspace,
  dispatchUiCommandWithTimeout,
  uniqueRunToken,
} = require("../support/workspace");

const buildState = chromeExtensionBuildState();
test.skip(!buildState.readyForShellTests, buildState.blocker);

async function openActiveWorkspace(page) {
  await page
    .getByRole("navigation", { name: "Main navigation" })
    .getByRole("button", { name: "Active Workspace", exact: true })
    .click();
  await expect(
    page.getByRole("button", { name: "Back to workspace list", exact: true })
  ).toBeVisible();
}

test.describe("Operator workflows", () => {
  test("search, filters, saved export profiles, download, and copy operate on canonical data", async ({
    chatmux,
  }) => {
    const page = chatmux.extensionPage;
    let workspaceId = null;
    const needle = uniqueRunToken("evidence needle");

    try {
      const workspace = await createWorkspaceAndOpen(
        page,
        uniqueRunToken("Export workspace")
      );
      workspaceId = workspace.workspaceId;
      const arranged = await dispatchUiCommandWithTimeout(page, {
        type: "send_manual_message",
        workspace_id: workspaceId,
        targets: ["gpt"],
        text: `A canonical user message with ${needle}`,
        approval_mode: "draft_only",
        parent_message_id: null,
      });
      expect(arranged?.ok).toBeTruthy();
      await openActiveWorkspace(page);

      await test.step("search and role/provider filters narrow the visible log", async () => {
        await page
          .getByRole("button", { name: "Search and filter messages", exact: true })
          .click();
        await page.getByPlaceholder("Search messages…").fill(needle);
        await expect(page.getByText(`A canonical user message with ${needle}`, { exact: true })).toBeVisible();
        await expect(page.getByText("1 of 1", { exact: true })).toBeVisible();

        await page.getByRole("button", { name: "Toggle filters", exact: true }).click();
        await page.getByRole("combobox", { name: "Filter by provider" }).selectOption("user");
        await page.getByRole("combobox", { name: "Filter by role" }).selectOption("user");
        await expect(page.getByText("1 of 1", { exact: true })).toBeVisible();
        await page.getByRole("button", { name: "Clear filters", exact: true }).click();
      });

      await test.step("save and reuse an exact JSON export profile", async () => {
        await page.getByRole("button", { name: "Export workspace", exact: true }).click();
        const dialog = page.getByRole("dialog");
        await expect(dialog.getByText("Export workspace", { exact: true })).toBeVisible();
        await dialog.getByLabel("Format", { exact: true }).selectOption("json");
        await expect(dialog.getByLabel("Format", { exact: true })).toHaveValue("json");
        await dialog.getByLabel("Save this configuration", { exact: true }).fill("Evidence JSON");
        await dialog.getByRole("button", { name: "Save profile", exact: true }).click();
        await expect(dialog.getByLabel("Saved profile")).toContainText("Evidence JSON");
        await expect.poll(async () => {
          const snapshot = await dispatchUiCommandWithTimeout(page, {
            type: "request_workspace_snapshot",
            workspace_id: workspaceId,
          });
          return snapshot.events
            .find((event) => event.type === "workspace_snapshot")
            ?.snapshot?.export_profiles
            ?.find((profile) => profile.name === "Evidence JSON")
            ?.format;
        }).toBe("json");
        await expect(dialog.getByLabel("Format", { exact: true })).toHaveValue("json");

        const downloadPromise = page.waitForEvent("download");
        await dialog.getByRole("button", { name: "Download file", exact: true }).click();
        const download = await downloadPromise;
        expect(download.suggestedFilename()).toMatch(/\.json$/);
        const filePath = await download.path();
        expect(filePath).toBeTruthy();
        const downloaded = await fs.readFile(filePath, "utf8");
        expect(JSON.parse(downloaded).messages.some((message) => message.body_text.includes(needle))).toBe(true);

        await page.getByRole("button", { name: "Export workspace", exact: true }).click();
        const copyDialog = page.getByRole("dialog");
        await copyDialog.getByLabel("Saved profile").selectOption({ label: "Evidence JSON" });
        const clipboardObserved = await page.evaluate(() => {
          globalThis.__chatmuxClipboardWrites = [];
          const clipboard = navigator.clipboard;
          const originalWriteText = clipboard?.writeText?.bind(clipboard);
          if (!clipboard || !originalWriteText) {
            return false;
          }
          try {
            Object.defineProperty(clipboard, "writeText", {
              configurable: true,
              value: async (text) => {
                globalThis.__chatmuxClipboardWrites.push(text);
                return await originalWriteText(text);
              },
            });
            return true;
          } catch (_error) {
            return false;
          }
        });
        expect(clipboardObserved).toBe(true);
        await copyDialog.getByRole("button", { name: "Copy rendered export", exact: true }).click();
        await expect(
          page.locator('[aria-label="Notifications"]')
            .getByText("Rendered export copied to the clipboard.", { exact: true })
        ).toBeVisible();
        const copied = await page.evaluate(() => globalThis.__chatmuxClipboardWrites.at(-1));
        expect(JSON.parse(copied).messages.some((message) => message.body_text.includes(needle))).toBe(true);
      });
    } finally {
      if (workspaceId) {
        await deleteWorkspace(page, workspaceId).catch(() => {});
      }
    }
  });

  test("custom templates are created, edited, persisted, and deleted from the GUI", async ({
    chatmux,
  }) => {
    const page = chatmux.extensionPage;
    let workspaceId = null;
    try {
      const workspace = await createWorkspaceAndOpen(
        page,
        uniqueRunToken("Template workspace")
      );
      workspaceId = workspace.workspaceId;
      const nav = page.getByRole("navigation", { name: "Main navigation" });
      await nav.getByRole("button", { name: "Templates", exact: true }).click();
      await expect(page.getByText("Neutral · Wrapped XML", { exact: true })).toBeVisible();

      await page.getByRole("button", { name: "+ Create", exact: true }).click();
      await page.getByPlaceholder("Template name").fill("Evidence package");
      await page.getByPlaceholder("Template content…").fill("{{provider_codename}} says:\n{{body}}");
      await page.getByRole("button", { name: "Save", exact: true }).click();
      await expect(page.getByText("Evidence package", { exact: true })).toBeVisible();

      await page.getByText("Evidence package", { exact: true }).click();
      await expect(page.getByPlaceholder("Template content…")).toHaveValue("{{provider_codename}} says:\n{{body}}");
      await page.getByRole("button", { name: "Delete", exact: true }).click();
      await expect(page.getByText("Evidence package", { exact: true })).toBeHidden();
    } finally {
      if (workspaceId) {
        await deleteWorkspace(page, workspaceId).catch(() => {});
      }
    }
  });

  test("the global kill switch visibly blocks and restores all composer sends", async ({
    chatmux,
  }) => {
    const page = chatmux.extensionPage;
    let workspaceId = null;
    try {
      const workspace = await createWorkspaceAndOpen(
        page,
        uniqueRunToken("Safety workspace")
      );
      workspaceId = workspace.workspaceId;
      await openActiveWorkspace(page);

      const kill = page.getByRole("button", {
        name: "Toggle global kill switch",
        exact: true,
      });
      await kill.click();
      await expect(kill).toHaveAttribute("aria-pressed", "true");
      await expect(page.getByText("Halted", { exact: true })).toBeVisible();
      await expect(page.getByText("Kill switch active — sending is disabled.", { exact: true })).toBeVisible();

      await kill.click();
      await expect(kill).toHaveAttribute("aria-pressed", "false");
      await expect(page.getByText("Halted", { exact: true })).toBeHidden();
    } finally {
      if (workspaceId) {
        await deleteWorkspace(page, workspaceId).catch(() => {});
      }
    }
  });

  test("a moderated pause lets the user edit, skip, inject, and resume the next round", async ({
    chatmux,
  }) => {
    const page = chatmux.extensionPage;
    let workspaceId = null;
    try {
      const workspace = await createWorkspaceAndOpen(
        page,
        uniqueRunToken("Moderated workspace")
      );
      workspaceId = workspace.workspaceId;

      for (const [provider, body] of [
        ["gpt", "GPT captured evidence"],
        ["claude", "Claude captured evidence"],
      ]) {
        const response = await page.evaluate(
          async ({ workspaceId: id, providerId, text }) => {
            const runtime = globalThis.browser ?? globalThis.chrome;
            return await runtime.runtime.sendMessage({
              channel: "chatmux_adapter_event",
              workspaceId: id,
              payload: {
                type: "messages_captured",
                provider: providerId,
                messages: [{
                  id: crypto.randomUUID(),
                  workspace_id: id,
                  participant_id: providerId,
                  role: "assistant",
                  round: 1,
                  parent_message_id: null,
                  child_message_ids: [],
                  branch_index: null,
                  timestamp: new Date().toISOString(),
                  body_text: text,
                  body_blocks: [],
                  source_binding_id: null,
                  dispatch_id: null,
                  raw_response_text: null,
                  network_capture: null,
                  tags: [],
                  capture_confidence: "certain",
                }],
              },
            });
          },
          { workspaceId, providerId: provider, text: body }
        );
        expect(response?.ok).toBeTruthy();
      }

      const started = await dispatchUiCommandWithTimeout(page, {
        type: "start_configured_run",
        workspace_id: workspaceId,
        configuration: {
          mode: "roundtable",
          participants: ["gpt", "claude"],
          moderator: null,
          relay_order: [],
          barrier_policy: { type: "wait_for_all" },
          timing_policy: {
            per_provider_generation_timeout_secs: 120,
            per_provider_cooldown_secs: 0,
            inter_round_delay_secs: 0,
            jitter_percent: 0,
            max_concurrent_sends: 2,
            max_rounds: 2,
            global_run_timeout_secs: 600,
            exponential_backoff_base_secs: 1,
          },
          stop_policy: {
            stop_on_max_rounds: true,
            stop_on_manual_pause: true,
            stop_on_sentinel_phrase: null,
            repeated_provider_failure_limit: null,
            repeated_timeout_limit: null,
            stagnation_window: null,
            require_approval_between_rounds: true,
          },
          require_review_between_rounds: true,
        },
      });
      expect(started?.ok).toBeTruthy();
      const runId = started.events.find((event) => event.type === "run_updated")?.run?.id;
      expect(runId).toBeTruthy();
      await openActiveWorkspace(page);
      await expect(page.getByText("Paused", { exact: true })).toBeVisible({ timeout: 20_000 });

      await page
        .getByRole("button", { name: "Review and edit next-round packages", exact: true })
        .click();
      const dialog = page.getByRole("dialog", { name: "Review next round", exact: true });
      await expect(dialog).toBeVisible();
      await expect(dialog.getByText("Round 2 preview", { exact: false })).toBeVisible();
      await dialog.getByRole("textbox", { name: "Package for ChatGPT" }).fill("GUI edited package");
      await dialog
        .getByText("Claude", { exact: true })
        .locator("xpath=ancestor::article")
        .getByLabel("Skip this round", { exact: true })
        .check();
      await dialog.getByLabel("Add a user message before the next round").fill("GUI injected instruction");

      await dialog.getByRole("button", { name: "Resume with changes", exact: true }).click();
      await expect(dialog).toBeHidden();
      await expect.poll(async () => {
        const response = await dispatchUiCommandWithTimeout(page, {
          type: "request_run_ledger",
          run_id: runId,
        });
        const ledger = response.events.find((event) => event.type === "run_ledger_snapshot")?.ledger;
        return {
          edited: ledger?.dispatches?.some((dispatch) =>
            dispatch.round_number === 2
            && dispatch.target_participant_id === "gpt"
            && dispatch.rendered_payload === "GUI edited package"
          ),
          skipped: ledger?.dispatches?.some((dispatch) =>
            dispatch.round_number === 2
            && dispatch.target_participant_id === "claude"
            && dispatch.outcome === "skipped"
          ),
        };
      }, { timeout: 20_000 }).toEqual({ edited: true, skipped: true });
      await expect(page.getByText("GUI injected instruction", { exact: true })).toBeVisible();
    } finally {
      if (workspaceId) {
        await deleteWorkspace(page, workspaceId).catch(() => {});
      }
    }
  });
});
