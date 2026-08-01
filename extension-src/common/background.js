import initChatmuxCore, * as chatmuxCore from "./wasm/chatmux_core.js";
import {
  conversationRefFromUrl,
  conversationRefMatchesTarget,
  hasStableConversationTarget,
  normalizeConversationUrl,
} from "./conversation-ref.mjs";
import { observeStableResponse } from "./completion-stability.mjs";
import { isTransientSendControlError } from "./send-readiness.mjs";

const runtimeApi = globalThis.browser ?? globalThis.chrome;
const logError = (error) => console.error(error?.message ?? error);
const ADAPTER_COMMAND_TIMEOUT_MS = 20_000;
const providerUrlPatterns = {
  gpt: ["https://chat.openai.com/*", "https://chatgpt.com/*"],
  gemini: ["https://gemini.google.com/*"],
  grok: ["https://grok.com/*", "https://x.com/i/grok*"],
  claude: ["https://claude.ai/*"],
};

async function currentActiveTab() {
  if (!runtimeApi.tabs?.query) {
    return null;
  }

  const tabs = await runtimeApi.tabs.query({ active: true, currentWindow: true });
  return tabs?.[0] ?? null;
}

async function openWorkspaceSurface(tab) {
  if (runtimeApi.sidebarAction?.open) {
    await runtimeApi.sidebarAction.open();
    return;
  }

  if (runtimeApi.sidePanel?.open) {
    const resolvedTab = tab ?? await currentActiveTab();
    const target = resolvedTab?.windowId != null
      ? { windowId: resolvedTab.windowId }
      : resolvedTab?.id != null
        ? { tabId: resolvedTab.id }
        : {};
    await runtimeApi.sidePanel.open(target);
  }
}

function callbackApiResult(invoker) {
  return new Promise((resolve, reject) => {
    invoker((result) => {
      const runtimeError = globalThis.chrome?.runtime?.lastError;
      if (runtimeError) {
        reject(new Error(runtimeError.message || String(runtimeError)));
        return;
      }
      resolve(result);
    });
  });
}

async function runtimeSendMessage(message) {
  const result = runtimeApi.runtime?.sendMessage?.(message);
  if (result && typeof result.then === "function") {
    return await result;
  }

  return await callbackApiResult((done) => runtimeApi.runtime.sendMessage(message, done));
}

async function tabsQuery(queryInfo) {
  const result = runtimeApi.tabs?.query?.(queryInfo);
  if (result && typeof result.then === "function") {
    return await result;
  }

  return await callbackApiResult((done) => runtimeApi.tabs.query(queryInfo, done));
}

async function tabsGet(tabId) {
  const result = runtimeApi.tabs?.get?.(tabId);
  if (result && typeof result.then === "function") {
    return await result;
  }

  return await callbackApiResult((done) => runtimeApi.tabs.get(tabId, done));
}

async function tabsUpdate(tabId, updateProperties) {
  const result = runtimeApi.tabs?.update?.(tabId, updateProperties);
  if (result && typeof result.then === "function") {
    return await result;
  }

  return await callbackApiResult((done) => runtimeApi.tabs.update(tabId, updateProperties, done));
}

async function tabsCreate(createProperties) {
  const result = runtimeApi.tabs?.create?.(createProperties);
  if (result && typeof result.then === "function") {
    return await result;
  }

  return await callbackApiResult((done) => runtimeApi.tabs.create(createProperties, done));
}

async function sendTabMessage(tabId, message) {
  const commandType = message?.payload?.type ?? "unknown";
  const timeoutLabel = `Adapter command ${commandType} did not respond for tab ${tabId}`;
  const result = runtimeApi.tabs?.sendMessage?.(tabId, message);
  if (result && typeof result.then === "function") {
    return await withTimeout(result, timeoutLabel, ADAPTER_COMMAND_TIMEOUT_MS);
  }

  return await withTimeout(
    callbackApiResult((done) => runtimeApi.tabs.sendMessage(tabId, message, done)),
    timeoutLabel,
    ADAPTER_COMMAND_TIMEOUT_MS
  );
}

function delay(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function withTimeout(promise, label, timeoutMs) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`${label} within ${timeoutMs}ms`));
    }, timeoutMs);

    Promise.resolve(promise)
      .then((value) => {
        clearTimeout(timer);
        resolve(value);
      })
      .catch((error) => {
        clearTimeout(timer);
        reject(error);
      });
  });
}

function isNoReceiverError(error) {
  const message = error?.message ?? String(error);
  return message.includes("Receiving end does not exist")
    || message.includes("Could not establish connection")
    || message.includes("No matching message handler");
}

function ignoreNoReceiver(error) {
  if (!isNoReceiverError(error)) {
    logError(error);
  }
}

async function broadcastUiEvents(events) {
  for (const event of events ?? []) {
    await runtimeSendMessage({
      channel: "chatmux_ui_event",
      payload: event,
    }).catch(ignoreNoReceiver);
  }
}

