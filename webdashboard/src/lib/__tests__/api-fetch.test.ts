import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { apiFetch } from "../api/helpers";

describe("apiFetch utility", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
    vi.useFakeTimers();
    localStorage.clear();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it("should setup a timeout via AbortController", async () => {
 
    vi.mocked(fetch).mockResolvedValue({ ok: true, json: () => Promise.resolve({}) } as unknown as Response);

    await apiFetch("/test");

    expect(fetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        signal: expect.any(AbortSignal) as unknown as AbortSignal,
      }) as RequestInit
    );
  });

  it("should set Content-Type to application/json by default", async () => {
 
    vi.mocked(fetch).mockResolvedValue({ ok: true, json: () => Promise.resolve({}) } as unknown as Response);

    await apiFetch("/test", { method: "POST", body: JSON.stringify({ data: 1 }) });

    const callInit = vi.mocked(fetch).mock.calls[0][1]!;
    expect(callInit.headers).toBeInstanceOf(Headers);
    expect((callInit.headers as Headers).get("Content-Type")).toBe("application/json");
  });

  it("should deduplicate Content-Type headers regardless of case", async () => {
    vi.mocked(fetch).mockResolvedValue({ ok: true, json: () => Promise.resolve({}) } as unknown as Response);

    await apiFetch("/test", {
      method: "POST",
      body: JSON.stringify({ data: 1 }),
      headers: { "content-type": "application/json" }
    });

    const callInit = vi.mocked(fetch).mock.calls[0][1]!;
    const headers = callInit.headers as Headers;
    expect(headers).toBeInstanceOf(Headers);
    // There should only be one Content-Type header in the Headers object
    // And it should map to 'application/json' (not 'application/json, application/json')
    expect(headers.get("Content-Type")).toBe("application/json");
    expect(headers.get("content-type")).toBe("application/json");
    
    // We can iterate to verify no duplicates exist in the Headers object iteration
    let count = 0;
    headers.forEach((_, key) => {
      if (key.toLowerCase() === "content-type") count++;
    });
    expect(count).toBe(1);
  });

  it("should not set Content-Type if body is FormData", async () => {
 
    vi.mocked(fetch).mockResolvedValue({ ok: true, json: () => Promise.resolve({}) } as unknown as Response);

    await apiFetch("/test", { method: "POST", body: new FormData() });

    const callHeaders = (vi.mocked(fetch).mock.calls[0][1]!).headers as Headers;
    expect(callHeaders.get("Content-Type")).toBeNull();
  });

  it("should include X-Workspace-Id if available in localStorage", async () => {
 
    localStorage.setItem("active_workspace_id", "ws-123");
    vi.mocked(fetch).mockResolvedValue({ ok: true, json: () => Promise.resolve({}) } as unknown as Response);

    await apiFetch("/test");

    const callHeaders = (vi.mocked(fetch).mock.calls[0][1]!).headers as Headers;
    expect(callHeaders.get("X-Workspace-Id")).toBe("ws-123");
  });

  it("should include Authorization header if token is available", async () => {
    localStorage.setItem("access_token", "token-abc");
    vi.mocked(fetch).mockResolvedValue({ ok: true, json: () => Promise.resolve({}) } as unknown as Response);

    await apiFetch("/test");

    const callHeaders = (vi.mocked(fetch).mock.calls[0][1]!).headers as Headers;
    expect(callHeaders.get("Authorization")).toBe("Bearer token-abc");
  });

  describe("401 Unauthorized handling", () => {
    const originalLocation = window.location;

    beforeEach(() => {
      // Mock window.location
      Object.defineProperty(window, "location", {
        value: { pathname: "/dashboard", href: "" },
        writable: true,
      });
      // Set auth tokens to verify they get cleared
      localStorage.setItem("access_token", "test-token");
      localStorage.setItem("refresh_token", "test-refresh");
      localStorage.setItem("active_user_id", "user-123");
      localStorage.setItem("active_workspace_id", "ws-123");
    });

    afterEach(() => {
      Object.defineProperty(window, "location", {
        value: originalLocation,
        writable: true,
      });
    });

    it("should clear auth session and redirect on 401 response", async () => {
      vi.mocked(fetch).mockResolvedValue({ status: 401, ok: false } as unknown as Response);

      await expect(apiFetch("/test")).rejects.toThrow("Session expired");

      // Verify all auth keys are cleared
 
      expect(localStorage.getItem("access_token")).toBeNull();
      expect(localStorage.getItem("refresh_token")).toBeNull();
      expect(localStorage.getItem("active_user_id")).toBeNull();
      expect(localStorage.getItem("active_workspace_id")).toBeNull();

      // Verify redirect to login
      expect(window.location.href).toBe("/login");
    });

    it("should not redirect if already on login page", async () => {
      window.location.pathname = "/login";
      vi.mocked(fetch).mockResolvedValue({ status: 401, ok: false } as unknown as Response);

      await expect(apiFetch("/test")).rejects.toThrow("Session expired");

      // Auth should still be cleared
 
      expect(localStorage.getItem("access_token")).toBeNull();

      // Should NOT set href when already on /login
      expect(window.location.href).toBe("");
    });

    it("should throw error to stop caller execution on 401", async () => {
      vi.mocked(fetch).mockResolvedValue({ status: 401, ok: false } as unknown as Response);

      const promise = apiFetch("/test");

      await expect(promise).rejects.toThrow("Session expired");
 
    });
  });
});
