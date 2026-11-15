import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect } from "vitest";

import PageHeader from "./page-header";

describe("PageHeader Component", () => {
  // ── Title ───────────────────────────────────────────────────────

  it("renders the title as h1", () => {
    render(<PageHeader title="Employees" />);
    const heading = screen.getByRole("heading", { level: 1 });
    expect(heading).toHaveTextContent("Employees");
  });

  it("applies data-testid to the h1 element", () => {
    render(<PageHeader data-testid="employees-title" title="Employees" />);
    const heading = screen.getByTestId("employees-title");
    expect(heading.tagName).toBe("H1");
  });

  // ── Subtitle ────────────────────────────────────────────────────

  it("renders subtitle when provided", () => {
    render(
      <PageHeader subtitle="Overview of attendance data" title="Dashboard" />,
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
    const { container } = render(
      <PageHeader
        actions={<button data-testid="add-btn">Add</button>}
        title="Members"
      />,
    );
    expect(screen.getByTestId("add-btn")).toBeInTheDocument();
    expect(container.querySelector(".w-full.flex-wrap")).toHaveClass("xl:w-auto");
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
        actions={
          <>
            <button data-testid="export-btn">Export</button>
            <button data-testid="import-btn">Import</button>
          </>
        }
        title="Reports"
      />,
    );
    expect(screen.getByTestId("export-btn")).toBeInTheDocument();
    expect(screen.getByTestId("import-btn")).toBeInTheDocument();
  });

  // ── Custom className ────────────────────────────────────────────

  it("merges custom className", () => {
    const { container } = render(
      <PageHeader className="my-header" title="Styled" />,
    );
    expect(container.firstElementChild!.className).toContain("my-header");
  });

  // ── Full composition ────────────────────────────────────────────

  it("renders title, subtitle, and actions together", () => {
    render(
      <PageHeader
        actions={<button>Action</button>}
        data-testid="full-title"
        subtitle="With everything"
        title="Full Header"
      />,
    );

    expect(screen.getByTestId("full-title")).toHaveTextContent("Full Header");
    expect(screen.getByText("With everything")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Action" }),
    ).toBeInTheDocument();
  });

  it("renders gradient title when gradientTitle is true", () => {
    render(<PageHeader gradientTitle title="Gradient Title" />);
    const heading = screen.getByRole("heading", { level: 1 });
    expect(heading).toHaveTextContent("Gradient Title");
    const gradientSpan = heading.querySelector("span");
    expect(gradientSpan?.className).toContain("bg-clip-text");
  });
});
