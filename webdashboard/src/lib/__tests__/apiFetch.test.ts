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
    (fetch as any).mockResolvedValue({ ok: true, json: async () => ({}) });

    await apiFetch("/test");

    expect(fetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        signal: expect.any(AbortSignal),
      })
    );
  });

  it("should set Content-Type to application/json by default", async () => {
    (fetch as any).mockResolvedValue({ ok: true, json: async () => ({}) });

    await apiFetch("/test", { method: "POST", body: JSON.stringify({ data: 1 }) });

    expect(fetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        headers: expect.objectContaining({
          "Content-Type": "application/json",
        }),
      })
    );
  });

  it("should not set Content-Type if body is FormData", async () => {
    (fetch as any).mockResolvedValue({ ok: true, json: async () => ({}) });

    const formData = new FormData();
    formData.append("file", new Blob(["test"]), "test.txt");

    await apiFetch("/test", { method: "POST", body: formData });

    const callHeaders = (fetch as any).mock.calls[0][1].headers;
    expect(callHeaders["Content-Type"]).toBeUndefined();
  });

  it("should include X-Workspace-Id if available in localStorage", async () => {
    localStorage.setItem("active_workspace_id", "ws-123");
    (fetch as any).mockResolvedValue({ ok: true, json: async () => ({}) });

    await apiFetch("/test");

    expect(fetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        headers: expect.objectContaining({
          "X-Workspace-Id": "ws-123",
        }),
      })
    );
  });

  it("should include Authorization header if token is available", async () => {
    localStorage.setItem("access_token", "token-abc");
    (fetch as any).mockResolvedValue({ ok: true, json: async () => ({}) });

    await apiFetch("/test");

    expect(fetch).toHaveBeenCalledWith(
      expect.any(String),
      expect.objectContaining({
        headers: expect.objectContaining({
          "Authorization": "Bearer token-abc",
        }),
      })
    );
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
      (fetch as any).mockResolvedValue({ status: 401, ok: false });

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
      (fetch as any).mockResolvedValue({ status: 401, ok: false });

      await expect(apiFetch("/test")).rejects.toThrow("Session expired");

      // Auth should still be cleared
      expect(localStorage.getItem("access_token")).toBeNull();

      // Should NOT set href when already on /login
      expect(window.location.href).toBe("");
    });

    it("should throw error to stop caller execution on 401", async () => {
      (fetch as any).mockResolvedValue({ status: 401, ok: false });

      const promise = apiFetch("/test");

      await expect(promise).rejects.toThrow("Session expired");
    });
  });
});
