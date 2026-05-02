import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect } from "vitest";

import StatusBadge from "./StatusBadge";

describe("StatusBadge Component", () => {
  // ── Rendering ───────────────────────────────────────────────────

  it("renders children text", () => {
    render(<StatusBadge>ACTIVE</StatusBadge>);
    expect(screen.getByText("ACTIVE")).toBeInTheDocument();
  });

  it("renders as a <span> element", () => {
    render(<StatusBadge>Test</StatusBadge>);
    const element = screen.getByText("Test");
    expect(element.tagName).toBe("SPAN");
  });

  // ── Variants ────────────────────────────────────────────────────

  it("defaults to neutral variant", () => {
    render(<StatusBadge>Neutral</StatusBadge>);
    const element = screen.getByText("Neutral");
    expect(element.className).toContain("bg-slate-50");
    expect(element.className).toContain("opacity-60");
  });

  it("applies success variant", () => {
    render(<StatusBadge variant="success">Active</StatusBadge>);
    expect(screen.getByText("Active").className).toContain("bg-emerald-50");
  });

  it("applies warning variant", () => {
    render(<StatusBadge variant="warning">Pending</StatusBadge>);
    expect(screen.getByText("Pending").className).toContain("bg-amber-50");
  });

  it("applies danger variant", () => {
    render(<StatusBadge variant="danger">Inactive</StatusBadge>);
    expect(screen.getByText("Inactive").className).toContain("bg-rose-50");
  });

  it("applies info variant", () => {
    render(<StatusBadge variant="info">Info</StatusBadge>);
    expect(screen.getByText("Info").className).toContain("bg-blue-50");
  });

  it("applies primary variant", () => {
    render(<StatusBadge variant="primary">Primary</StatusBadge>);
    expect(screen.getByText("Primary").className).toContain("bg-primary-50");
  });

  it("applies indigo variant", () => {
    render(<StatusBadge variant="indigo">Indigo</StatusBadge>);
    expect(screen.getByText("Indigo").className).toContain("bg-indigo-50");
  });

  it("applies emerald variant", () => {
    render(<StatusBadge variant="emerald">Emerald</StatusBadge>);
    expect(screen.getByText("Emerald").className).toContain("bg-emerald-50");
  });

  it("applies orange variant", () => {
    render(<StatusBadge variant="orange">Orange</StatusBadge>);
    expect(screen.getByText("Orange").className).toContain("bg-orange-50");
  });

  // ── Icon ────────────────────────────────────────────────────────

  it("renders icon alongside children", () => {
    render(
      <StatusBadge icon={<span data-testid="badge-icon">●</span>}>
        Online
      </StatusBadge>,
    );
    expect(screen.getByTestId("badge-icon")).toBeInTheDocument();
    expect(screen.getByText("Online")).toBeInTheDocument();
  });

  it("renders without icon by default", () => {
    const { container } = render(<StatusBadge>Solo</StatusBadge>);
    // Only the text child, no extra elements
    const span = container.querySelector("span")!;
    expect(span.children).toHaveLength(0); // text node, no child elements
  });

  // ── Pulse animation ─────────────────────────────────────────────

  it("does not apply animate-pulse by default", () => {
    render(<StatusBadge>Static</StatusBadge>);
    expect(screen.getByText("Static").className).not.toContain("animate-pulse");
  });

  it("applies animate-pulse when pulse=true", () => {
    render(<StatusBadge pulse>Pulsing</StatusBadge>);
    expect(screen.getByText("Pulsing").className).toContain("animate-pulse");
  });

  // ── Custom className ────────────────────────────────────────────

  it("merges custom className", () => {
    render(<StatusBadge className="my-badge">Custom</StatusBadge>);
    expect(screen.getByText("Custom").className).toContain("my-badge");
  });

  // ── Base styles always present ──────────────────────────────────

  it("always has base badge styles", () => {
    render(<StatusBadge>Base</StatusBadge>);
    const element = screen.getByText("Base");
    expect(element.className).toContain("rounded-full");
    expect(element.className).toContain("font-bold");
    expect(element.className).toContain("uppercase");
    expect(element.className).toContain("border");
  });
});
