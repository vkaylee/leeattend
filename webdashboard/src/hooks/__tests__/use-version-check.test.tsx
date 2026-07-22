import { renderHook, act, waitFor } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

import { useVersionCheck } from "../use-version-check";

function manifestWith(hash: string) {
  return {
    ok: true,
    json: () => Promise.resolve({ buildId: hash }),
  } as unknown as never;
}

// Flush pending microtasks + the async check() body (fetch → text → state).
// A macrotask tick guarantees all queued microtasks have drained first.
async function flush() {
  await act(async () => {
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
}

// Fire one more check() via the visibilitychange path and let it settle.
async function fireCheck() {
  await act(async () => {
    document.dispatchEvent(new Event("visibilitychange"));
  });
  await flush();
}

describe("useVersionCheck", () => {
  beforeEach(() => {
    (globalThis as unknown as { __BUILD_HASH__: string }).__BUILD_HASH__ =
      "abc123";
    globalThis.fetch = vi.fn();
    sessionStorage.clear();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    sessionStorage.clear();
  });

  it("returns hasNewVersion=false when versions match", async () => {
    vi.mocked(globalThis.fetch).mockResolvedValue(manifestWith("abc123"));

    const { result } = renderHook(() => useVersionCheck());

    await flush();
    await fireCheck();
    expect(result.current.hasNewVersion).toBe(false);
  });

  it("does NOT show the banner on a single sighting of a new version", async () => {
    vi.mocked(globalThis.fetch).mockResolvedValue(manifestWith("def456"));

    const { result } = renderHook(() => useVersionCheck());

    // Only the mount check has run — one sighting is not yet trusted.
    await flush();
    expect(result.current.hasNewVersion).toBe(false);
  });

  it("shows the banner after two consecutive checks confirm the same new hash", async () => {
    vi.mocked(globalThis.fetch).mockResolvedValue(manifestWith("def456"));

    const { result } = renderHook(() => useVersionCheck());

    await flush(); // check #1 → pending sighting
    await fireCheck(); // check #2 → confirmed
    await waitFor(() => {
      expect(result.current.hasNewVersion).toBe(true);
    });
  });

  it("does not re-show a build that was already attempted during edge convergence", async () => {
    sessionStorage.setItem(
      "versionReloadAttempt",
      JSON.stringify({ buildId: "def456", attemptedAt: Date.now() }),
    );
    vi.mocked(globalThis.fetch).mockResolvedValue(manifestWith("def456"));

    const { result } = renderHook(() => useVersionCheck());
    await flush();
    await fireCheck();

    expect(result.current.hasNewVersion).toBe(false);
  });

  it("hides an active banner when the manifest converges back to the running build", async () => {
    const fetchFunction = vi.mocked(globalThis.fetch);
    fetchFunction.mockResolvedValue(manifestWith("def456"));
    const { result } = renderHook(() => useVersionCheck());
    await flush();
    await fireCheck();
    await waitFor(() => { expect(result.current.hasNewVersion).toBe(true); });

    fetchFunction.mockResolvedValue(manifestWith("abc123"));
    await fireCheck();

    expect(result.current.hasNewVersion).toBe(false);
  });

  it("does NOT confirm if the new hash flaps back to the current one", async () => {
    const fetchFunction = vi.mocked(globalThis.fetch);
    fetchFunction.mockResolvedValueOnce(manifestWith("def456")); // #1 mount → pending
    fetchFunction.mockResolvedValueOnce(manifestWith("abc123")); // #2 → flap back, clears pending
    fetchFunction.mockResolvedValueOnce(manifestWith("def456")); // #3 → pending again
    fetchFunction.mockResolvedValue(manifestWith("def456")); // #4+ → confirm

    const { result } = renderHook(() => useVersionCheck());

    await flush(); // #1
    await fireCheck(); // #2 flap back
    await fireCheck(); // #3 pending again (not two-in-a-row yet)
    expect(result.current.hasNewVersion).toBe(false);

    await fireCheck(); // #4 confirms
    await waitFor(() => {
      expect(result.current.hasNewVersion).toBe(true);
    });
  });

  it("calls dismiss to reset hasNewVersion to false", async () => {
    vi.mocked(globalThis.fetch).mockResolvedValue(manifestWith("def456"));

    const { result } = renderHook(() => useVersionCheck());

    await flush();
    await fireCheck();
    await waitFor(() => {
      expect(result.current.hasNewVersion).toBe(true);
    });

    act(() => {
      result.current.dismiss();
    });
    expect(result.current.hasNewVersion).toBe(false);
  });

  it("does not re-show the banner for a version already dismissed", async () => {
    vi.mocked(globalThis.fetch).mockResolvedValue(manifestWith("def456"));

    const { result } = renderHook(() => useVersionCheck());

    await flush();
    await fireCheck();
    await waitFor(() => {
      expect(result.current.hasNewVersion).toBe(true);
    });

    act(() => {
      result.current.dismiss();
    });
    expect(result.current.hasNewVersion).toBe(false);

    // Re-check with the SAME server hash — must stay dismissed (no nag).
    await fireCheck();
    await fireCheck();
    expect(result.current.hasNewVersion).toBe(false);
  });

  it("stays dismissed across a fresh mount (reload) for the same version", async () => {
    vi.mocked(globalThis.fetch).mockResolvedValue(manifestWith("def456"));

    // First mount: banner shows, user dismisses.
    const first = renderHook(() => useVersionCheck());
    await flush();
    await fireCheck();
    await waitFor(() => {
      expect(first.result.current.hasNewVersion).toBe(true);
    });
    act(() => {
      first.result.current.dismiss();
    });
    expect(first.result.current.hasNewVersion).toBe(false);

    // Simulate a page reload: unmount, then mount a brand-new hook instance
    // (the in-memory refs reset, only sessionStorage survives).
    first.unmount();

    const second = renderHook(() => useVersionCheck());
    // Same server hash still deployed — the dismissal must persist via
    // sessionStorage even after two fresh confirming checks.
    await flush();
    await fireCheck();
    expect(second.result.current.hasNewVersion).toBe(false);
  });

  it("re-shows the banner when a newer version is deployed after dismiss", async () => {
    const fetchFunction = vi.mocked(globalThis.fetch);
    fetchFunction.mockResolvedValue(manifestWith("def456"));

    const { result } = renderHook(() => useVersionCheck());

    await flush();
    await fireCheck();
    await waitFor(() => {
      expect(result.current.hasNewVersion).toBe(true);
    });

    act(() => {
      result.current.dismiss();
    });
    expect(result.current.hasNewVersion).toBe(false);

    // A newer deploy ships an even newer hash — banner must return after it is
    // confirmed on two consecutive checks.
    fetchFunction.mockResolvedValue(manifestWith("ghi789"));

    await fireCheck(); // pending = ghi789
    await fireCheck(); // confirmed
    await waitFor(() => {
      expect(result.current.hasNewVersion).toBe(true);
    });
  });

  it("fetches with cache-busting query param", async () => {
    const fetchFunction = vi.mocked(globalThis.fetch);
    fetchFunction.mockResolvedValue(manifestWith("abc123"));

    renderHook(() => useVersionCheck());

    await waitFor(() => {
      expect(fetchFunction.mock.calls.length).toBeGreaterThan(0);
    });

    const url = fetchFunction.mock.calls[0][0];
    expect(url).toMatch(/\/version\.json\?_v=\d+/);
  });

  it("sends no-cache header", async () => {
    const fetchFunction = vi.mocked(globalThis.fetch);
    fetchFunction.mockResolvedValue(manifestWith("abc123"));

    renderHook(() => useVersionCheck());

    await waitFor(() => {
      expect(fetchFunction.mock.calls.length).toBeGreaterThan(0);
    });

    const options = fetchFunction.mock.calls[0][1]!;
    expect(options.headers).toMatchObject({ "Cache-Control": "no-cache" });
    expect(options.cache).toBe("no-store");
  });

  it("does not throw on fetch failure", async () => {
    vi.mocked(globalThis.fetch).mockRejectedValue(new Error("network error"));

    const { result } = renderHook(() => useVersionCheck());

    await flush();
    await fireCheck();
    expect(result.current.hasNewVersion).toBe(false);
  });
});