let killSwitchActive = false;

function observeKillSwitch(events) {
  for (const event of events ?? []) {
    if (event?.type === "kill_switch_changed") {
      killSwitchActive = Boolean(event.active);
    }
    if (event?.type === "workspace_snapshot") {
      killSwitchActive = Boolean(event.snapshot?.kill_switch_active);
    }
  }
}

async function executeCoreCommand(wasmModule, command) {
  if (!wasmModule?.handle_ui_command_json) {
    throw new Error("Chatmux background core is unavailable");
  }
  const events = await wasmModule.handle_ui_command_json(JSON.stringify(command));
  observeKillSwitch(events);
  await broadcastUiEvents(events);
  return events;
}

function pickProviderPatterns(providerId) {
  switch (providerId) {
    case "gpt":
      return providerUrlPatterns.gpt;
    case "gemini":
      return providerUrlPatterns.gemini;
    case "grok":
      return providerUrlPatterns.grok;
    case "claude":
      return providerUrlPatterns.claude;
    default:
      return [];
  }
}

function bindingHasBoundTarget(binding, providerId) {
  return hasStableConversationTarget(providerId, binding?.bound_conversation_ref);
}

function mismatchDetail(binding, observedRef) {
  const target = binding?.bound_conversation_ref ?? {};
  const expected = target.conversation_id ?? normalizeConversationUrl(target.url) ?? "unknown chat";
  const actual = observedRef?.conversation_id ?? normalizeConversationUrl(observedRef?.url) ?? "unknown chat";
  return `Bound chat mismatch: expected ${expected} but tab is on ${actual}`;
}

function conversationRefEvent(events) {
  return (events ?? []).find((event) => event?.type === "conversation_ref_discovered")?.conversation_ref ?? null;
}

function blockingStateEvent(events) {
  return (events ?? []).find((event) => event?.type === "blocking_state_detected")?.blocking_state ?? null;
}

function providerDisplayName(providerId) {
  switch (providerId) {
    case "gpt":
      return "ChatGPT";
    case "gemini":
      return "Gemini";
    case "grok":
      return "Grok";
    case "claude":
      return "Claude";
    default:
      return providerId;
  }
}

function blockingStateDetail(providerId, blockingState) {
  const kind = String(blockingState?.kind ?? "blocked").replaceAll("_", " ");
  const detail = blockingState?.detail ? `: ${blockingState.detail}` : "";
  return `${providerDisplayName(providerId)} is blocked (${kind})${detail}`;
}

async function providerBlockingDetail(workspaceId, providerId, tab) {
  const result = await sendAdapterCommand(workspaceId, providerId, { type: "detect_blocking_state" }, tab);
  const blockingState = blockingStateEvent(result?.events);
  return blockingState ? blockingStateDetail(providerId, blockingState) : null;
}

function originFromUrl(url) {
  if (!url) {
    return null;
  }

  try {
    return new URL(url).origin;
  } catch (_error) {
    return null;
  }
}

function tabMatchesProvider(tab, providerId) {
  const url = String(tab?.url ?? "");
  if (!url) {
    return false;
  }

  try {
    const hostname = new URL(url).hostname;
    switch (providerId) {
      case "gpt":
        return hostname === "chat.openai.com" || hostname === "chatgpt.com";
      case "gemini":
        return hostname === "gemini.google.com";
      case "grok":
        return hostname === "grok.com" || hostname === "x.com";
      case "claude":
        return hostname === "claude.ai";
      default:
        return false;
    }
  } catch (_error) {
    return false;
  }
}

async function requestWorkspaceSnapshot(readyModule, workspaceId) {
  const events = await readyModule.handle_ui_command_json(JSON.stringify({
    type: "request_workspace_snapshot",
    workspace_id: String(workspaceId),
  }));
  const snapshotEvent = (events ?? []).find((event) => event?.type === "workspace_snapshot");
  return snapshotEvent?.snapshot ?? null;
}

function bindingForProvider(snapshot, providerId) {
  return snapshot?.bindings?.find((binding) => binding.provider_id === providerId) ?? null;
}

async function listProviderTabs(providerId, boundTabId = null) {
  const patterns = pickProviderPatterns(providerId);
  if (patterns.length === 0) {
    throw new Error(`Unsupported provider target: ${providerId}`);
  }

  const tabs = await tabsQuery({ url: patterns });
  return tabs
    .filter((tab) => tab?.id && tabMatchesProvider(tab, providerId))
    .map((tab) => {
      const conversationRef = conversationRefFromUrl(providerId, tab.url);
      return {
        _tab: tab,
        candidate: {
          tab_id: tab.id,
          window_id: tab.windowId ?? null,
          title: tab.title ?? null,
          url: tab.url ?? null,
          conversation_id: conversationRef.conversation_id,
          conversation_title: conversationRef.conversation_title ?? tab.title ?? null,
          has_stable_target: Boolean(conversationRef.conversation_id),
          is_active: Boolean(tab.active && tab.currentWindow),
          is_bound: boundTabId != null && tab.id === boundTabId,
          is_pinned: Boolean(tab.pinned),
        },
      };
    });
}

