const runtimeApi = globalThis.browser ?? globalThis.chrome;

(async () => {
  const moduleUrl = runtimeApi.runtime.getURL("wasm/chatmux_adapter_gpt.js");
  const wasmModule = await import(moduleUrl);
  await wasmModule.default(runtimeApi.runtime.getURL("wasm/chatmux_adapter_gpt_bg.wasm"));
  if (typeof wasmModule.bootstrap_gpt_content_script === "function") {
    wasmModule.bootstrap_gpt_content_script();
  }
})();
