const DEFAULT_MIN_SAMPLES = 4;
const DEFAULT_MIN_AGE_MS = 4_500;

export function responseObservationKey(response) {
  if (!response) {
    return null;
  }
  const body = String(response.body_text ?? response.raw_response_text ?? "").trim();
  if (!body) {
    return null;
  }
  return `${String(response.id ?? "no-id")}\u0000${body}`;
}

export function observeStableResponse(
  observation,
  response,
  nowMs,
  { minSamples = DEFAULT_MIN_SAMPLES, minAgeMs = DEFAULT_MIN_AGE_MS } = {}
) {
  const key = responseObservationKey(response);
  if (!key) {
    return {
      observation: null,
      stable: false,
    };
  }

  const previous = observation?.key === key ? observation : null;
  const next = {
    key,
    firstSeenAt: previous?.firstSeenAt ?? nowMs,
    samples: (previous?.samples ?? 0) + 1,
  };
  return {
    observation: next,
    stable: next.samples >= minSamples && nowMs - next.firstSeenAt >= minAgeMs,
  };
}