async function findProviderTab(providerId) {
  const patterns = pickProviderPatterns(providerId);
  if (patterns.length === 0) {
    throw new Error(`Unsupported provider target: ${providerId}`);
  }

  const tabs = await tabsQuery({ url: patterns });
  const preferred = tabs.find((tab) => tab.active && tab.currentWindow) ?? tabs[0];
  if (!preferred?.id) {
    throw new Error(`No open tab found for provider ${providerId}`);
  }
  return preferred;
}

async function persistBinding(readyModule, workspaceId, providerId, tab, options = {}) {
  if (!tab?.id) {
    throw new Error(`Cannot bind provider ${providerId} without a tab id`);
  }

  const pin = Boolean(options.pin);
  if (pin && runtimeApi.tabs?.update) {
    await tabsUpdate(tab.id, { pinned: true }).catch(logError);
  }

  const tabUrl = options.tab_url ?? tab.url ?? null;
  const urlConversationRef = conversationRefFromUrl(providerId, tabUrl);
  const conversationRef = {
    ...urlConversationRef,
    conversation_id: options.conversation_id ?? urlConversationRef.conversation_id,
    conversation_title: options.conversation_title ?? tab.title ?? null,
    conversation_url: options.conversation_url ?? urlConversationRef.conversation_url,
  };
  const hasStableTarget = Boolean(conversationRef.conversation_id);

  const events = await readyModule.handle_ui_command_json(JSON.stringify({
    type: "bind_provider_tab",
    workspace_id: String(workspaceId),
    provider: providerId,
    tab_id: tab.id,
    window_id: tab.windowId ?? null,
    origin: options.origin ?? originFromUrl(tabUrl),
    tab_title: options.tab_title ?? tab.title ?? null,
    tab_url: tabUrl,
    conversation_id: hasStableTarget ? conversationRef.conversation_id : null,
    conversation_title: hasStableTarget ? conversationRef.conversation_title : null,
    conversation_url: hasStableTarget ? conversationRef.conversation_url : null,
    pin,
  }));
  await broadcastUiEvents(events);
  return events;
}

function providerStartUrl(providerId) {
  switch (providerId) {
    case "gpt":
      return "https://chatgpt.com/";
    case "gemini":
      return "https://gemini.google.com/";
    case "grok":
      return "https://grok.com/";
    case "claude":
      return "https://claude.ai/";
    default:
      throw new Error(`Unsupported provider target: ${providerId}`);
  }
}

async function chooseProviderTab(providerId, binding, preferExisting) {
  if (binding?.tab_id != null) {
    try {
      const boundTab = await tabsGet(binding.tab_id);
      if (boundTab?.id && tabMatchesProvider(boundTab, providerId)) {
        return boundTab;
      }
    } catch (_error) {
      // Fall through to recovery.
    }
  }

  if (preferExisting) {
    const candidates = (await listProviderTabs(providerId, binding?.tab_id ?? null)).map((entry) => entry._tab);
    const activeTab = candidates.find((tab) => tab?.active && tab?.currentWindow);
    const chosen = activeTab ?? candidates[0];
    if (chosen?.id) {
      return chosen;
    }
  }

  return await tabsCreate({
    url: providerStartUrl(providerId),
    active: true,
  });
}

async function openAndBindProviderTab(readyModule, workspaceId, providerId, preferExisting = true) {
  const snapshot = await requestWorkspaceSnapshot(readyModule, workspaceId);
  const binding = bindingForProvider(snapshot, providerId);
  const tab = await chooseProviderTab(providerId, binding, preferExisting);
  if (!tab?.id) {
    throw new Error(`No tab available for provider ${providerId}`);
  }

  await tabsUpdate(tab.id, { active: true, pinned: true }).catch(logError);
  await persistBinding(readyModule, workspaceId, providerId, tab, {
    pin: true,
    tab_url: tab.url ?? providerStartUrl(providerId),
  });
  await sendAdapterCommand(
    workspaceId,
    providerId,
    { type: "get_conversation_ref" },
    tab
  );
  await sendAdapterCommand(
    workspaceId,
    providerId,
    { type: "get_provider_snapshot" },
    tab
  );

  return tab;
}

