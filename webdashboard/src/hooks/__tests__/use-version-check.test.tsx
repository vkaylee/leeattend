import { renderHook, act, waitFor } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { useVersionCheck } from "../use-version-check";

describe("useVersionCheck", () => {
  beforeEach(() => {

    (globalThis as unknown as { __BUILD_HASH__: string }).__BUILD_HASH__ = "abc123";
    globalThis.fetch = vi.fn();
    sessionStorage.clear();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    sessionStorage.clear();
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

  it("does not re-show the banner for a version already dismissed", async () => {

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

    // Re-check with the SAME server hash — must stay dismissed (no nag).
    await act(async () => {
      document.dispatchEvent(new Event("visibilitychange"));
    });

    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(result.current.hasNewVersion).toBe(false);
  });

  it("stays dismissed across a fresh mount (reload) for the same version", async () => {
    vi.mocked(globalThis.fetch).mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="def456" /></head></html>',
        ),
    } as unknown as never);

    // First mount: banner shows, user dismisses.
    const first = renderHook(() => useVersionCheck());
    await waitFor(() => { expect(first.result.current.hasNewVersion).toBe(true); });
    act(() => { first.result.current.dismiss(); });
    expect(first.result.current.hasNewVersion).toBe(false);

    // Simulate a page reload: unmount, then mount a brand-new hook instance
    // (the in-memory ref resets, only sessionStorage survives).
    first.unmount();

    const second = renderHook(() => useVersionCheck());
    // Same server hash still deployed — the dismissal must persist via
    // sessionStorage even though the ref started fresh.
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(second.result.current.hasNewVersion).toBe(false);
  });

  it("re-shows the banner when a newer version is deployed after dismiss", async () => {

    const fetchFunction = vi.mocked(globalThis.fetch);
    fetchFunction.mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="def456" /></head></html>',
        ),
    } as unknown as never);

    const { result } = renderHook(() => useVersionCheck());

    await waitFor(() => { expect(result.current.hasNewVersion).toBe(true); });

    act(() => { result.current.dismiss(); });
    expect(result.current.hasNewVersion).toBe(false);

    // A newer deploy ships an even newer hash — banner must return.
    fetchFunction.mockResolvedValue({
      text: () =>
        Promise.resolve(
          '<html><head><meta name="build-version" content="ghi789" /></head></html>',
        ),
    } as unknown as never);

    await act(async () => {
      document.dispatchEvent(new Event("visibilitychange"));
    });

    await waitFor(() => { expect(result.current.hasNewVersion).toBe(true); });
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
