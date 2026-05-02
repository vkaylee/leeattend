import { renderHook, act, waitFor } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { useVersionCheck } from "../use-version-check";

describe("useVersionCheck", () => {
  beforeEach(() => {
 
    (globalThis as unknown as { __BUILD_HASH__: string }).__BUILD_HASH__ = "abc123";
    globalThis.fetch = vi.fn();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("returns hasNewVersion=false when versions match", async () => {
 
    vi.mocked(globalThis.fetch).mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="abc123" /></head></html>',
        ),
    } as unknown as never);

    const { result } = renderHook(() => useVersionCheck());

    await waitFor(() => { expect(result.current.hasNewVersion).toBe(false); });
  });

  it("sets hasNewVersion=true when build version differs", async () => {
 
    vi.mocked(globalThis.fetch).mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="def456" /></head></html>',
        ),
    } as unknown as never);

    const { result } = renderHook(() => useVersionCheck());

    await waitFor(() => { expect(result.current.hasNewVersion).toBe(true); });
  });

  it("calls dismiss to reset hasNewVersion to false", async () => {
 
    vi.mocked(globalThis.fetch).mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="def456" /></head></html>',
        ),
    } as unknown as never);

    const { result } = renderHook(() => useVersionCheck());

    await waitFor(() => { expect(result.current.hasNewVersion).toBe(true); });

    act(() => { result.current.dismiss(); });
    expect(result.current.hasNewVersion).toBe(false);
  });

  it("fetches with cache-busting query param", async () => {
 
    const fetchFunction = vi.mocked(globalThis.fetch);
    fetchFunction.mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="abc123" /></head></html>',
        ),
    } as unknown as never);

    renderHook(() => useVersionCheck());

    await waitFor(() => {
      expect(fetchFunction.mock.calls.length).toBeGreaterThan(0);
    });

    const url = fetchFunction.mock.calls[0][0];
    expect(url).toMatch(/\/\?_v=\d+/);
  });

  it("sends no-cache header", async () => {
 
    const fetchFunction = vi.mocked(globalThis.fetch);
    fetchFunction.mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="abc123" /></head></html>',
        ),
    } as unknown as never);

    renderHook(() => useVersionCheck());

    await waitFor(() => {
      expect(fetchFunction.mock.calls.length).toBeGreaterThan(0);
    });

    const options = fetchFunction.mock.calls[0][1]!;
    expect(options.headers).toMatchObject({ "Cache-Control": "no-cache" });
  });

  it("does not throw on fetch failure", async () => {
 
    vi.mocked(globalThis.fetch).mockRejectedValue(new Error("network error"));

    const { result } = renderHook(() => useVersionCheck());

    await waitFor(() => { expect(result.current.hasNewVersion).toBe(false); });
  });
});
