import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, expect, it, vi } from "vitest";

import { FilterChips } from "../filter-chips";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
  }),
}));

const MOCK_CHIPS = [
  { key: "dept-1", label: "Engineering", category: "department" },
  { key: "dept-2", label: "Design", category: "department" },
  { key: "status-terminated", label: "Đã nghỉ việc", category: "status" },
];

describe("FilterChips", () => {
  it("renders nothing when chips array is empty", () => {
    const { container } = render(
      <FilterChips chips={[]} onClearAll={vi.fn()} onRemove={vi.fn()} />,
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders all chips with labels", () => {
    render(
      <FilterChips
        chips={MOCK_CHIPS}
        data-testid="filter-chips"
        onClearAll={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    expect(screen.getByText("Engineering")).toBeInTheDocument();
    expect(screen.getByText("Design")).toBeInTheDocument();
    expect(screen.getByText("Đã nghỉ việc")).toBeInTheDocument();
  });

  it("renders label when provided", () => {
    render(
      <FilterChips
        chips={[MOCK_CHIPS[0]]}
        data-testid="filter-chips"
        label="Đang lọc"
        onClearAll={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    expect(screen.getByText("Đang lọc")).toBeInTheDocument();
  });

  it("calls onRemove when a chip X is clicked", () => {
    const onRemove = vi.fn();
    render(
      <FilterChips
        chips={MOCK_CHIPS}
        data-testid="filter-chips"
        onClearAll={vi.fn()}
        onRemove={onRemove}
      />,
    );

    fireEvent.click(screen.getByTestId("filter-chips-chip-dept-1"));
    expect(onRemove).toHaveBeenCalledWith("dept-1");
  });

  it("calls onClearAll when Clear all is clicked", () => {
    const onClearAll = vi.fn();
    render(
      <FilterChips
        chips={MOCK_CHIPS}
        data-testid="filter-chips"
        onClearAll={onClearAll}
        onRemove={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByTestId("filter-chips-clear-all"));
    expect(onClearAll).toHaveBeenCalledTimes(1);
  });

  it("hides Clear all button when showClearAll=false", () => {
    render(
      <FilterChips
        chips={MOCK_CHIPS}
        data-testid="filter-chips"
        onClearAll={vi.fn()}
        onRemove={vi.fn()}
        showClearAll={false}
      />,
    );

    expect(screen.queryByTestId("filter-chips-clear-all")).toBeNull();
  });

  it("uses custom clearAllLabel", () => {
    render(
      <FilterChips
        chips={MOCK_CHIPS}
        clearAllLabel="Reset filters"
        data-testid="filter-chips"
        onClearAll={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    expect(screen.getByText("Reset filters")).toBeInTheDocument();
  });

  it("applies department category style", () => {
    render(
      <FilterChips
        chips={[MOCK_CHIPS[0]]}
        data-testid="filter-chips"
        onClearAll={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    const chip = screen.getByTestId("filter-chips-chip-dept-1");
    expect(chip.className).toContain("text-primary-600");
  });

  it("applies status category style", () => {
    render(
      <FilterChips
        chips={[MOCK_CHIPS[2]]}
        data-testid="filter-chips"
        onClearAll={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    const chip = screen.getByTestId("filter-chips-chip-status-terminated");
    expect(chip.className).toContain("text-emerald-600");
  });

  it("applies default style for unknown category", () => {
    render(
      <FilterChips
        chips={[{ key: "unknown", label: "Mystery", category: "unknown" }]}
        data-testid="filter-chips"
        onClearAll={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    const chip = screen.getByTestId("filter-chips-chip-unknown");
    expect(chip.className).toContain("text-slate-600");
  });

  it("allows custom category style override", () => {
    render(
      <FilterChips
        categoryStyles={{
          department: { baseClass: "bg-red-50 text-red-600" },
        }}
        chips={[MOCK_CHIPS[0]]}
        data-testid="filter-chips"
        onClearAll={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    const chip = screen.getByTestId("filter-chips-chip-dept-1");
    expect(chip.className).toContain("text-red-600");
  });
});
