import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { useMonitorStream } from "../use-monitor-stream";

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

describe("useMonitorStream Hook", () => {
  let capturedUrl: string | undefined;
  let capturedOptions: Record<string, unknown> | undefined;
  interface MockEventSourceHandle {
    emit(type: string, data: unknown): void;
  }

  let currentEventSource: MockEventSourceHandle | undefined;

  const metric = {
    unit: "jobs",
    formula: "test",
    source: "test",
    value: 0,
    observed_at: "2026-08-11T12:00:00.000Z",
    time_window: {
      start: "2026-08-11T11:45:00.000Z",
      end: "2026-08-11T12:00:00.000Z",
      duration_seconds: 900,
    },
    unknown: null,
  };
  const queueHealth = Object.fromEntries([
    "dead_letter_depth", "depth_high", "depth_low", "depth_normal", "depth_total",
    "enqueue_rate_per_second", "estimated_drain_seconds", "growth_rate_per_second",
    "oldest_queued_age_seconds", "processing_rate_per_second", "queue_wait_seconds_p50", "queue_wait_seconds_p95",
  ].map((field) => [field, metric]));
  const reliability = Object.fromEntries([
    "consecutive_failures", "dead_letter_count", "execution_seconds_p50", "execution_seconds_p95",
    "execution_seconds_p99", "failure_rate", "lost_count", "retryable_count", "success_rate", "timeout_count",
  ].map((field) => [field, metric]));
  const thresholds = {
    source: "backend_default",
    queue_pressure: { medium_at: 1, high_at: 2, critical_at: 3 },
    worker_utilization: { scale_down_below_percent: 20, warning_at_percent: 60, critical_at_percent: 80 },
  };

  class MockEventSource {
    url: string;
    readyState = 0;
    listeners: Record<string, (event: MessageEvent) => void> = {};

    constructor(url: string, options?: Record<string, unknown>) {
      capturedUrl = url;
      capturedOptions = options;
      currentEventSource = {
        emit: (type, data) => this.listeners[type]?.({ data: JSON.stringify(data) } as MessageEvent),
      };
      this.url = url;
    }

    addEventListener = vi.fn(
      (type: string, listener: (event: MessageEvent) => void) => {
        this.listeners[type] = listener;
      },
    );
    removeEventListener = vi.fn();
    close = vi.fn();
    dispatchEvent = vi.fn();

    emit(type: string, data: unknown) {
      this.listeners[type]?.({ data: JSON.stringify(data) } as MessageEvent);
    }
  }

  beforeEach(() => {
    capturedUrl = undefined;
    capturedOptions = undefined;
    currentEventSource = undefined;
    vi.stubGlobal("EventSource", MockEventSource);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
    localStorage.clear();
  });

  it("should never append an access token to the SSE URL", async () => {
    localStorage.setItem("access_token", "test-token-123");

    renderHook(() => useMonitorStream(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(capturedUrl).toBeDefined();
    });

    expect(capturedUrl).not.toContain("access_token");
  });

  it("should not append access_token if not available in localStorage", async () => {
    localStorage.removeItem("access_token");

    renderHook(() => useMonitorStream(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(capturedUrl).toBeDefined();
    });

    expect(capturedUrl).not.toContain("?access_token=");
  });

  it("should include withCredentials: true option", async () => {
    renderHook(() => useMonitorStream(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(capturedOptions).toBeDefined();
    });

    expect(capturedOptions).toEqual({ withCredentials: true });
  });

  it("marks a snapshot stale when its measured age exceeds the freshness window", async () => {
    const { result } = renderHook(() => useMonitorStream(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(currentEventSource).toBeDefined();
    });

    currentEventSource?.emit("monitor-update", {
      active_workers: 0,
      completed_today: 0,
      durable_jobs: { status: "available", jobs: [] },
      dependencies: {},
      failed_today: 0,
      queue_health: queueHealth,
      reliability,
      thresholds,
      idle_workers: 0,
      job_type_stats: [],
      observed_at: "2026-08-11T12:00:00.000Z",
      pending_high: 0,
      pending_low: 0,
      pending_normal: 0,
      pending_tasks: 0,
      quality: {
        data_quality: "complete",
        durable_db: { status: "available", error_code: null },
        freshness: "stale",
        redis: { status: "available", error_code: null },
        snapshot_sequence: "100-stale",
        unknown_fields: [],
        worker_heartbeats: { status: "available", error_code: null },
      },
      recent_jobs: [],
      snapshot_age_seconds: 16,
      total_workers: 0,
      workers: [],
    });

    await waitFor(() => {
      expect(result.current.status).toBe("stale");
    });
    expect(result.current.isFresh).toBe(false);
    expect(result.current.errorCode).toBe(null);
  });

  it("accepts nullable metrics and reports a fresh partial snapshot as degraded", async () => {
    const { result } = renderHook(() => useMonitorStream(), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(currentEventSource).toBeDefined();
    });

    currentEventSource?.emit("monitor-update", {
      active_workers: null,
      completed_today: null,
      durable_jobs: { status: "available", jobs: [] },
      dependencies: {},
      failed_today: null,
      idle_workers: null,
      job_type_stats: [],
      queue_health: queueHealth,
      reliability,
      thresholds,
      observed_at: "2026-08-11T12:00:00.000Z",
      pending_high: null,
      pending_low: null,
      pending_normal: null,
      pending_tasks: null,
      quality: {
        data_quality: "partial",
        durable_db: { status: "available", error_code: null },
        freshness: "fresh",
        redis: { status: "unavailable", error_code: "REDIS_MONITOR_TIMEOUT" },
        snapshot_sequence: "101-nullable",
        unknown_fields: ["pending_tasks", "workers"],
        worker_heartbeats: { status: "unavailable", error_code: "WORKER_HEARTBEATS_UNAVAILABLE" },
      },
      recent_jobs: [],
      snapshot_age_seconds: 0,
      total_workers: null,
      workers: [],
    });

    await waitFor(() => {
      expect(result.current.status).toBe("degraded");
    });
    expect(result.current.isFresh).toBe(true);
    expect(result.current.errorCode).toBe(null);
  });
});
