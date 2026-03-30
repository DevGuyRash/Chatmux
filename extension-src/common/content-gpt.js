const runtimeApi = globalThis.browser ?? globalThis.chrome;
const NETWORK_CAPTURE_FLAG = "__chatmuxNetworkCaptureInstalled";
const NETWORK_CAPTURE_KEY = "__chatmuxLatestNetworkCapture";

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

function installNetworkCapture() {
  if (globalThis[NETWORK_CAPTURE_FLAG]) {
    return;
  }
  globalThis[NETWORK_CAPTURE_FLAG] = true;

  globalThis.addEventListener("message", (event) => {
    if (event.source !== globalThis || event.data?.source !== "chatmux-network-capture") {
      return;
    }
    globalThis[NETWORK_CAPTURE_KEY] = event.data.capture ?? null;
  });

  const script = document.createElement("script");
  script.dataset.chatmuxNetworkCapture = "true";
  script.textContent = `
    (() => {
      if (window.${NETWORK_CAPTURE_FLAG}) return;
      window.${NETWORK_CAPTURE_FLAG} = true;
      const store = (capture) => {
        window.${NETWORK_CAPTURE_KEY} = capture;
        window.postMessage({ source: "chatmux-network-capture", capture }, "*");
      };

      const originalFetch = window.fetch;
      if (typeof originalFetch === "function") {
        window.fetch = async (...args) => {
          const [input, init] = args;
          const requestUrl = typeof input === "string" ? input : input?.url;
          const requestMethod = init?.method || input?.method || "GET";
          const requestBody = typeof init?.body === "string" ? init.body : null;
          const response = await originalFetch(...args);
          let responseBody = null;
          try {
            responseBody = await response.clone().text();
          } catch (_error) {}
          store({
            request_method: requestMethod,
            request_url: requestUrl ?? null,
            request_body: requestBody,
            response_status: Number.isFinite(response.status) ? response.status : null,
            response_body: responseBody,
            capture_strategy: "network",
          });
          return response;
        };
      }
    })();
  `;
  (document.documentElement || document.head || document.body).appendChild(script);
  script.remove();
}

(async () => {
  installNetworkCapture();
  const moduleUrl = runtimeApi.runtime.getURL("wasm/chatmux_adapter_gpt.js");
  const wasmModule = await import(moduleUrl);
  await wasmModule.default(runtimeApi.runtime.getURL("wasm/chatmux_adapter_gpt_bg.wasm"));
  if (typeof wasmModule.bootstrap_gpt_content_script === "function") {
    wasmModule.bootstrap_gpt_content_script();
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
