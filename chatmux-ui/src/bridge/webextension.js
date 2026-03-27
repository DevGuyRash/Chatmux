function globalApi(name) {
  return globalThis[name] ?? null;
}

function extensionRuntime() {
  const browserApi = globalApi("browser");
  if (browserApi?.runtime) {
    return { runtime: browserApi.runtime, style: "promise" };
  }

  const chromeApi = globalApi("chrome");
  if (chromeApi?.runtime) {
    return { runtime: chromeApi.runtime, style: "callback" };
  }

  throw new Error("extension runtime API is unavailable");
}

function extensionStorageLocal() {
  const browserApi = globalApi("browser");
  if (browserApi?.storage?.local) {
    return { area: browserApi.storage.local, style: "promise" };
  }

  const chromeApi = globalApi("chrome");
  if (chromeApi?.storage?.local) {
    return { area: chromeApi.storage.local, style: "callback" };
  }

  throw new Error("extension storage.local is unavailable");
}

function extensionPermissions() {
  const browserApi = globalApi("browser");
  if (browserApi?.permissions) {
    return { permissions: browserApi.permissions, style: "promise" };
  }

  const chromeApi = globalApi("chrome");
  if (chromeApi?.permissions) {
    return { permissions: chromeApi.permissions, style: "callback" };
  }

  throw new Error("extension permissions API is unavailable");
}

function extensionCommands() {
  const browserApi = globalApi("browser");
  if (browserApi?.commands) {
    return { commands: browserApi.commands, style: "promise" };
  }

  const chromeApi = globalApi("chrome");
  if (chromeApi?.commands) {
    return { commands: chromeApi.commands, style: "callback" };
  }

  throw new Error("extension commands API is unavailable");
}

function extensionTabs() {
  const browserApi = globalApi("browser");
  if (browserApi?.tabs) {
    return { tabs: browserApi.tabs, style: "promise" };
  }

  const chromeApi = globalApi("chrome");
  if (chromeApi?.tabs) {
    return { tabs: chromeApi.tabs, style: "callback" };
  }

  throw new Error("extension tabs API is unavailable");
}

function callbackResult(executor) {
  return new Promise((resolve, reject) => {
    executor((result) => {
      const runtimeError = globalThis.chrome?.runtime?.lastError;
      if (runtimeError) {
        reject(new Error(runtimeError.message || String(runtimeError)));
        return;
      }
      resolve(result);
    });
  });
}

export async function runtime_send_message(message) {
  const { runtime, style } = extensionRuntime();
  if (style === "promise") {
    return await runtime.sendMessage(message);
  }

  return await callbackResult((done) => runtime.sendMessage(message, done));
}

export function runtime_add_listener(callback) {
  const { runtime } = extensionRuntime();
  runtime.onMessage.addListener((message) => {
    callback(message);
    return false;
  });
}

export async function storage_local_get(key) {
  const { area, style } = extensionStorageLocal();
  if (style === "promise") {
    const result = await area.get(key);
    return result?.[key];
  }

  return await callbackResult((done) => area.get(key, done)).then((result) => result?.[key]);
}

export async function storage_local_set(key, value) {
  const { area, style } = extensionStorageLocal();
  const payload = { [key]: value };
  if (style === "promise") {
    await area.set(payload);
    return null;
  }

  await callbackResult((done) => area.set(payload, done));
  return null;
}

export async function storage_local_get_bytes_in_use() {
  const { area, style } = extensionStorageLocal();
  if (typeof area.getBytesInUse !== "function") {
    return 0;
  }

  if (style === "promise") {
    return await area.getBytesInUse(null);
  }

  return await callbackResult((done) => area.getBytesInUse(null, done));
}

export async function permissions_contains_origins(origins) {
  const { permissions, style } = extensionPermissions();
  const query = { origins: Array.from(origins ?? []) };
  if (style === "promise") {
    return await permissions.contains(query);
  }

  return await callbackResult((done) => permissions.contains(query, done));
}

export async function permissions_request_origins(origins) {
  const { permissions, style } = extensionPermissions();
  const request = { origins: Array.from(origins ?? []) };
  if (style === "promise") {
    return await permissions.request(request);
  }

  return await callbackResult((done) => permissions.request(request, done));
}

export async function commands_get_all() {
  const { commands, style } = extensionCommands();
  if (style === "promise") {
    return await commands.getAll();
  }

  return await callbackResult((done) => commands.getAll(done));
}

export async function tabs_open(url) {
  const { tabs, style } = extensionTabs();
  if (style === "promise") {
    return await tabs.create({ url });
  }

  return await callbackResult((done) => tabs.create({ url }, done));
}

export async function tabs_query(urlPatterns) {
  const { tabs, style } = extensionTabs();
  const query = Array.isArray(urlPatterns) && urlPatterns.length > 0
    ? { url: urlPatterns }
    : {};
  if (style === "promise") {
    return await tabs.query(query);
  }

  return await callbackResult((done) => tabs.query(query, done));
}

export async function clipboard_write_text(text) {
  const clipboard = globalThis.navigator?.clipboard;
  if (!clipboard?.writeText) {
    throw new Error("clipboard API is unavailable");
  }

  await clipboard.writeText(text);
  return null;
}
