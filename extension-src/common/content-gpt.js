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
      const normalizeUrl = (value) => {
        if (!value) return null;
        return String(value).split("#")[0].split("?")[0].replace(/\/+$/, "");
      };
      const conversationIdFromUrl = (value) => {
        if (!value) return null;
        try {
          const parsed = new URL(value, window.location.href);
          const segments = parsed.pathname.split("/").filter(Boolean);
          const chatIndex = segments.indexOf("c");
          return chatIndex >= 0 ? segments[chatIndex + 1] ?? null : null;
        } catch (_error) {
          return null;
        }
      };
      const findConversationId = (value, depth = 0) => {
        if (depth > 6 || value == null) return null;
        if (Array.isArray(value)) {
          for (const item of value) {
            const nested = findConversationId(item, depth + 1);
            if (nested) return nested;
          }
          return null;
        }
        if (typeof value === "object") {
          if (typeof value.conversation_id === "string" && value.conversation_id) {
            return value.conversation_id;
          }
          if (typeof value.conversationId === "string" && value.conversationId) {
            return value.conversationId;
          }
          for (const nestedValue of Object.values(value)) {
            const nested = findConversationId(nestedValue, depth + 1);
            if (nested) return nested;
          }
        }
        return null;
      };
      const tryParseJson = (text) => {
        if (!text) return null;
        try {
          return JSON.parse(text);
        } catch (_error) {
          return null;
        }
      };
      const relevantRequest = (requestUrl, requestBody, responseBody) => {
        const normalizedUrl = normalizeUrl(requestUrl);
        return Boolean(
          (normalizedUrl && normalizedUrl.includes("/backend-api/"))
            || conversationIdFromUrl(requestUrl)
            || findConversationId(tryParseJson(requestBody))
            || findConversationId(tryParseJson(responseBody))
        );
      };
      const buildConversationRef = (requestUrl, requestBody, responseBody) => {
        const requestJson = tryParseJson(requestBody);
        const responseJson = tryParseJson(responseBody);
        const conversationId =
          conversationIdFromUrl(requestUrl)
          || findConversationId(requestJson)
          || findConversationId(responseJson);
        if (!conversationId) return null;
        return {
          conversation_id: conversationId,
          title: null,
          url: normalizeUrl(new URL("/c/" + conversationId, window.location.origin).toString()),
          model_label: null,
        };
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
          if (!relevantRequest(requestUrl, requestBody, responseBody)) {
            return response;
          }
          store({
            request_method: requestMethod,
            request_url: requestUrl ?? null,
            request_body: requestBody,
            response_status: Number.isFinite(response.status) ? response.status : null,
            response_body: responseBody,
            capture_strategy: "network",
            conversation_ref: buildConversationRef(requestUrl, requestBody, responseBody),
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