async function resolveBoundProviderTab(readyModule, workspaceId, providerId) {
  const snapshot = await requestWorkspaceSnapshot(readyModule, workspaceId);
  const binding = bindingForProvider(snapshot, providerId);

  if (binding?.tab_id != null) {
    try {
      const boundTab = await tabsGet(binding.tab_id);
      if (boundTab?.id && tabMatchesProvider(boundTab, providerId)) {
        return { tab: boundTab, binding, snapshot };
      }
    } catch (_error) {
      throw new Error(`Bound tab is no longer available for provider ${providerId}; rebind required`);
    }
    throw new Error(`Bound tab is no longer a valid ${providerId} tab; rebind required`);
  }

  if (binding) {
    throw new Error(`Provider ${providerId} is bound but has no recoverable tab; rebind required`);
  }

  const tabs = (await listProviderTabs(providerId, binding?.tab_id ?? null)).map((entry) => entry._tab);
  const activeTab = tabs.find((tab) => tab?.active && tab?.currentWindow);
  const chosen = activeTab ?? (tabs.length === 1 ? tabs[0] : null);
  if (!chosen?.id) {
    throw new Error(`No bound tab found for provider ${providerId}`);
  }

  await persistBinding(readyModule, workspaceId, providerId, chosen, { pin: true });
  return { tab: chosen, binding: null, snapshot };
}

async function ensureBoundConversationMatch(workspaceId, providerId, binding, tab) {
  if (!bindingHasBoundTarget(binding, providerId)) {
    return { mismatch: false, observedRef: binding?.conversation_ref ?? null };
  }

  const result = await sendAdapterCommand(workspaceId, providerId, { type: "get_conversation_ref" }, tab);
  const observedRef = conversationRefEvent(result?.events) ?? binding?.conversation_ref ?? null;
  return {
    mismatch: !conversationRefMatchesTarget(observedRef, binding.bound_conversation_ref),
    observedRef,
  };
}

function providerContentScriptFile(providerId) {
  switch (providerId) {
    case "gpt":
      return "content-gpt.js";
    case "gemini":
      return "content-gemini.js";
    case "grok":
      return "content-grok.js";
    case "claude":
      return "content-claude.js";
    default:
      throw new Error(`Unsupported provider target: ${providerId}`);
  }
}

async function injectProviderContentScript(tabId, providerId) {
  const file = providerContentScriptFile(providerId);

  if (runtimeApi.scripting?.executeScript) {
    await runtimeApi.scripting.executeScript({
      target: { tabId },
      files: [file],
    });
    return;
  }

  const executeScript = runtimeApi.tabs?.executeScript;
  if (!executeScript) {
    throw new Error(`No content-script injection API available for ${providerId}`);
  }

  const result = executeScript.call(runtimeApi.tabs, tabId, { file });
  if (result && typeof result.then === "function") {
    await result;
    return;
  }

  await callbackApiResult((done) => executeScript.call(runtimeApi.tabs, tabId, { file }, done));
}

async function pingProviderContentScript(tabId) {
  return await sendTabMessage(tabId, { channel: "chatmux_adapter_ping" });
}

async function ensureProviderContentScript(tabId, providerId) {
  try {
    const ping = await pingProviderContentScript(tabId);
    if (ping?.ok && ping?.installed) {
      return ping;
    }
  } catch (error) {
    if (!isNoReceiverError(error)) {
      throw error;
    }
  }

  await injectProviderContentScript(tabId, providerId);
  const ping = await pingProviderContentScript(tabId);
  if (!ping?.ok || !ping?.installed) {
    throw new Error(`Content runtime did not become ready for ${providerId}`);
  }
  return ping;
}

function adapterCommandFailure(events) {
  return (events ?? []).find((event) => event?.type === "command_failed") ?? null;
}

async function sendAdapterCommand(workspaceId, providerId, payload, tab, options = {}) {
  const resolvedTab = tab?.id ? tab : await findProviderTab(providerId);
  const message = {
    channel: "chatmux_adapter_command",
    workspaceId: String(workspaceId),
    payload,
    emitEvents: options.emitEvents !== false,
  };
  let result;

  await ensureProviderContentScript(resolvedTab.id, providerId);
  try {
    result = await sendTabMessage(resolvedTab.id, message);
  } catch (error) {
    if (!isNoReceiverError(error)) {
      throw error;
    }

    await ensureProviderContentScript(resolvedTab.id, providerId);
    result = await sendTabMessage(resolvedTab.id, message);
  }

  if (result?.ok === false) {
    throw new Error(result.error || `Adapter command failed for ${providerId}`);
  }

  const commandFailure = adapterCommandFailure(result?.events);
  if (commandFailure) {
    throw new Error(commandFailure.detail || `Adapter command failed for ${providerId}`);
  }

  if (payload?.type === "structural_probe") {
    const probeFailure = (result?.events ?? [])
      .find((event) => event?.type === "structural_probe_failed");
    if (probeFailure) {
      throw new Error(probeFailure.detail || `${providerDisplayName(providerId)} DOM probe failed`);
    }
  }

  return result;
}

async function waitForAdapterChange(tabId, timeoutMs) {
  try {
    const result = await sendTabMessage(tabId, {
      channel: "chatmux_adapter_wait_for_change",
      timeoutMs,
    });
    return Boolean(result?.ok && result?.changed);
  } catch (error) {
    if (!isNoReceiverError(error)) {
      throw error;
    }
    return false;
  }
}

