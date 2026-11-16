import { fireEvent, render, renderHook, screen, act } from "@testing-library/react";
import React from "react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import { TableDensityToggle } from "../table-density-toggle";
import { useTableDensity } from "../use-table-density";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
  }),
}));

describe("TableDensityToggle", () => {
  it("renders all 3 density options", () => {
    const onDensityChange = vi.fn();
    render(
      <TableDensityToggle density="normal" onDensityChange={onDensityChange} />
    );

    expect(screen.getByTestId("table-density-toggle")).toBeInTheDocument();
    expect(screen.getByTestId("density-btn-compact")).toBeInTheDocument();
    expect(screen.getByTestId("density-btn-normal")).toBeInTheDocument();
    expect(screen.getByTestId("density-btn-comfortable")).toBeInTheDocument();
  });

  it("calls onDensityChange with clicked density", () => {
    const onDensityChange = vi.fn();
    render(
      <TableDensityToggle density="normal" onDensityChange={onDensityChange} />
    );

    fireEvent.click(screen.getByTestId("density-btn-compact"));
    expect(onDensityChange).toHaveBeenCalledWith("compact");

    fireEvent.click(screen.getByTestId("density-btn-comfortable"));
    expect(onDensityChange).toHaveBeenCalledWith("comfortable");
  });

  it("indicates aria-pressed state on active density option", () => {
    const onDensityChange = vi.fn();
    render(
      <TableDensityToggle density="compact" onDensityChange={onDensityChange} />
    );

    expect(screen.getByTestId("density-btn-compact")).toHaveAttribute(
      "aria-pressed",
      "true"
    );
    expect(screen.getByTestId("density-btn-normal")).toHaveAttribute(
      "aria-pressed",
      "false"
    );
    expect(screen.getByTestId("density-btn-comfortable")).toHaveAttribute(
      "aria-pressed",
      "false"
    );
  });
});

describe("useTableDensity", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("defaults to normal when nothing is stored", () => {
    const { result } = renderHook(() => useTableDensity("test_density"));
    expect(result.current.density).toBe("normal");
  });

  it("initializes from localStorage if valid", () => {
    localStorage.setItem("test_density", "compact");
    const { result } = renderHook(() => useTableDensity("test_density"));
    expect(result.current.density).toBe("compact");
  });

  it("updates state and persists to localStorage on setDensity", () => {
    const { result } = renderHook(() => useTableDensity("test_density"));

    act(() => {
      result.current.setDensity("comfortable");
    });

    expect(result.current.density).toBe("comfortable");
    expect(localStorage.getItem("test_density")).toBe("comfortable");
  });
});
