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

async function sendTabMessage(tabId, message) {
  const result = runtimeApi.tabs?.sendMessage?.(tabId, message);
  if (result && typeof result.then === "function") {
    return await result;
  }

  return await callbackApiResult((done) => runtimeApi.tabs.sendMessage(tabId, message, done));
}

function ignoreNoReceiver(error) {
  const message = error?.message ?? String(error);
  if (!message.includes("Receiving end does not exist")) {
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

async function sendAdapterCommand(workspaceId, providerId, payload) {
  const tab = await findProviderTab(providerId);
  const result = await sendTabMessage(tab.id, {
    channel: "chatmux_adapter_command",
    workspaceId: String(workspaceId),
    payload,
  });

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
      await sendAdapterCommand(command.workspace_id, target, { type: "detect_blocking_state" });
      await sendAdapterCommand(command.workspace_id, target, { type: "inject_input", text: payloadText });
      await sendAdapterCommand(command.workspace_id, target, { type: "send" });
      setTimeout(() => {
        sendAdapterCommand(command.workspace_id, target, { type: "get_conversation_ref" })
          .catch((error) => reportAdapterFailure(wasmModule, command.workspace_id, target, error?.message ?? String(error)).catch(logError));
        sendAdapterCommand(command.workspace_id, target, { type: "extract_latest_response" })
          .catch((error) => reportAdapterFailure(wasmModule, command.workspace_id, target, error?.message ?? String(error)).catch(logError));
      }, 2000);
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
    await sendAdapterCommand(command.workspace_id, provider, { type: "detect_blocking_state" });
    await sendAdapterCommand(command.workspace_id, provider, { type: "get_conversation_ref" });
    await sendAdapterCommand(command.workspace_id, provider, { type: "extract_full_history" });
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
    request_provider_control_state: [{ type: "get_provider_snapshot" }],
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
    for (const payload of adapterCommands) {
      await sendAdapterCommand(command.workspace_id, command.provider, payload);
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

(async () => {
  wireWorkspaceOpeners();

  const moduleUrl = runtimeApi.runtime.getURL("wasm/chatmux_core.js");
  const wasmModule = await import(moduleUrl);
  await wasmModule.default(runtimeApi.runtime.getURL("wasm/chatmux_core_bg.wasm"));
  if (typeof wasmModule.bootstrap_background === "function") {
    await wasmModule.bootstrap_background();
  }

  runtimeApi.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (!message || typeof message !== "object") {
      return false;
    }

    if (message.channel === "chatmux_ui_command" && wasmModule.handle_ui_command_json) {
      wasmModule
        .handle_ui_command_json(JSON.stringify(message.payload))
        .then(async (events) => {
          await broadcastUiEvents(events);
          await maybeDriveManualMessage(wasmModule, message.payload).catch(logError);
          await maybeSyncProviderConversation(wasmModule, message.payload).catch(logError);
          await maybeDriveProviderControl(wasmModule, message.payload).catch(logError);
          sendResponse({ ok: true, events });
        })
        .catch((error) =>
          sendResponse({ ok: false, error: error?.message ?? String(error) })
        );
      return true;
    }

    if (
      message.channel === "chatmux_adapter_event" &&
      wasmModule.handle_adapter_event_json
    ) {
      wasmModule
        .handle_adapter_event_json(
          String(message.workspaceId),
          JSON.stringify(message.payload)
        )
        .then(async (events) => {
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
})();
