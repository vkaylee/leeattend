import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { apiFetch, getAccessToken, getCurrentUserId, setAccessToken, setCurrentUserId } from "../api/helpers";

function makeToken(expiresAtSeconds: number): string {
  const payload = btoa(JSON.stringify({ exp: expiresAtSeconds }));
  return `header.${payload.replace(/=/g, "")}.signature`;
}

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

  it("refreshes an access token before requesting a protected endpoint when it is near expiry", async () => {
    setAccessToken(makeToken(Math.floor(Date.now() / 1000) + 30));

    vi.mocked(fetch)
      .mockResolvedValueOnce(new Response(JSON.stringify({ access_token: "fresh-token" }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ ok: true }), { status: 200 }));

    const response = await apiFetch("/test");

    expect(response.status).toBe(200);
    expect(fetch).toHaveBeenCalledTimes(2);
    expect(vi.mocked(fetch).mock.calls[0][0]).toContain("/auth/refresh");
    expect(vi.mocked(fetch).mock.calls[1][0]).toContain("/test");
    expect(((vi.mocked(fetch).mock.calls[1][1] as RequestInit).headers as Headers).get("Authorization")).toBe("Bearer fresh-token");
  });

  it("does not call a protected endpoint when preflight refresh reports an expired session", async () => {
    setAccessToken(makeToken(Math.floor(Date.now() / 1000) - 1));
    vi.mocked(fetch).mockResolvedValue(new Response(JSON.stringify({ ok: true }), { status: 200 }));
    vi.mocked(fetch).mockResolvedValueOnce(new Response(null, { status: 401 }));

    await expect(apiFetch("/test")).rejects.toThrow("Session expired");

    expect(fetch).toHaveBeenCalledTimes(2);
    expect(vi.mocked(fetch).mock.calls[0][0]).toContain("/auth/refresh");
    expect(vi.mocked(fetch).mock.calls[1][0]).toContain("/auth/logout");
    expect(getAccessToken()).toBeNull();
  });

  it("refreshes once and lets concurrent protected requests continue with the new token", async () => {
    setAccessToken(makeToken(Math.floor(Date.now() / 1000) + 30));

    const { promise: refreshResponse, resolve: resolveRefresh } = Promise.withResolvers<Response>();
    const mockFetch = vi.mocked(fetch);
    mockFetch.mockImplementation((input) => {
      if (String(input).includes("/auth/refresh")) return refreshResponse;
      return Promise.resolve(new Response(JSON.stringify({ ok: true }), { status: 200 }));
    });

    const requests = [apiFetch("/employees"), apiFetch("/departments"), apiFetch("/notifications")];
    await Promise.resolve();

    expect(mockFetch.mock.calls.filter(([input]) => String(input).includes("/auth/refresh"))).toHaveLength(1);
    expect(mockFetch.mock.calls.filter(([input]) => String(input).includes("/employees"))).toHaveLength(0);

    resolveRefresh(new Response(JSON.stringify({ access_token: "fresh-token" }), { status: 200 }));
    const responses = await Promise.all(requests);

    expect(responses).toHaveLength(3);
    const protectedCalls = mockFetch.mock.calls.filter(([input]) => !String(input).includes("/auth/refresh"));
    expect(protectedCalls).toHaveLength(3);
    for (const [, init] of protectedCalls) {
      expect((init?.headers as Headers).get("Authorization")).toBe("Bearer fresh-token");
    }
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
