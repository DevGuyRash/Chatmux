const runtimeApi = globalThis.browser ?? globalThis.chrome;

(async () => {
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
        .then((events) => sendResponse({ ok: true, events }))
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
        .then((events) => sendResponse({ ok: true, events }))
        .catch((error) =>
          sendResponse({ ok: false, error: error?.message ?? String(error) })
        );
      return true;
    }

    return false;
  });
})();
