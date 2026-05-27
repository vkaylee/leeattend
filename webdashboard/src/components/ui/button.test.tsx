import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

import Button from "./button";

describe("Button Component", () => {
  // ── Rendering ───────────────────────────────────────────────────

  it("renders children text", () => {
    render(<Button variant="ghost">Click Me</Button>);
    expect(screen.getByRole("button", { name: /click me/i })).toBeInTheDocument();
  });

  it("defaults to type='button'", () => {
    render(<Button variant="primary">Save</Button>);
    expect(screen.getByRole("button")).toHaveAttribute("type", "button");
  });

  it("supports type='submit'", () => {
    render(<Button type="submit" variant="primary">Submit</Button>);
    expect(screen.getByRole("button")).toHaveAttribute("type", "submit");
  });

  // ── Variants ────────────────────────────────────────────────────

  it("applies primary variant classes by default", () => {
    render(<Button>Primary</Button>);
    const button = screen.getByRole("button");
    expect(button.className).toContain("bg-primary-600");
  });

  it("applies outline variant classes", () => {
    render(<Button variant="outline">Outline</Button>);
    const button = screen.getByRole("button");
    expect(button.className).toContain("border");
    expect(button.className).not.toContain("bg-primary-600");
  });

  it("applies secondary variant classes", () => {
    render(<Button variant="secondary">Secondary</Button>);
    const button = screen.getByRole("button");
    expect(button.className).toContain("bg-slate-900");
  });

  it("applies ghost variant classes", () => {
    render(<Button variant="ghost">Ghost</Button>);
    const button = screen.getByRole("button");
    expect(button.className).not.toContain("bg-primary-600");
    expect(button.className).not.toContain("shadow-xl");
  });

  it("applies destructive variant classes", () => {
    render(<Button variant="destructive">Delete</Button>);
    const button = screen.getByRole("button");
    expect(button.className).toContain("bg-rose-600");
  });

  // ── Sizes ───────────────────────────────────────────────────────

  it("applies md size by default", () => {
    render(<Button variant="ghost">Medium</Button>);
    expect(screen.getByRole("button").className).toContain("px-8");
  });

  it("applies sm size", () => {
    render(<Button size="sm" variant="ghost">Small</Button>);
    expect(screen.getByRole("button").className).toContain("px-4");
  });

  it("applies lg size", () => {
    render(<Button size="lg" variant="ghost">Large</Button>);
    expect(screen.getByRole("button").className).toContain("px-10");
  });

  // ── Icon ────────────────────────────────────────────────────────

  it("renders icon alongside children", () => {
    render(
      <Button icon={<span data-testid="icon">★</span>} variant="ghost">With Icon</Button>,
    );
    expect(screen.getByTestId("icon")).toBeInTheDocument();
    expect(screen.getByText("With Icon")).toBeInTheDocument();
  });

  // ── Loading state ───────────────────────────────────────────────

  it("disables button and shows spinner when loading", () => {
    render(
      <Button icon={<span data-testid="icon">★</span>} loading variant="ghost">
        Loading
      </Button>,
    );
    const button = screen.getByRole("button");
    expect(button).toBeDisabled();
    // The icon should be replaced by Loader2 spinner
    expect(screen.queryByTestId("icon")).not.toBeInTheDocument();
  });

  // ── Disabled state ──────────────────────────────────────────────

  it("is disabled when disabled prop is true", () => {
    render(<Button disabled variant="ghost">Disabled</Button>);
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("is disabled when loading even without explicit disabled", () => {
    render(<Button loading variant="ghost">Saving</Button>);
    expect(screen.getByRole("button")).toBeDisabled();
  });

  // ── Click handler ───────────────────────────────────────────────

  it("calls onClick handler when clicked", () => {
    const handler = vi.fn();
    render(<Button onClick={handler} variant="ghost">Click</Button>);
    fireEvent.click(screen.getByRole("button"));
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it("does not call onClick when disabled", () => {
    const handler = vi.fn();
    render(
      <Button disabled onClick={handler} variant="ghost">
        Click
      </Button>,
    );
    fireEvent.click(screen.getByRole("button"));
    expect(handler).not.toHaveBeenCalled();
  });

  // ── Custom className ────────────────────────────────────────────

  it("merges custom className", () => {
    render(<Button className="my-custom" variant="ghost">Custom</Button>);
    expect(screen.getByRole("button").className).toContain("my-custom");
  });

  // ── Extra HTML attributes via rest spread ───────────────────────

  it("passes through data-testid and aria attributes", () => {
    render(
      <Button aria-label="Save changes" data-testid="my-btn" variant="primary">
        Save
      </Button>,
    );
    expect(screen.getByTestId("my-btn")).toBeInTheDocument();
    expect(screen.getByRole("button")).toHaveAttribute(
      "aria-label",
      "Save changes",
    );
  });
});