async function reportAdapterFailure(wasmModule, workspaceId, providerId, detail) {
  if (!wasmModule?.handle_adapter_event_json) {
    return;
  }

  const events = await wasmModule.handle_adapter_event_json(
    String(workspaceId),
    JSON.stringify({
      type: "command_failed",
      provider: providerId,
      level: "critical",
      detail,
    })
  );
  await broadcastUiEvents(events);
}

function capturedMessages(result) {
  return (result?.events ?? [])
    .filter((event) => event?.type === "messages_captured")
    .flatMap((event) => event.messages ?? []);
}

function latestAssistant(messages, providerId) {
  return [...(messages ?? [])]
    .reverse()
    .find((message) => message?.participant_id === providerId && message?.role === "assistant") ?? null;
}

async function providerBaseline(workspaceId, providerId, tab) {
  const result = await sendAdapterCommand(
    workspaceId,
    providerId,
    { type: "extract_full_history" },
    tab,
    { emitEvents: false }
  );
  return latestAssistant(capturedMessages(result), providerId)?.id ?? null;
}

async function completedProviderResponse(workspaceId, providerId, tab, afterMessageId) {
  const timeoutMs = 120_000;
  const deadline = Date.now() + timeoutMs;
  let lastObservation = "no assistant response observed";
  let stableObservation = null;

  while (Date.now() < deadline) {
    if (killSwitchActive) {
      throw new Error("Global kill switch activated while waiting for the provider response");
    }

    await waitForAdapterChange(tab.id, Math.min(1_500, deadline - Date.now()));

    const healthResult = await sendAdapterCommand(
      workspaceId,
      providerId,
      { type: "get_health" },
      tab,
      { emitEvents: false }
    );
    const health = (healthResult?.events ?? [])
      .find((event) => event?.type === "health_report")?.health ?? "ready";

    let messages;
    try {
      const deltaResult = await sendAdapterCommand(
        workspaceId,
        providerId,
        { type: "extract_incremental_delta", after_message_id: afterMessageId },
        tab,
        { emitEvents: false }
      );
      messages = capturedMessages(deltaResult);
    } catch (_error) {
      const fullResult = await sendAdapterCommand(
        workspaceId,
        providerId,
        { type: "extract_full_history" },
        tab,
        { emitEvents: false }
      );
      messages = capturedMessages(fullResult).filter((message) => message?.id !== afterMessageId);
    }

    const assistants = messages.filter(
      (message) => message?.participant_id === providerId && message?.role === "assistant"
    );
    const response = assistants.at(-1) ?? null;
    const stability = observeStableResponse(stableObservation, response, Date.now());
    stableObservation = stability.observation;
    lastObservation = response
      ? `assistant=${response.id}, confidence=${response.capture_confidence}, health=${health}`
      : `assistant=none, health=${health}`;

    if (response && response.capture_confidence === "certain" && health !== "generating") {
      return assistants;
    }
    // Provider UIs occasionally leave stale generation affordances mounted
    // after the final answer is visible. A new assistant body that remains
    // byte-for-byte stable across several polls is a bounded completion
    // fallback; the baseline filter prevents accepting an older turn.
    if (response && stability.stable) {
      return assistants;
    }
  }

  throw new Error(
    `${providerDisplayName(providerId)} did not produce a completed response within ${timeoutMs}ms (${lastObservation})`
  );
}

async function sendProviderPromptWhenReady(workspaceId, providerId, tab) {
  const deadline = Date.now() + 5_000;
  let lastError = null;

  while (Date.now() < deadline) {
    try {
      return await sendAdapterCommand(
        workspaceId,
        providerId,
        { type: "send" },
        tab,
        { emitEvents: false }
      );
    } catch (error) {
      if (!isTransientSendControlError(error)) {
        throw error;
      }
      lastError = error;
      await waitForAdapterChange(tab.id, Math.min(500, deadline - Date.now()));
    }
  }

  throw lastError ?? new Error(
    `${providerDisplayName(providerId)} send control did not become ready`
  );
}

