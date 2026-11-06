import { renderHook, act } from "@testing-library/react";
import { describe, it, expect } from "vitest";

import { useDsmLauncher, openDsmLauncher } from "../use-dsm-launcher";

describe("useDsmLauncher", () => {
  it("initializes with isOpen = false", () => {
    const { result } = renderHook(() => useDsmLauncher());
    expect(result.current.isOpen).toBe(false);
  });

  it("opens, closes, and toggles state", () => {
    const { result } = renderHook(() => useDsmLauncher());

    act(() => {
      result.current.open();
    });
    expect(result.current.isOpen).toBe(true);

    act(() => {
      result.current.close();
    });
    expect(result.current.isOpen).toBe(false);

    act(() => {
      result.current.toggle();
    });
    expect(result.current.isOpen).toBe(true);
  });

  it("toggles on Cmd+K keydown", () => {
    const { result } = renderHook(() => useDsmLauncher());
    expect(result.current.isOpen).toBe(false);

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "k",
        metaKey: true,
        bubbles: true,
        cancelable: true,
      });
      window.dispatchEvent(event);
    });

    expect(result.current.isOpen).toBe(true);

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "k",
        metaKey: true,
        bubbles: true,
        cancelable: true,
      });
      window.dispatchEvent(event);
    });

    expect(result.current.isOpen).toBe(false);
  });

  it("toggles on Ctrl+K keydown", () => {
    const { result } = renderHook(() => useDsmLauncher());

    act(() => {
      const event = new KeyboardEvent("keydown", {
        key: "k",
        ctrlKey: true,
        bubbles: true,
        cancelable: true,
      });
      window.dispatchEvent(event);
    });

    expect(result.current.isOpen).toBe(true);
  });

  it("opens on openDsmLauncher() custom event", () => {
    const { result } = renderHook(() => useDsmLauncher());
    expect(result.current.isOpen).toBe(false);

    act(() => {
      openDsmLauncher();
    });

    expect(result.current.isOpen).toBe(true);
  });
});
