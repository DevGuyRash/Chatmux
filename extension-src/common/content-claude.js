const runtimeApi = globalThis.browser ?? globalThis.chrome;

(async () => {
  const moduleUrl = runtimeApi.runtime.getURL("wasm/chatmux_adapter_claude.js");
  const wasmModule = await import(moduleUrl);
  await wasmModule.default(runtimeApi.runtime.getURL("wasm/chatmux_adapter_claude_bg.wasm"));
  if (typeof wasmModule.bootstrap_claude_content_script === "function") {
    wasmModule.bootstrap_claude_content_script();
  }
})();
