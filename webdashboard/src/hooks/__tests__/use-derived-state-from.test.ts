import { act, renderHook } from "@testing-library/react";

import { describe, expect, it } from "vitest";

import { useDerivedStateFrom, useDerivedStateFromWithReset } from "../use-derived-state-from";

describe("useDerivedStateFrom", () => {
  it("preserves a local update while the external value is unchanged", () => {
    const { result } = renderHook(() => useDerivedStateFrom("external"));

    act(() => {
      result.current[1]("local");
    });

    expect(result.current[0]).toBe("local");
  });

  it("resynchronizes when the external value changes", () => {
    const { result, rerender } = renderHook(
      ({ externalValue }) => useDerivedStateFrom(externalValue),
      { initialProps: { externalValue: "first" } },
    );

    act(() => {
      result.current[1]("local");
    });
    expect(result.current[0]).toBe("local");

    rerender({ externalValue: "second" });

    expect(result.current[0]).toBe("second");
  });

  it("uses the current external value as the base for a first functional update", () => {
    const externalValue = { name: "external" };
    const { result } = renderHook(() => useDerivedStateFrom(externalValue));

    act(() => {
      result.current[1]((previous) => ({ ...previous, name: "local" }));
    });

    expect(result.current[0]).toEqual({ name: "local" });
  });
});

describe("useDerivedStateFromWithReset", () => {
  it("preserves a local update and reset restores the external value", () => {
    const { result } = renderHook(() => useDerivedStateFromWithReset("external"));

    act(() => {
      result.current[1]("local");
    });
    expect(result.current[0]).toBe("local");

    act(() => {
      result.current[2]();
    });

    expect(result.current[0]).toBe("external");
  });
});
