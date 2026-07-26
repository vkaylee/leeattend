import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, expect, it, vi } from "vitest";

import { FilterTrigger } from "../filter-trigger";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("FilterTrigger — pill-button variant", () => {
  it("renders with label and no badge when inactive", () => {
    render(
      <FilterTrigger
        data-testid="filter-trigger"
        label="Department"
        variant="pill-button"
      />,
    );

    const btn = screen.getByTestId("filter-trigger");
    expect(btn).toBeInTheDocument();
    expect(btn).toHaveTextContent("Department");
    // Inactive → no primary tint
    expect(btn.className).toContain("text-slate-500");
  });

  it("applies active styling when hasActive=true", () => {
    render(
      <FilterTrigger
        data-testid="filter-trigger"
        hasActive
        label="Department"
        variant="pill-button"
      />,
    );

    const btn = screen.getByTestId("filter-trigger");
    expect(btn.className).toContain("text-primary-600");
  });

  it("shows count badge when activeCount > 0", () => {
    render(
      <FilterTrigger
        activeCount={3}
        data-testid="filter-trigger"
        label="Department"
        variant="pill-button"
      />,
    );

    expect(screen.getByText("3")).toBeInTheDocument();
    const btn = screen.getByTestId("filter-trigger");
    // Badge presence also triggers active styling
    expect(btn.className).toContain("text-primary-600");
  });

  it("renders with chevron when chevron=true", () => {
    render(
      <FilterTrigger
        chevron
        data-testid="filter-trigger"
        label="Shift Type"
        variant="pill-button"
      />,
    );

    const btn = screen.getByTestId("filter-trigger");
    // ChevronDown icon should be in the DOM
    expect(btn.querySelector("svg")).toBeInTheDocument();
  });

  it("renders custom children instead of default icon+label", () => {
    render(
      <FilterTrigger
        data-testid="filter-trigger"
        label="Custom"
        variant="pill-button"
      >
        <span>Custom Content</span>
      </FilterTrigger>,
    );

    expect(screen.getByText("Custom Content")).toBeInTheDocument();
    expect(screen.queryByText("Custom")).not.toBeInTheDocument();
  });

  it("calls onClick when clicked", () => {
    const onClick = vi.fn();
    render(
      <FilterTrigger
        data-testid="filter-trigger"
        label="Test"
        onClick={onClick}
        variant="pill-button"
      />,
    );

    fireEvent.click(screen.getByTestId("filter-trigger"));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("supports disabled state", () => {
    render(
      <FilterTrigger
        data-testid="filter-trigger"
        disabled
        label="Test"
        variant="pill-button"
      />,
    );

    expect(screen.getByTestId("filter-trigger")).toBeDisabled();
  });
});

describe("FilterTrigger — icon-button variant", () => {
  it("renders as a compact icon button with aria-label", () => {
    render(
      <FilterTrigger
        data-testid="filter-trigger"
        label="Filter by department"
        variant="icon-button"
      />,
    );

    const btn = screen.getByTestId("filter-trigger");
    expect(btn).toHaveAttribute("aria-label", "Filter by department");
    expect(btn).toHaveAttribute("title", "Filter by department");
  });

  it("applies active tint when hasActive=true", () => {
    render(
      <FilterTrigger
        data-testid="filter-trigger"
        hasActive
        label="Filter"
        variant="icon-button"
      />,
    );

    const btn = screen.getByTestId("filter-trigger");
    expect(btn.className).toContain("text-primary-600");
  });

  it("shows positioned count badge", () => {
    render(
      <FilterTrigger
        activeCount={2}
        data-testid="filter-trigger"
        label="Filter"
        variant="icon-button"
      />,
    );

    expect(screen.getByText("2")).toBeInTheDocument();
  });

  it("hides badge when count is 0", () => {
    render(
      <FilterTrigger
        activeCount={0}
        data-testid="filter-trigger"
        label="Filter"
        variant="icon-button"
      />,
    );

    // No badge elements
    const btn = screen.getByTestId("filter-trigger");
    expect(btn.querySelectorAll("span").length).toBe(0);
  });

  it("supports aria-pressed for toggle behavior", () => {
    render(
      <FilterTrigger
        aria-pressed={true}
        data-testid="filter-trigger"
        label="Toggle"
        variant="icon-button"
      />,
    );

    expect(screen.getByTestId("filter-trigger")).toHaveAttribute("aria-pressed", "true");
  });

  it("supports aria-haspopup", () => {
    render(
      <FilterTrigger
        aria-haspopup="listbox"
        data-testid="filter-trigger"
        label="Select"
        variant="icon-button"
      />,
    );

    expect(screen.getByTestId("filter-trigger")).toHaveAttribute("aria-haspopup", "listbox");
  });
});
