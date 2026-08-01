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
  seedCapturedMessage,
  seedReadyBinding,
  uniqueRunToken,
} = require("../support/workspace");

const buildState = chromeExtensionBuildState();
test.skip(!buildState.readyForShellTests, buildState.blocker);

test("picked context and notes render into the exact editable package committed by Draft", async ({
  chatmux,
}) => {
  const page = chatmux.extensionPage;
  let workspaceId = null;
  const evidence = uniqueRunToken("Claude evidence");
  const exactOverride = uniqueRunToken("Exact reviewed payload");

  try {
    const workspace = await createWorkspaceAndOpen(
      page,
      uniqueRunToken("Package preview workspace")
    );
    workspaceId = workspace.workspaceId;
    await seedReadyBinding(page, workspaceId, "gpt");
    await seedCapturedMessage(page, workspaceId, "claude", evidence);

    await page
      .getByRole("navigation", { name: "Main navigation" })
      .getByRole("button", { name: "Active Workspace", exact: true })
      .click();
    await expect(page.getByRole("button", { name: /^ChatGPT/ })).toHaveAttribute(
      "aria-pressed",
      "true"
    );

    await test.step("pick one exact context message", async () => {
      await page
        .getByRole("button", { name: "Pick messages for context", exact: true })
        .click();
      const contextCard = page.getByRole("article").filter({ hasText: evidence });
      await contextCard.getByRole("checkbox", { name: "Select message", exact: true }).check();
      await expect(page.getByText("1 selected for context", { exact: true })).toBeVisible();
      await page
        .getByRole("toolbar", { name: "Context message selection", exact: true })
        .getByRole("button", { name: "Done", exact: true })
        .click();
    });

    await test.step("add shared and target-private notes", async () => {
      await page.getByRole("button", { name: "Edit package notes", exact: true }).click();
      await page.getByLabel("Pinned package note", { exact: true }).fill("Use this evidence.");
      await page.getByLabel("Note for ChatGPT", { exact: true }).fill("Act as reviewer.");
    });

    await test.step("render and edit the exact package", async () => {
      await page.getByPlaceholder("Type a message…").fill("Produce the final critique.");
      await page
        .getByRole("button", { name: "Preview exact outbound packages", exact: true })
        .click();
      const packageEditor = page.getByLabel("Exact outbound package for ChatGPT", {
        exact: true,
      });
      await expect(packageEditor).toHaveValue(new RegExp(evidence));
      await expect(packageEditor).toHaveValue(/Produce the final critique\./);
      await expect(packageEditor).toHaveValue(/Use this evidence\./);
      await expect(packageEditor).toHaveValue(/Act as reviewer\./);
      await expect(page.locator(".package-preview").getByText(evidence, { exact: true })).toBeVisible();
      await page.getByRole("button", { name: /Remove context block/ }).click();
      await expect(packageEditor).not.toHaveValue(new RegExp(evidence));
      await expect(packageEditor).toHaveValue(/Produce the final critique\./);
      await packageEditor.fill(exactOverride);
      await expect(page.getByText("Edited", { exact: true })).toBeVisible();
    });

    await test.step("Draft commits the reviewed text without provider I/O", async () => {
      await page.getByRole("radio", { name: "Draft", exact: true }).click();
      await page.getByRole("button", { name: "Draft", exact: true }).click();
      await expect(
        page.locator('[aria-label="Notifications"]').getByText("Draft saved.", { exact: true })
      ).toBeVisible();

      await expect
        .poll(async () => {
          const snapshot = await requestWorkspaceSnapshot(page, workspaceId);
          const run = snapshot?.runs?.at(-1);
          if (!run) {
            return null;
          }
          const ledger = await dispatchUiCommandWithTimeout(page, {
            type: "request_run_ledger",
            run_id: run.id,
          });
          return ledger?.events
            ?.find((event) => event.type === "run_ledger_snapshot")
            ?.ledger?.dispatches?.[0]?.rendered_payload ?? null;
        })
        .toBe(exactOverride);
    });
  } finally {
    if (workspaceId) {
      await deleteWorkspace(page, workspaceId).catch(() => {});
    }
  }
});
