import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import PageHeader from "./PageHeader";
import React from "react";

describe("PageHeader Component", () => {
  // ── Title ───────────────────────────────────────────────────────

  it("renders the title as h1", () => {
    render(<PageHeader title="Employees" />);
    const heading = screen.getByRole("heading", { level: 1 });
    expect(heading).toHaveTextContent("Employees");
  });

  it("applies data-testid to the h1 element", () => {
    render(<PageHeader title="Devices" data-testid="devices-title" />);
    const heading = screen.getByTestId("devices-title");
    expect(heading.tagName).toBe("H1");
  });

  // ── Subtitle ────────────────────────────────────────────────────

  it("renders subtitle when provided", () => {
    render(
      <PageHeader title="Dashboard" subtitle="Overview of attendance data" />,
    );
    expect(
      screen.getByText("Overview of attendance data"),
    ).toBeInTheDocument();
  });

  it("does not render subtitle when not provided", () => {
    const { container } = render(<PageHeader title="Settings" />);
    const paragraphs = container.querySelectorAll("p");
    expect(paragraphs).toHaveLength(0);
  });

  // ── Actions ─────────────────────────────────────────────────────

  it("renders actions when provided", () => {
    render(
      <PageHeader
        title="Members"
        actions={<button data-testid="add-btn">Add</button>}
      />,
    );
    expect(screen.getByTestId("add-btn")).toBeInTheDocument();
  });

  it("does not render actions container when not provided", () => {
    const { container } = render(<PageHeader title="Solo" />);
    // Only the title div should exist
    const wrapperChildren = container.firstElementChild!.children;
    expect(wrapperChildren).toHaveLength(1); // just the title div
  });

  it("renders multiple action elements", () => {
    render(
      <PageHeader
        title="Reports"
        actions={
          <>
            <button data-testid="export-btn">Export</button>
            <button data-testid="import-btn">Import</button>
          </>
        }
      />,
    );
    expect(screen.getByTestId("export-btn")).toBeInTheDocument();
    expect(screen.getByTestId("import-btn")).toBeInTheDocument();
  });

  // ── Custom className ────────────────────────────────────────────

  it("merges custom className", () => {
    const { container } = render(
      <PageHeader title="Styled" className="my-header" />,
    );
    expect(container.firstElementChild!.className).toContain("my-header");
  });

  // ── Full composition ────────────────────────────────────────────

  it("renders title, subtitle, and actions together", () => {
    render(
      <PageHeader
        title="Full Header"
        subtitle="With everything"
        data-testid="full-title"
        actions={<button>Action</button>}
      />,
    );

    expect(screen.getByTestId("full-title")).toHaveTextContent("Full Header");
    expect(screen.getByText("With everything")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Action" }),
    ).toBeInTheDocument();
  });
});
