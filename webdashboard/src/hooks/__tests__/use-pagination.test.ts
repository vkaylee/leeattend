import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { usePagination } from "../use-pagination";

describe("usePagination", () => {
  it("defaults to page 1 and pageSize 20", () => {
    const { result } = renderHook(() => usePagination());
    expect(result.current.page).toBe(1);
    expect(result.current.pageSize).toBe(20);
  });

  it("accepts custom initial values", () => {
    const { result } = renderHook(() =>
      usePagination({ initialPage: 3, initialPageSize: 50 }),
    );
    expect(result.current.page).toBe(3);
    expect(result.current.pageSize).toBe(50);
  });

  it("setPage updates page", () => {
    const { result } = renderHook(() => usePagination());
    act(() => result.current.setPage(5));
    expect(result.current.page).toBe(5);
  });

  it("setPageSize updates pageSize but does NOT change page", () => {
    const { result } = renderHook(() =>
      usePagination({ initialPage: 2, initialPageSize: 20 }),
    );
    act(() => result.current.setPageSize(50));
    expect(result.current.pageSize).toBe(50);
    expect(result.current.page).toBe(2);
  });

  it("reset() returns to initial page", () => {
    const { result } = renderHook(() =>
      usePagination({ initialPage: 1, initialPageSize: 20 }),
    );
    act(() => result.current.setPage(7));
    expect(result.current.page).toBe(7);
    act(() => result.current.reset());
    expect(result.current.page).toBe(1);
  });

  it("reset() after initialPage change uses latest initialPage", () => {
    const { result, rerender } = renderHook(
      (props) => usePagination(props),
      { initialProps: { initialPage: 1, initialPageSize: 20 } },
    );
    act(() => result.current.setPage(10));
    rerender({ initialPage: 2, initialPageSize: 20 });
    act(() => result.current.reset());
    expect(result.current.page).toBe(2);
  });
});
