import { renderHook, act, waitFor } from "@testing-library/react";
import { describe, it, expect } from "vitest";

import { useConnectivityState } from "../use-connectivity-logic";

/**
 * Regression coverage for the device-timezone default.
 *
 * The agent resolves the clock's naive wall-clock reading against the device's
 * timezone. A hardcoded "UTC" default meant every device created from an
 * Asia/Ho_Chi_Minh workspace stored punches 7 hours off — a wrong *instant*, at
 * ingestion, upstream of all reporting.
 */
describe("useConnectivityState — device timezone default", () => {
  it("seeds a new device with the workspace timezone", () => {
    const { result } = renderHook(() => useConnectivityState("Asia/Ho_Chi_Minh"));

    expect(result.current.newDevice.timezone).toBe("Asia/Ho_Chi_Minh");
  });

  it("adopts the workspace timezone once the workspace query resolves", async () => {
    // The workspace is undefined on first render, so the hook starts at UTC and
    // must pick the real zone up when it arrives.
    const { result, rerender } = renderHook(
      ({ tz }: { tz?: string }) => useConnectivityState(tz),
      { initialProps: { tz: undefined as string | undefined } },
    );

    expect(result.current.newDevice.timezone).toBe("UTC");

    rerender({ tz: "Asia/Bangkok" });

    await waitFor(() => {
      expect(result.current.newDevice.timezone).toBe("Asia/Bangkok");
    });
  });

  it("does not overwrite a timezone the user explicitly chose", async () => {
    const { result, rerender } = renderHook(
      ({ tz }: { tz?: string }) => useConnectivityState(tz),
      { initialProps: { tz: undefined as string | undefined } },
    );

    act(() => {
      result.current.setNewDevice({ ...result.current.newDevice, timezone: "Europe/London" });
    });

    rerender({ tz: "Asia/Ho_Chi_Minh" });

    await waitFor(() => {
      expect(result.current.newDevice.timezone).toBe("Europe/London");
    });
  });

  it("falls back to UTC when the workspace timezone is unavailable", () => {
    const { result } = renderHook(() => useConnectivityState(undefined));

    expect(result.current.newDevice.timezone).toBe("UTC");
  });

  it("updates an inherited timezone when the active workspace changes", async () => {
    const { result, rerender } = renderHook(
      ({ tz }: { tz?: string }) => useConnectivityState(tz),
      { initialProps: { tz: "Asia/Ho_Chi_Minh" as string | undefined } },
    );

    rerender({ tz: "Asia/Bangkok" });

    await waitFor(() => {
      expect(result.current.newDevice.timezone).toBe("Asia/Bangkok");
    });
  });
});
