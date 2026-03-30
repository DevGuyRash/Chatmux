const runtimeApi = globalThis.browser ?? globalThis.chrome;
const logError = (error) => console.error(error?.message ?? error);
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

async function sendTabMessage(tabId, message) {
  const result = runtimeApi.tabs?.sendMessage?.(tabId, message);
  if (result && typeof result.then === "function") {
    return await result;
  }

  return await callbackApiResult((done) => runtimeApi.tabs.sendMessage(tabId, message, done));
}

function delay(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function isNoReceiverError(error) {
  const message = error?.message ?? String(error);
  return message.includes("Receiving end does not exist");
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

function conversationRefFromUrl(url) {
  if (!url) {
    return {
      conversation_id: null,
      conversation_title: null,
      conversation_url: null,
    };
  }

  try {
    const parsed = new URL(url);
    const segments = parsed.pathname.split("/").filter(Boolean);
    const cIndex = segments.indexOf("c");
    return {
      conversation_id: cIndex >= 0 ? segments[cIndex + 1] ?? null : null,
      conversation_title: null,
      conversation_url: parsed.toString(),
    };
  } catch (_error) {
    return {
      conversation_id: null,
      conversation_title: null,
      conversation_url: null,
    };
  }
}

function normalizeConversationUrl(url) {
  if (!url) {
    return null;
  }
  return String(url).split("#")[0].split("?")[0].replace(/\/+$/, "");
}

function conversationRefMatchesTarget(currentRef, targetRef) {
  if (!currentRef || !targetRef) {
    return false;
  }
  if (currentRef.conversation_id && targetRef.conversation_id) {
    return currentRef.conversation_id === targetRef.conversation_id;
  }
  const currentUrl = normalizeConversationUrl(currentRef.url);
  const targetUrl = normalizeConversationUrl(targetRef.url);
  return Boolean(currentUrl && targetUrl && currentUrl === targetUrl);
}

function bindingHasBoundTarget(binding) {
  const target = binding?.bound_conversation_ref;
  return Boolean(target?.conversation_id || normalizeConversationUrl(target?.url));
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
      const conversationRef = conversationRefFromUrl(tab.url);
      return {
        type: "provider_tab_candidates",
        _tab: tab,
        candidate: {
          tab_id: tab.id,
          window_id: tab.windowId ?? null,
          title: tab.title ?? null,
          url: tab.url ?? null,
          conversation_id: conversationRef.conversation_id,
          conversation_title: conversationRef.conversation_title ?? tab.title ?? null,
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
  const conversationRef = {
    ...conversationRefFromUrl(tabUrl),
    conversation_id: options.conversation_id ?? conversationRefFromUrl(tabUrl).conversation_id,
    conversation_title: options.conversation_title ?? tab.title ?? null,
    conversation_url: options.conversation_url ?? conversationRefFromUrl(tabUrl).conversation_url,
  };

  const events = await readyModule.handle_ui_command_json(JSON.stringify({
    type: "bind_provider_tab",
    workspace_id: String(workspaceId),
    provider: providerId,
    tab_id: tab.id,
    window_id: tab.windowId ?? null,
    origin: options.origin ?? originFromUrl(tabUrl),
    tab_title: options.tab_title ?? tab.title ?? null,
    tab_url: tabUrl,
    conversation_id: conversationRef.conversation_id,
    conversation_title: conversationRef.conversation_title,
    conversation_url: conversationRef.conversation_url,
    pin,
  }));
  await broadcastUiEvents(events);
  return events;
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
  if (!bindingHasBoundTarget(binding)) {
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

async function sendAdapterCommand(workspaceId, providerId, payload, tab) {
  const resolvedTab = tab?.id ? tab : await findProviderTab(providerId);
  const message = {
    channel: "chatmux_adapter_command",
    workspaceId: String(workspaceId),
    payload,
  };
  let result;

  try {
    result = await sendTabMessage(resolvedTab.id, message);
  } catch (error) {
    if (!isNoReceiverError(error)) {
      throw error;
    }

    await injectProviderContentScript(resolvedTab.id, providerId);
    await delay(150);
    result = await sendTabMessage(resolvedTab.id, message);
  }

  if (result?.ok === false) {
    throw new Error(result.error || `Adapter command failed for ${providerId}`);
  }

  return result;
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

async function maybeDriveManualMessage(wasmModule, command) {
  if (command?.type !== "send_manual_message") {
    return;
  }

  const payloadText = String(command.text ?? "");
  for (const target of command.targets ?? []) {
    try {
      const { tab, binding, snapshot } = await resolveBoundProviderTab(wasmModule, command.workspace_id, target);
      const match = await ensureBoundConversationMatch(command.workspace_id, target, binding, tab);
      if (match.mismatch) {
        throw new Error(mismatchDetail(binding, match.observedRef));
      }
      const recentMessages = snapshot?.recent_messages ?? [];
      const latestAssistant = [...recentMessages]
        .reverse()
        .find((message) => message?.participant_id === target && message?.role === "assistant");
      const afterMessageId = latestAssistant?.id ?? null;

      await sendAdapterCommand(command.workspace_id, target, { type: "detect_blocking_state" }, tab);
      await sendAdapterCommand(command.workspace_id, target, { type: "inject_input", text: payloadText }, tab);
      await sendAdapterCommand(command.workspace_id, target, { type: "send" }, tab);

      for (let attempt = 0; attempt < 30; attempt += 1) {
        await delay(1000);
        const result = await sendAdapterCommand(
          command.workspace_id,
          target,
          { type: "extract_incremental_delta", after_message_id: afterMessageId },
          tab
        );
        const messages = (result?.events ?? [])
          .filter((event) => event?.type === "messages_captured")
          .flatMap((event) => event.messages ?? []);
        if (messages.some((message) => message?.participant_id === target && message?.role === "assistant")) {
          break;
        }
      }

      await sendAdapterCommand(command.workspace_id, target, { type: "get_conversation_ref" }, tab);
      await sendAdapterCommand(command.workspace_id, target, { type: "get_provider_snapshot" }, tab);
    } catch (error) {
      await reportAdapterFailure(wasmModule, command.workspace_id, target, error?.message ?? String(error)).catch(logError);
    }
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
    await sendAdapterCommand(command.workspace_id, provider, { type: "detect_blocking_state" }, tab);
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

  return null;
}

async function preflightBoundConversationCommand(wasmModule, command) {
  if (!command?.type) {
    return null;
  }

  if (command.type === "send_manual_message") {
    for (const target of command.targets ?? []) {
      const { tab, binding } = await resolveBoundProviderTab(wasmModule, command.workspace_id, target);
      const match = await ensureBoundConversationMatch(command.workspace_id, target, binding, tab);
      if (match.mismatch) {
        return mismatchDetail(binding, match.observedRef);
      }
    }
    return null;
  }

  if (command.type === "sync_provider_conversation") {
    const { tab, binding } = await resolveBoundProviderTab(wasmModule, command.workspace_id, command.provider);
    const match = await ensureBoundConversationMatch(command.workspace_id, command.provider, binding, tab);
    return match.mismatch ? mismatchDetail(binding, match.observedRef) : null;
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
        (runtimeApi.tabs?.create?.({ url: dashboardUrl }) ?? Promise.resolve()).catch(logError);
      }
    });
  }
}

wireWorkspaceOpeners();

const wasmReady = (async () => {
  const moduleUrl = runtimeApi.runtime.getURL("wasm/chatmux_core.js");
  const readyModule = await import(moduleUrl);
  await readyModule.default(runtimeApi.runtime.getURL("wasm/chatmux_core_bg.wasm"));
  if (typeof readyModule.bootstrap_background === "function") {
    await readyModule.bootstrap_background();
  }
  return readyModule;
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
        if (!readyModule.handle_ui_command_json) {
          throw new Error("Chatmux background core is unavailable");
        }

        const events = await readyModule.handle_ui_command_json(JSON.stringify(message.payload));
        await broadcastUiEvents(events);
        await maybeDriveManualMessage(readyModule, message.payload).catch(logError);
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
