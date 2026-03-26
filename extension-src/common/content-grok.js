const runtimeApi = globalThis.browser ?? globalThis.chrome;

(async () => {
  const moduleUrl = runtimeApi.runtime.getURL("wasm/chatmux_adapter_grok.js");
  const wasmModule = await import(moduleUrl);
  await wasmModule.default(runtimeApi.runtime.getURL("wasm/chatmux_adapter_grok_bg.wasm"));
  if (typeof wasmModule.bootstrap_grok_content_script === "function") {
    wasmModule.bootstrap_grok_content_script();
  }
})();
