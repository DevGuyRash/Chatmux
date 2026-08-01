const { safeText, safeUrl } = require("./providers/provider-surface");

const MAX_ENTRIES = 200;

function createBrowserDiagnostics(testInfo) {
  const entries = [];
  let droppedEntries = 0;
  const observedPages = new WeakSet();
  const observedWorkers = new WeakSet();

  function record(entry) {
    if (entries.length >= MAX_ENTRIES) {
      droppedEntries += 1;
      return;
    }
    entries.push({
      timestamp: new Date().toISOString(),
      ...entry,
      url: safeUrl(entry.url || ""),
      message: safeText(entry.message || "", 1_000),
    });
  }

  function observePage(page) {
    if (observedPages.has(page)) {
      return;
    }
    observedPages.add(page);

    page.on("pageerror", (error) => {
      record({
        kind: "pageerror",
        level: "error",
        url: page.url(),
        message: error?.message || error,
      });
    });
    page.on("crash", () => {
      record({
        kind: "page-crash",
        level: "error",
        url: page.url(),
        message: "The browser page crashed.",
      });
    });
  }

  function observeWorker(worker) {
    if (observedWorkers.has(worker)) {
      return;
    }
    observedWorkers.add(worker);
    record({
      kind: "service-worker",
      level: "info",
      url: worker.url(),
      message: "Service worker observed.",
    });
    worker.on("close", () => {
      record({
        kind: "service-worker",
        level: "info",
        url: worker.url(),
        message: "Service worker closed.",
      });
    });
  }

  function observeContext(context) {
    for (const page of context.pages()) {
      observePage(page);
    }
    for (const worker of context.serviceWorkers()) {
      observeWorker(worker);
    }

    context.on("page", observePage);
    context.on("serviceworker", observeWorker);
    context.on("console", (message) => {
      const page = message.page();
      const location = message.location();
      record({
        kind: "console",
        level: message.type(),
        url: page?.url() || location.url || "",
        message: message.text(),
        location: {
          url: safeUrl(location.url || ""),
          lineNumber: location.lineNumber,
          columnNumber: location.columnNumber,
        },
      });
    });
    context.on("weberror", (webError) => {
      const page = webError.page();
      const error = webError.error();
      record({
        kind: "weberror",
        level: "error",
        url: page?.url() || "",
        message: error?.message || error,
      });
    });
  }

  function report() {
    return {
      test: testInfo.titlePath,
      entries,
      droppedEntries,
    };
  }

  async function attachIfUseful(force = false) {
    const hasNoteworthyEntry = entries.some((entry) =>
      ["warning", "error", "assert"].includes(entry.level)
    );
    if (!force && !hasNoteworthyEntry && droppedEntries === 0) {
      return;
    }
    await testInfo.attach("browser-diagnostics.json", {
      body: Buffer.from(JSON.stringify(report(), null, 2)),
      contentType: "application/json",
    });
  }

  function productErrors(extensionId) {
    if (!extensionId) {
      return [];
    }
    const extensionOrigin = `chrome-extension://${extensionId}/`;
    return entries.filter(
      (entry) =>
        entry.level === "error" &&
        typeof entry.url === "string" &&
        entry.url.startsWith(extensionOrigin)
    );
  }

  return {
    attachIfUseful,
    observeContext,
    productErrors,
    report,
  };
}

module.exports = { createBrowserDiagnostics };
