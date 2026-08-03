import { describe, expect, it } from "vitest";

import type { SocketCloseState } from "../use-report-stream-reducer";
import { reduceSocketState } from "../use-report-stream-reducer";

function socketState(overrides: Partial<SocketCloseState> = {}): SocketCloseState {
  return {
    status: "streaming",
    retryCount: 0,
    shouldReconnect: true,
    maxRetries: 3,
    ...overrides,
  };
}

describe("reduceSocketState", () => {
  it("returns auth_error for 1006 close while connecting with zero retries", () => {
    expect(
      reduceSocketState(socketState({ status: "connecting", retryCount: 0 }), {
        type: "close",
        code: 1006,
      }),
    ).toEqual({ type: "auth_error" });
  });

  it("does not treat a 1006 close as auth error once retries were used", () => {
    expect(
      reduceSocketState(socketState({ status: "connecting", retryCount: 2 }), {
        type: "close",
        code: 1006,
      }),
    ).toEqual({ type: "reconnect" });
  });

  it("does not treat a clean close (code 1000) as auth error", () => {
    expect(
      reduceSocketState(socketState({ status: "connecting", retryCount: 0 }), {
        type: "close",
        code: 1000,
      }),
    ).toEqual({ type: "reconnect" });
  });

  it("ignores close events after status completed", () => {
    expect(
      reduceSocketState(socketState({ status: "completed" }), { type: "close", code: 1006 }),
    ).toEqual({ type: "ignore" });
  });

  it("ignores close events after status error", () => {
    expect(
      reduceSocketState(socketState({ status: "error" }), { type: "close", code: 1006 }),
    ).toEqual({ type: "ignore" });
  });

  it("schedules reconnect while retries remain", () => {
    expect(
      reduceSocketState(socketState({ status: "streaming", retryCount: 1 }), {
        type: "close",
        code: 1006,
      }),
    ).toEqual({ type: "reconnect" });
  });

  it("gives up when retry count reaches maxRetries", () => {
    expect(
      reduceSocketState(socketState({ status: "streaming", retryCount: 3 }), {
        type: "close",
        code: 1000,
      }),
    ).toEqual({ type: "give_up" });
  });

  it("ignores unexpected close when reconnection is disabled and retries remain", () => {
    expect(
      reduceSocketState(socketState({ status: "streaming", retryCount: 1, shouldReconnect: false }), {
        type: "close",
        code: 1000,
      }),
    ).toEqual({ type: "ignore" });
  });

  it("gives up even when reconnection is disabled once retries are exhausted", () => {
    expect(
      reduceSocketState(socketState({ status: "streaming", retryCount: 4, shouldReconnect: false }), {
        type: "close",
        code: 1000,
      }),
    ).toEqual({ type: "give_up" });
  });
});
