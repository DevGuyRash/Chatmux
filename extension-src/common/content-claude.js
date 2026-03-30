const runtimeApi = globalThis.browser ?? globalThis.chrome;

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

(async () => {
  const moduleUrl = runtimeApi.runtime.getURL("wasm/chatmux_adapter_claude.js");
  const wasmModule = await import(moduleUrl);
  await wasmModule.default(runtimeApi.runtime.getURL("wasm/chatmux_adapter_claude_bg.wasm"));
  if (typeof wasmModule.bootstrap_claude_content_script === "function") {
    wasmModule.bootstrap_claude_content_script();
  }

  runtimeApi.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (!message || typeof message !== "object" || message.channel !== "chatmux_adapter_command") {
      return false;
    }

    Promise.resolve(
      wasmModule.handle_adapter_command_json(JSON.stringify(message.payload))
    )
      .then(async (events) => {
        for (const event of events ?? []) {
          await runtimeSendMessage({
            channel: "chatmux_adapter_event",
            workspaceId: String(message.workspaceId),
            payload: event,
          });
        }
        sendResponse({ ok: true, eventCount: events?.length ?? 0, events });
      })
      .catch((error) => {
        sendResponse({ ok: false, error: error?.message ?? String(error) });
      });

    return true;
  });
})();
