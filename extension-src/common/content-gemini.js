const runtimeApi = globalThis.browser ?? globalThis.chrome;

(async () => {
  const moduleUrl = runtimeApi.runtime.getURL("wasm/chatmux_adapter_gemini.js");
  const wasmModule = await import(moduleUrl);
  await wasmModule.default(runtimeApi.runtime.getURL("wasm/chatmux_adapter_gemini_bg.wasm"));
  if (typeof wasmModule.bootstrap_gemini_content_script === "function") {
    wasmModule.bootstrap_gemini_content_script();
  }
})();
