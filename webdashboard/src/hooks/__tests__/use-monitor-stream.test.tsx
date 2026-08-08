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

  class MockEventSource {
    url: string;
    readyState = 0;
    
    constructor(url: string, options?: Record<string, unknown>) {
      capturedUrl = url;
      capturedOptions = options;
      this.url = url;
    }
    
    addEventListener = vi.fn();
    removeEventListener = vi.fn();
    close = vi.fn();
    dispatchEvent = vi.fn();
  }

  beforeEach(() => {
    capturedUrl = undefined;
    capturedOptions = undefined;
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
});
