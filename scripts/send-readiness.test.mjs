import test from "node:test";
import assert from "node:assert/strict";

import { isTransientSendControlError } from "../extension-src/common/send-readiness.mjs";

test("recognizes a provider send control that has not rendered yet", () => {
  assert.equal(
    isTransientSendControlError(
      new Error("not found: no visible and enabled Gemini send control found")
    ),
    true
  );
});

test("does not retry unrelated adapter failures", () => {
  assert.equal(isTransientSendControlError(new Error("conversation changed")), false);
  assert.equal(isTransientSendControlError(new Error("login required")), false);
});
