import assert from "node:assert/strict";
import test from "node:test";

import {
  observeStableResponse,
  responseObservationKey,
} from "../extension-src/common/completion-stability.mjs";

test("empty responses never become stable", () => {
  assert.equal(responseObservationKey(null), null);
  assert.equal(responseObservationKey({ id: "one", body_text: " " }), null);
  assert.equal(observeStableResponse(null, null, 0).stable, false);
});

test("streaming body changes reset the stability window", () => {
  let state = null;
  ({ observation: state } = observeStableResponse(state, { id: "one", body_text: "partial" }, 0));
  ({ observation: state } = observeStableResponse(state, { id: "one", body_text: "partial" }, 2_000));
  const changed = observeStableResponse(state, { id: "one", body_text: "complete" }, 5_000);

  assert.equal(changed.stable, false);
  assert.equal(changed.observation.samples, 1);
  assert.equal(changed.observation.firstSeenAt, 5_000);
});

test("unchanged new response becomes a bounded completion fallback", () => {
  let state = null;
  let result;
  for (const now of [0, 1_500, 3_000, 4_500]) {
    result = observeStableResponse(state, { id: "one", body_text: "complete" }, now);
    state = result.observation;
  }

  assert.equal(result.stable, true);
  assert.equal(state.samples, 4);
});
