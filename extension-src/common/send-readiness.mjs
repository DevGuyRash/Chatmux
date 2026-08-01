const TRANSIENT_SEND_CONTROL_PATTERNS = [
  "no visible and enabled",
  "send control found",
];

export function isTransientSendControlError(error) {
  const detail = String(error?.message ?? error ?? "").toLowerCase();
  return TRANSIENT_SEND_CONTROL_PATTERNS.every((pattern) => detail.includes(pattern));
}
