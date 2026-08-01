const { test } = require("../support/chrome-extension");

test.describe("Live provider qualification backlog", () => {
  test.fixme(
    "Claude GUI roundtrip produces one completed, automatically ingested response",
    async () => {}
  );

  test.fixme(
    "Gemini GUI roundtrip produces one completed, automatically ingested response",
    async () => {}
  );

  test.fixme(
    "Grok GUI roundtrip produces one completed, automatically ingested response",
    async () => {}
  );

  test.fixme(
    "Draft mode preserves a local draft without changing any provider user-message count",
    async () => {}
  );

  test.fixme(
    "Copy mode copies the rendered package without changing any provider user-message count",
    async () => {}
  );

  test.fixme(
    "repeated provider polling and explicit sync never duplicate a canonical message",
    async () => {}
  );

  test.fixme(
    "one GUI broadcast to all four providers yields four completed unified responses",
    async () => {}
  );

  test.fixme(
    "directed context excludes the target provider's own prior assistant turn",
    async () => {}
  );
});