async function drivePendingDispatch(wasmModule, workspaceId, dispatch) {
  const target = dispatch.target_participant_id;
  try {
    if (killSwitchActive) {
      throw new Error("Global kill switch is active");
    }
    const { tab, binding } = await resolveBoundProviderTab(wasmModule, workspaceId, target);
    const match = await ensureBoundConversationMatch(workspaceId, target, binding, tab);
    if (match.mismatch) {
      throw new Error(mismatchDetail(binding, match.observedRef));
    }
    const blockingDetail = await providerBlockingDetail(workspaceId, target, tab);
    if (blockingDetail) {
      throw new Error(blockingDetail);
    }

    await sendAdapterCommand(
      workspaceId,
      target,
      { type: "structural_probe" },
      tab,
      { emitEvents: false }
    );
    const afterMessageId = await providerBaseline(workspaceId, target, tab);

    if (killSwitchActive) {
      throw new Error("Global kill switch activated before input injection");
    }
    await sendAdapterCommand(
      workspaceId,
      target,
      { type: "inject_input", text: String(dispatch.rendered_payload ?? "") },
      tab,
      { emitEvents: false }
    );
    if (killSwitchActive) {
      throw new Error("Global kill switch activated before send");
    }
    await sendProviderPromptWhenReady(workspaceId, target, tab);
    await executeCoreCommand(wasmModule, {
      type: "acknowledge_dispatch_delivered",
      dispatch_id: dispatch.id,
    });

    const messages = await completedProviderResponse(
      workspaceId,
      target,
      tab,
      afterMessageId
    );
    const continuationEvents = await executeCoreCommand(wasmModule, {
      type: "acknowledge_dispatch_captured",
      dispatch_id: dispatch.id,
      messages,
    });
    await drivePendingDispatches(
      wasmModule,
      { type: "acknowledge_dispatch_captured", workspace_id: workspaceId },
      continuationEvents
    );

    await sendAdapterCommand(workspaceId, target, { type: "get_conversation_ref" }, tab);
    await sendAdapterCommand(workspaceId, target, { type: "get_provider_snapshot" }, tab);
  } catch (error) {
    const detail = error?.message ?? String(error);
    try {
      const continuationEvents = await executeCoreCommand(wasmModule, {
        type: "acknowledge_dispatch_failed",
        dispatch_id: dispatch.id,
        detail,
      });
      await drivePendingDispatches(
        wasmModule,
        { type: "acknowledge_dispatch_failed", workspace_id: workspaceId },
        continuationEvents
      );
    } catch (ackError) {
      logError(ackError);
    }
    await reportAdapterFailure(wasmModule, workspaceId, target, detail).catch(logError);
  }
}

function workspaceIdForEvents(command, events) {
  if (command?.workspace_id) {
    return command.workspace_id;
  }
  return (events ?? [])
    .find((event) => event?.type === "run_updated")?.run?.workspace_id ?? null;
}

async function drivePendingDispatches(wasmModule, command, events) {
  const workspaceId = workspaceIdForEvents(command, events);
  const pending = (events ?? [])
    .filter((event) => event?.type === "dispatch_updated")
    .map((event) => event.dispatch)
    .filter((dispatch) => dispatch?.outcome === "pending");
  if (!workspaceId || pending.length === 0) {
    return;
  }
  const run = (events ?? [])
    .find((event) => event?.type === "run_updated")?.run ?? null;
  if (command?.type === "acknowledge_dispatch_captured" && run) {
    const baseDelay = Number(run.timing_policy?.inter_round_delay_secs ?? 0) * 1000;
    const jitter = Math.min(100, Number(run.timing_policy?.jitter_percent ?? 0));
    const factor = 1 + (((Math.random() * 2) - 1) * jitter / 100);
    await delay(Math.max(0, Math.round(baseDelay * factor)));
  }
  const concurrency = Math.max(1, Number(run?.timing_policy?.max_concurrent_sends ?? 4));
  for (let index = 0; index < pending.length; index += concurrency) {
    const batch = pending.slice(index, index + concurrency);
    await Promise.all(batch.map((dispatch) =>
      drivePendingDispatch(wasmModule, workspaceId, dispatch)
    ));
  }
}

async function maybeSyncProviderConversation(wasmModule, command) {
  if (command?.type !== "sync_provider_conversation") {
    return;
  }

  try {
    const provider = command.provider;
    const { tab, binding } = await resolveBoundProviderTab(wasmModule, command.workspace_id, provider);
    const match = await ensureBoundConversationMatch(command.workspace_id, provider, binding, tab);
    if (match.mismatch) {
      throw new Error(mismatchDetail(binding, match.observedRef));
    }
    const blockingDetail = await providerBlockingDetail(command.workspace_id, provider, tab);
    if (blockingDetail) {
      throw new Error(blockingDetail);
    }
    await sendAdapterCommand(command.workspace_id, provider, { type: "get_provider_snapshot" }, tab);
    await sendAdapterCommand(command.workspace_id, provider, { type: "extract_full_history" }, tab);
  } catch (error) {
    await reportAdapterFailure(
      wasmModule,
      command.workspace_id,
      command.provider,
      error?.message ?? String(error)
    ).catch(logError);
  }
}

