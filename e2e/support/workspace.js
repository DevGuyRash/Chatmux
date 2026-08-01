const crypto = require("node:crypto");
const { dispatchUiCommand, expect } = require("./chrome-extension");

function uniqueRunToken(prefix) {
  return `${prefix}_${crypto.randomUUID().replaceAll("-", "").slice(0, 12)}`;
}

async function dispatchUiCommandWithTimeout(page, payload, timeout = 30_000) {
  const commandType = payload?.type ?? "unknown";
  let timer;
  try {
    return await Promise.race([
      dispatchUiCommand(page, payload),
      new Promise((_, reject) => {
        timer = setTimeout(() => {
          reject(new Error(`Timed out waiting for ${commandType} after ${timeout}ms`));
        }, timeout);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

async function createWorkspaceAndOpen(page, name = uniqueRunToken("Workspace")) {
  const createResponse = await dispatchUiCommandWithTimeout(page, {
    type: "create_workspace",
    name,
  });
  expect(createResponse?.ok).toBeTruthy();

  const workspaces = createResponse?.events?.find(
    (event) => event?.type === "workspace_list"
  )?.workspaces;
  const workspaceId = workspaces?.find((workspace) => workspace.name === name)?.id;
  expect(workspaceId, `create_workspace did not return ${JSON.stringify(name)}`).toBeTruthy();

  const openResponse = await dispatchUiCommandWithTimeout(page, {
    type: "open_workspace",
    workspace_id: workspaceId,
  });
  expect(openResponse?.ok).toBeTruthy();
  return { name, workspaceId };
}

async function requestWorkspaceSnapshot(page, workspaceId) {
  const response = await dispatchUiCommandWithTimeout(page, {
    type: "request_workspace_snapshot",
    workspace_id: workspaceId,
  });
  expect(response?.ok).toBeTruthy();
  return response?.events?.find((event) => event?.type === "workspace_snapshot")?.snapshot ?? null;
}

async function deleteWorkspace(page, workspaceId) {
  const response = await dispatchUiCommandWithTimeout(page, {
    type: "delete_workspace",
    workspace_id: workspaceId,
  });
  expect(response?.ok).toBeTruthy();
}

async function seedReadyBinding(page, workspaceId, provider) {
  await page.evaluate(
    async ({ workspaceId: id, providerId }) => {
      const storeNames = [
        "workspaces",
        "bindings",
        "messages",
        "runs",
        "rounds",
        "dispatches",
        "cursors",
        "edge_policies",
        "templates",
        "export_profiles",
        "diagnostics",
      ];
      const db = await new Promise((resolve, reject) => {
        const request = indexedDB.open("chatmux", 1);
        request.onupgradeneeded = () => {
          for (const storeName of storeNames) {
            if (!request.result.objectStoreNames.contains(storeName)) {
              request.result.createObjectStore(storeName, { keyPath: "id" });
            }
          }
        };
        request.onsuccess = () => resolve(request.result);
        request.onerror = () => reject(request.error);
      });
      const conversationId = `e2e-${providerId}-${crypto.randomUUID()}`;
      const conversationRef = {
        conversation_id: conversationId,
        title: `E2E ${providerId}`,
        url: `https://example.invalid/${providerId}/${conversationId}`,
        model_label: null,
      };
      const transaction = db.transaction("bindings", "readwrite");
      transaction.objectStore("bindings").put({
        id: crypto.randomUUID(),
        workspace_id: id,
        provider_id: providerId,
        tab_id: 9001,
        window_id: 1,
        origin: "https://example.invalid",
        tab_title: `E2E ${providerId}`,
        tab_url: conversationRef.url,
        pinned: false,
        stale: false,
        bound_conversation_ref: conversationRef,
        conversation_ref: conversationRef,
        provider_control: null,
        health_state: "ready",
        capability_snapshot: {
          supports_follow_up_while_generating: false,
          can_auto_send: true,
          can_capture_full_history: true,
          can_capture_delta: true,
        },
        last_seen_at: new Date().toISOString(),
      });
      await new Promise((resolve, reject) => {
        transaction.oncomplete = resolve;
        transaction.onerror = () => reject(transaction.error);
        transaction.onabort = () => reject(transaction.error);
      });
      db.close();
    },
    { workspaceId, providerId: provider }
  );

  return await requestWorkspaceSnapshot(page, workspaceId);
}

async function seedCapturedMessage(page, workspaceId, provider, body) {
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
            body_blocks: [{ type: "paragraph", text }],
            source_binding_id: null,
            dispatch_id: null,
            raw_response_text: text,
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
  return response;
}

async function waitForWorkspaceMessage(
  page,
  workspaceId,
  predicate,
  timeout = 45_000,
  label = "workspace message"
) {
  let lastSnapshot = null;
  let matchedMessage = null;
  try {
    await expect
      .poll(
        async () => {
          lastSnapshot = await requestWorkspaceSnapshot(page, workspaceId);
          matchedMessage = (lastSnapshot?.recent_messages ?? []).find(predicate) ?? null;
          return Boolean(matchedMessage);
        },
        { message: label, timeout }
      )
      .toBe(true);
  } catch (error) {
    throw new Error(
      `${label} did not appear within ${timeout}ms. ` +
        `Last workspace snapshot: ${JSON.stringify(lastSnapshot)}`,
      { cause: error }
    );
  }
  return { message: matchedMessage, snapshot: lastSnapshot };
}

module.exports = {
  createWorkspaceAndOpen,
  deleteWorkspace,
  dispatchUiCommandWithTimeout,
  requestWorkspaceSnapshot,
  seedReadyBinding,
  seedCapturedMessage,
  uniqueRunToken,
  waitForWorkspaceMessage,
};
