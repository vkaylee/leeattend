import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { apiFetch, getAccessToken, getCurrentUserId, setAccessToken, setCurrentUserId } from "../api/helpers";

describe("apiFetch utility", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
    vi.useFakeTimers();
    localStorage.clear();
    // Tokens now live in the in-memory store (lib/api/token-store.ts).
    setAccessToken(null);
    setCurrentUserId(null);
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
      Object.defineProperty(window, "location", {
        value: { pathname: "/dashboard", href: "" },
        writable: true,
      });
    });
    afterEach(() => {
      Object.defineProperty(window, "location", {
        value: originalLocation,
        writable: true,
      });
    });

    it("should clear auth session and redirect on 401 response", async () => {
      // Pre-condition: tokens exist in the in-memory store.
      setAccessToken("stale");
      setCurrentUserId("u-1");

      vi.mocked(fetch).mockResolvedValue({ status: 401, ok: false } as unknown as Response);

      await expect(apiFetch("/test")).rejects.toThrow("Session expired");

      // Verify all auth keys are cleared from the in-memory store.
      // (Tokens no longer live in localStorage — see lib/api/token-store.ts.)
      expect(getAccessToken()).toBeNull();
      expect(getCurrentUserId()).toBeNull();

      // Verify redirect to login
      expect(window.location.href).toBe("/login");
    });

    it("should not redirect if already on login page", async () => {
      setAccessToken("stale");
      window.location.pathname = "/login";
      vi.mocked(fetch).mockResolvedValue({ status: 401, ok: false } as unknown as Response);

      await expect(apiFetch("/test")).rejects.toThrow("Session expired");

      // Auth should still be cleared from the in-memory store.
      expect(getAccessToken()).toBeNull();

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
