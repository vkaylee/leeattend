import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { useVersionCheck } from "../use-version-check";

describe("useVersionCheck", () => {
  beforeEach(() => {
    (globalThis as any).__BUILD_HASH__ = "abc123";
    globalThis.fetch = vi.fn();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("returns hasNewVersion=false when versions match", async () => {
    (globalThis.fetch as any).mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="abc123" /></head></html>',
        ),
    });

    const { result } = renderHook(() => useVersionCheck());

    await waitFor(() => expect(result.current.hasNewVersion).toBe(false));
  });

  it("sets hasNewVersion=true when build version differs", async () => {
    (globalThis.fetch as any).mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="def456" /></head></html>',
        ),
    });

    const { result } = renderHook(() => useVersionCheck());

    await waitFor(() => expect(result.current.hasNewVersion).toBe(true));
  });

  it("calls dismiss to reset hasNewVersion to false", async () => {
    (globalThis.fetch as any).mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="def456" /></head></html>',
        ),
    });

    const { result } = renderHook(() => useVersionCheck());

    await waitFor(() => expect(result.current.hasNewVersion).toBe(true));

    act(() => result.current.dismiss());
    expect(result.current.hasNewVersion).toBe(false);
  });

  it("fetches with cache-busting query param", async () => {
    const fetchFn = globalThis.fetch as any;
    fetchFn.mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="abc123" /></head></html>',
        ),
    });

    renderHook(() => useVersionCheck());

    await waitFor(() => {
      expect(fetchFn.mock.calls.length).toBeGreaterThan(0);
    });

    const url = fetchFn.mock.calls[0][0];
    expect(url).toMatch(/\/\?_v=\d+/);
  });

  it("sends no-cache header", async () => {
    const fetchFn = globalThis.fetch as any;
    fetchFn.mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="abc123" /></head></html>',
        ),
    });

    renderHook(() => useVersionCheck());

    await waitFor(() => {
      expect(fetchFn.mock.calls.length).toBeGreaterThan(0);
    });

    const options = fetchFn.mock.calls[0][1];
    expect(options.headers).toMatchObject({ "Cache-Control": "no-cache" });
  });

  it("does not throw on fetch failure", async () => {
    (globalThis.fetch as any).mockRejectedValue(new Error("network error"));

    const { result } = renderHook(() => useVersionCheck());

    await waitFor(() => expect(result.current.hasNewVersion).toBe(false));
  });
});
