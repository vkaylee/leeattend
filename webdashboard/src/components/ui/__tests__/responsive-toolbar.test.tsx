import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import React from "react";

import { describe, expect, it, vi } from "vitest";

import ResponsiveToolbar from "../responsive-toolbar";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
  }),
}));

vi.mock("@/hooks/use-focus-trap", () => ({
  useFocusTrap: () => ({ current: null }),
}));

describe("ResponsiveToolbar", () => {
  it("renders search, filters, actions, and active filter badge", () => {
    render(
      <ResponsiveToolbar
        actions={<button type="button">Export</button>}
        filterCount={2}
        searchSlot={<input aria-label="Search employees" />}
      >
        <button type="button">Department</button>
      </ResponsiveToolbar>,
    );

    expect(screen.getByLabelText("Search employees")).toBeInTheDocument();
    expect(screen.getByText("Department")).toBeInTheDocument();
    expect(screen.getByText("Export")).toBeInTheDocument();
    expect(screen.getAllByText("2").length).toBeGreaterThanOrEqual(1);
  });

  it("opens and applies the mobile filter sheet", async () => {
    render(
      <ResponsiveToolbar filterCount={1}>
        <button type="button">Status filter</button>
      </ResponsiveToolbar>,
    );

    fireEvent.click(screen.getByTestId("filter-trigger"));

    await waitFor(() => {
      expect(screen.getByTestId("filter-sheet")).toBeInTheDocument();
    });
    expect(screen.getByRole("dialog")).toHaveAttribute("aria-modal", "true");

    fireEvent.click(screen.getByTestId("filter-sheet-apply"));
    expect(screen.getByTestId("filter-sheet")).toHaveClass("translate-y-full");
  });

  it("closes the filter sheet on Escape", async () => {
    render(
      <ResponsiveToolbar>
        <button type="button">Filter content</button>
      </ResponsiveToolbar>,
    );

    fireEvent.click(screen.getByTestId("filter-trigger"));
    await waitFor(() => { expect(screen.getByTestId("filter-sheet")).toBeInTheDocument(); });

    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.getByTestId("filter-sheet")).toHaveClass("translate-y-full");
  });
});
