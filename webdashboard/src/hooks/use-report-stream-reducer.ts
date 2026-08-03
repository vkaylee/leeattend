import type { StreamStatus } from "@/types/report-stream";

/**
 * Snapshot of the report-stream socket state at decision time.
 * Mirrors the mutable refs owned by `useReportStreamSocket` / the caller.
 */
export interface SocketCloseState {
  maxRetries: number;
  retryCount: number;
  shouldReconnect: boolean;
  status: StreamStatus;
}

export type SocketDecision =
  | { type: "auth_error" }
  | { type: "ignore" }
  | { type: "reconnect" }
  | { type: "give_up" };

/**
 * Pure state-machine reducer for WebSocket close events.
 * Returns the action the socket lifecycle should take; the React hook owns
 * the corresponding side effects (token refresh, backoff scheduling, error UI).
 */
export function reduceSocketState(
  state: SocketCloseState,
  event: { type: "close"; code: number },
): SocketDecision {
  const isAuthError =
    event.code === 1006 && state.retryCount === 0 && state.status === "connecting";
  if (isAuthError) return { type: "auth_error" };

  const isTerminal = state.status === "completed" || state.status === "error";
  if (isTerminal) return { type: "ignore" };

  if (state.shouldReconnect && state.retryCount < state.maxRetries) {
    return { type: "reconnect" };
  }

  if (state.retryCount >= state.maxRetries) return { type: "give_up" };

  return { type: "ignore" };
}
