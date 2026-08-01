const runtimeApi = globalThis.browser ?? globalThis.chrome;
const PROVIDER_ID = "gemini";
const INSTALL_KEY = "__chatmux_gemini_content_runtime";

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

function commandFailure(events) {
  return (events ?? []).find((event) => event?.type === "command_failed") ?? null;
}

function waitForDocumentChange(timeoutMs = 1_500) {
  return new Promise((resolve) => {
    const root = document.body ?? document.documentElement;
    if (!root) {
      resolve({ changed: false });
      return;
    }

    let settled = false;
    const finish = (changed) => {
      if (settled) {
        return;
      }
      settled = true;
      observer.disconnect();
      clearTimeout(timer);
      resolve({ changed });
    };
    const observer = new MutationObserver(() => finish(true));
    observer.observe(root, {
      subtree: true,
      childList: true,
      characterData: true,
      attributes: true,
      attributeFilter: ["aria-disabled", "disabled", "data-state"],
    });
    const timer = setTimeout(() => finish(false), Math.max(50, Number(timeoutMs) || 1_500));
  });
}

if (!globalThis[INSTALL_KEY]) {
  const state = {
    module: null,
    probeError: null,
    moduleReady: null,
  };
  globalThis[INSTALL_KEY] = state;

  let resolveModule;
  let rejectModule;
  state.moduleReady = new Promise((resolve, reject) => {
    resolveModule = resolve;
    rejectModule = reject;
  });

  runtimeApi.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (!message || typeof message !== "object") {
      return false;
    }

    if (message.channel === "chatmux_adapter_ping") {
      sendResponse({
        ok: true,
        provider: PROVIDER_ID,
        installed: true,
        ready: Boolean(state.module),
        probeError: state.probeError,
      });
      return false;
    }

    if (message.channel === "chatmux_adapter_wait_for_change") {
      waitForDocumentChange(message.timeoutMs)
        .then((result) => sendResponse({ ok: true, ...result }))
        .catch((error) => sendResponse({ ok: false, error: error?.message ?? String(error) }));
      return true;
    }

    if (message.channel !== "chatmux_adapter_command") {
      return false;
    }

    state.moduleReady
      .then(async (wasmModule) => {
        const execute = () => Promise.resolve(
          wasmModule.handle_adapter_command_json(JSON.stringify(message.payload))
        );

        let events = await execute();
        if (message.payload?.type === "send" && commandFailure(events)) {
          await waitForDocumentChange(1_500);
          events = await execute();
        }

        if (message.emitEvents !== false) {
          for (const event of events ?? []) {
            await runtimeSendMessage({
              channel: "chatmux_adapter_event",
              workspaceId: String(message.workspaceId),
              payload: event,
            });
          }
        }

        sendResponse({ ok: true, eventCount: events?.length ?? 0, events });
      })
      .catch((error) => {
        sendResponse({ ok: false, error: error?.message ?? String(error) });
      });

    return true;
  });

  (async () => {
    const moduleUrl = runtimeApi.runtime.getURL("wasm/chatmux_adapter_gemini.js");
    const wasmModule = await import(moduleUrl);
    await wasmModule.default({
      module_or_path: runtimeApi.runtime.getURL("wasm/chatmux_adapter_gemini_bg.wasm"),
    });
    state.module = wasmModule;
    if (typeof wasmModule.bootstrap_gemini_content_script === "function") {
      try {
        wasmModule.bootstrap_gemini_content_script();
      } catch (error) {
        state.probeError = error?.message ?? String(error);
      }
    }
    resolveModule(wasmModule);
  })().catch((error) => {
    state.probeError = error?.message ?? String(error);
    rejectModule(error);
  });
}
