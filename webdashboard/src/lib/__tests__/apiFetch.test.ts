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
});