async function maybeDriveProviderControl(wasmModule, command) {
  if (!command?.type || !command?.workspace_id || !command?.provider) {
    return;
  }

  const commandMap = {
    request_provider_control_state: [
      { type: "get_conversation_ref" },
      { type: "get_provider_snapshot" },
    ],
    create_provider_project: [
      { type: "create_project", title: String(command.title ?? "") },
      { type: "get_provider_snapshot" },
    ],
    select_provider_project: [
      { type: "select_project", project_id: String(command.project_id ?? "") },
      { type: "get_provider_snapshot" },
    ],
    create_provider_conversation: [
      {
        type: "create_conversation",
        project_id: command.project_id == null ? null : String(command.project_id),
        title: String(command.title ?? ""),
      },
      { type: "get_provider_snapshot" },
    ],
    select_provider_conversation: [
      { type: "select_conversation", conversation_id: String(command.conversation_id ?? "") },
      { type: "get_provider_snapshot" },
    ],
    set_provider_model: [
      { type: "set_model", model_id: String(command.model_id ?? "") },
      { type: "get_provider_snapshot" },
    ],
    set_provider_reasoning: [
      { type: "set_reasoning", reasoning_id: String(command.reasoning_id ?? "") },
      { type: "get_provider_snapshot" },
    ],
    set_provider_feature_flag: [
      {
        type: "set_feature_flag",
        key: String(command.key ?? ""),
        enabled: Boolean(command.enabled),
      },
      { type: "get_provider_snapshot" },
    ],
  };

  const adapterCommands = commandMap[command.type];
  if (!adapterCommands) {
    return;
  }

  try {
    const { tab, binding } = await resolveBoundProviderTab(wasmModule, command.workspace_id, command.provider);
    const isRecoverySelect = command.type === "select_provider_conversation"
      && bindingHasBoundTarget(binding)
      && String(command.conversation_id ?? "") === String(binding?.bound_conversation_ref?.conversation_id ?? "");
    if (command.type !== "request_provider_control_state") {
      const match = await ensureBoundConversationMatch(command.workspace_id, command.provider, binding, tab);
      if (match.mismatch && !isRecoverySelect) {
        throw new Error(mismatchDetail(binding, match.observedRef));
      }
    }
    for (const payload of adapterCommands) {
      await sendAdapterCommand(command.workspace_id, command.provider, payload, tab);
    }
  } catch (error) {
    await reportAdapterFailure(
      wasmModule,
      command.workspace_id,
      command.provider,
      error?.message ?? String(error)
    ).catch(logError);
  }
}

async function maybeHandleProviderBindingCommand(wasmModule, command) {
  if (!command?.type || !command?.workspace_id || !command?.provider) {
    return null;
  }

  if (command.type === "request_provider_tab_candidates") {
    const snapshot = await requestWorkspaceSnapshot(wasmModule, command.workspace_id);
    const binding = bindingForProvider(snapshot, command.provider);
    const candidates = (await listProviderTabs(command.provider, binding?.tab_id ?? null))
      .map((entry) => entry.candidate);
    const events = [{
      type: "provider_tab_candidates",
      workspace_id: String(command.workspace_id),
      provider: command.provider,
      candidates,
    }];
    await broadcastUiEvents(events);
    return events;
  }

  if (command.type === "bind_provider_tab") {
    const tab = await tabsGet(command.tab_id);
    await persistBinding(wasmModule, command.workspace_id, command.provider, tab, {
      pin: command.pin,
      origin: command.origin,
      tab_title: command.tab_title,
      tab_url: command.tab_url,
      conversation_id: command.conversation_id,
      conversation_title: command.conversation_title,
      conversation_url: command.conversation_url,
    });
    await sendAdapterCommand(
      command.workspace_id,
      command.provider,
      { type: "get_conversation_ref" },
      tab
    );
    await sendAdapterCommand(
      command.workspace_id,
      command.provider,
      { type: "get_provider_snapshot" },
      tab
    );
    return [];
  }

  if (command.type === "open_provider_tab") {
    await openAndBindProviderTab(
      wasmModule,
      command.workspace_id,
      command.provider,
      command.prefer_existing !== false
    );
    return [];
  }

  return null;
}

async function preflightBoundConversationCommand(wasmModule, command) {
  if (!command?.type) {
    return null;
  }

  if (command.type === "send_manual_message") {
    if (command.approval_mode !== "auto_send") {
      return null;
    }
    for (const target of command.targets ?? []) {
      const { tab, binding } = await resolveBoundProviderTab(wasmModule, command.workspace_id, target);
      const match = await ensureBoundConversationMatch(command.workspace_id, target, binding, tab);
      if (match.mismatch) {
        return mismatchDetail(binding, match.observedRef);
      }
      const blockingDetail = await providerBlockingDetail(command.workspace_id, target, tab);
      if (blockingDetail) {
        return blockingDetail;
      }
    }
    return null;
  }

  if (command.type === "sync_provider_conversation") {
    const { tab, binding } = await resolveBoundProviderTab(wasmModule, command.workspace_id, command.provider);
    const match = await ensureBoundConversationMatch(command.workspace_id, command.provider, binding, tab);
    if (match.mismatch) {
      return mismatchDetail(binding, match.observedRef);
    }
    return await providerBlockingDetail(command.workspace_id, command.provider, tab);
  }

  const guardedTypes = new Set([
    "create_provider_project",
    "select_provider_project",
    "create_provider_conversation",
    "select_provider_conversation",
    "set_provider_model",
    "set_provider_reasoning",
    "set_provider_feature_flag",
  ]);
  if (!guardedTypes.has(command.type)) {
    return null;
  }

  const { tab, binding } = await resolveBoundProviderTab(wasmModule, command.workspace_id, command.provider);
  const isRecoverySelect = command.type === "select_provider_conversation"
    && bindingHasBoundTarget(binding)
    && String(command.conversation_id ?? "") === String(binding?.bound_conversation_ref?.conversation_id ?? "");
  if (isRecoverySelect) {
    return null;
  }

  const match = await ensureBoundConversationMatch(command.workspace_id, command.provider, binding, tab);
  return match.mismatch ? mismatchDetail(binding, match.observedRef) : null;
}

function shouldHandleActionClick() {
  return Boolean(runtimeApi.sidebarAction?.open || !runtimeApi.sidePanel?.setPanelBehavior);
}

function wireWorkspaceOpeners() {
  if (runtimeApi.sidePanel?.setPanelBehavior) {
    runtimeApi.sidePanel
      .setPanelBehavior({ openPanelOnActionClick: true })
      .catch(logError);
  }

  if (runtimeApi.action?.onClicked) {
    runtimeApi.action.onClicked.addListener((tab) => {
      if (shouldHandleActionClick()) {
        openWorkspaceSurface(tab).catch(logError);
      }
    });
  }

  if (runtimeApi.commands?.onCommand) {
    runtimeApi.commands.onCommand.addListener((command) => {
      if (command === "open-workspace") {
        openWorkspaceSurface().catch(logError);
      }
    });
  }
}

// Context menu: "Open Chatmux Dashboard" on right-click of extension icon
const menusApi = runtimeApi.contextMenus ?? runtimeApi.menus;
if (menusApi?.create) {
  menusApi.create(
    {
      id: "chatmux-open-dashboard",
      title: "Open Chatmux Dashboard",
      contexts: ["action"],
    },
    () => {
      // Ignore "duplicate id" errors on restart
      const err = globalThis.chrome?.runtime?.lastError ?? globalThis.browser?.runtime?.lastError;
      if (err && !String(err.message || err).includes("duplicate")) {
        logError(err);
      }
    }
  );

  const clickApi = menusApi.onClicked ?? menusApi.onClicked;
  if (clickApi) {
    clickApi.addListener((info) => {
      if (info.menuItemId === "chatmux-open-dashboard") {
        const dashboardUrl = runtimeApi.runtime.getURL("ui/index.html");
        tabsCreate({ url: dashboardUrl }).catch(logError);
      }
    });
  }
}

wireWorkspaceOpeners();

const wasmReady = (async () => {
  await initChatmuxCore();
  if (typeof chatmuxCore.bootstrap_background === "function") {
    const status = await chatmuxCore.bootstrap_background();
    killSwitchActive = Boolean(status?.kill_switch_active);
  }
  return chatmuxCore;
})().catch((error) => {
  logError(error);
  throw error;
});

runtimeApi.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (!message || typeof message !== "object") {
    return false;
  }

  if (message.channel === "chatmux_ui_command") {
    wasmReady
      .then(async (readyModule) => {
        const handledBindingEvents = await maybeHandleProviderBindingCommand(readyModule, message.payload).catch(logError);
        if (handledBindingEvents) {
          sendResponse({ ok: true, events: handledBindingEvents });
          return;
        }
        const preflightError = await preflightBoundConversationCommand(readyModule, message.payload);
        if (preflightError) {
          sendResponse({ ok: false, error: preflightError });
          return;
        }
        const events = await executeCoreCommand(readyModule, message.payload);
        drivePendingDispatches(readyModule, message.payload, events).catch(logError);
        await maybeSyncProviderConversation(readyModule, message.payload).catch(logError);
        await maybeDriveProviderControl(readyModule, message.payload).catch(logError);
        sendResponse({ ok: true, events });
      })
      .catch((error) =>
        sendResponse({ ok: false, error: error?.message ?? String(error) })
      );
    return true;
  }

  if (message.channel === "chatmux_adapter_event") {
    wasmReady
      .then(async (readyModule) => {
        if (!readyModule.handle_adapter_event_json) {
          throw new Error("Chatmux adapter event bridge is unavailable");
        }

        const events = await readyModule.handle_adapter_event_json(
          String(message.workspaceId),
          JSON.stringify(message.payload)
        );
        await broadcastUiEvents(events);
        sendResponse({ ok: true, events });
      })
      .catch((error) =>
        sendResponse({ ok: false, error: error?.message ?? String(error) })
      );
    return true;
  }

  return false;
});
