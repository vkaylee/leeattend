import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import FormField, { LABEL_BASE, INPUT_BASE } from "./FormField";
import React from "react";

describe("FormField Component", () => {
  // ── Label rendering ─────────────────────────────────────────────

  it("renders the label text", () => {
    render(<FormField label="Email Address" />);
    expect(screen.getByText("Email Address")).toBeInTheDocument();
  });

  it("associates label with input via htmlFor", () => {
    render(<FormField label="Username" name="username" />);
    const label = screen.getByText("Username");
    const input = screen.getByRole("textbox");
    expect(label).toHaveAttribute("for", "username");
    expect(input).toHaveAttribute("id", "username");
  });

  it("uses id prop over name for association", () => {
    render(<FormField label="Field" name="field_name" id="custom-id" />);
    const label = screen.getByText("Field");
    expect(label).toHaveAttribute("for", "custom-id");
  });

  // ── Input types ─────────────────────────────────────────────────

  it("renders text input by default", () => {
    render(<FormField label="Name" />);
    const input = screen.getByRole("textbox");
    expect(input.tagName).toBe("INPUT");
    expect(input).toHaveAttribute("type", "text");
  });

  it("renders date input", () => {
    render(<FormField label="Birthday" type="date" name="dob" />);
    const input = document.querySelector('input[type="date"]');
    expect(input).toBeInTheDocument();
  });

  it("renders email input", () => {
    render(<FormField label="Email" type="email" name="email" />);
    const input = document.querySelector('input[type="email"]');
    expect(input).toBeInTheDocument();
  });

  it("renders tel input", () => {
    render(<FormField label="Phone" type="tel" name="phone" />);
    const input = document.querySelector('input[type="tel"]');
    expect(input).toBeInTheDocument();
  });

  it("renders number input", () => {
    render(<FormField label="Age" type="number" name="age" />);
    const input = document.querySelector('input[type="number"]');
    expect(input).toBeInTheDocument();
  });

  it("renders textarea when type='textarea'", () => {
    render(<FormField label="Notes" type="textarea" />);
    const textarea = document.querySelector("textarea");
    expect(textarea).toBeInTheDocument();
    expect(textarea).toHaveAttribute("rows", "5"); // default rows
  });

  it("renders textarea with custom rows", () => {
    render(<FormField label="Description" type="textarea" rows={3} />);
    const textarea = document.querySelector("textarea");
    expect(textarea).toHaveAttribute("rows", "3");
  });

  // ── Value & onChange ────────────────────────────────────────────

  it("supports controlled value and onChange", () => {
    const handler = vi.fn();
    render(
      <FormField label="Test" value="hello" onChange={handler} />,
    );
    const input = screen.getByRole("textbox");
    expect(input).toHaveValue("hello");

    fireEvent.change(input, { target: { value: "world" } });
    expect(handler).toHaveBeenCalledTimes(1);
  });

  it("supports placeholder", () => {
    render(<FormField label="Search" placeholder="Type here..." />);
    expect(screen.getByPlaceholderText("Type here...")).toBeInTheDocument();
  });

  // ── Required ────────────────────────────────────────────────────

  it("passes required attribute to input", () => {
    render(<FormField label="Required" required />);
    expect(screen.getByRole("textbox")).toBeRequired();
  });

  // ── data-testid ─────────────────────────────────────────────────

  it("passes data-testid to input", () => {
    render(<FormField label="Test" data-testid="my-field" />);
    expect(screen.getByTestId("my-field")).toBeInTheDocument();
  });

  // ── Custom classNames ──────────────────────────────────────────

  it("merges wrapper className", () => {
    const { container } = render(
      <FormField label="Wrapper" className="extra-wrapper" />,
    );
    expect(container.firstElementChild!.className).toContain("extra-wrapper");
  });

  it("merges inputClassName on input", () => {
    render(<FormField label="Font" inputClassName="font-mono" />);
    expect(screen.getByRole("textbox").className).toContain("font-mono");
  });
});

// ── Exported constants ────────────────────────────────────────────

describe("FormField exports", () => {
  it("exports LABEL_BASE as a non-empty string", () => {
    expect(typeof LABEL_BASE).toBe("string");
    expect(LABEL_BASE.length).toBeGreaterThan(0);
    expect(LABEL_BASE).toContain("uppercase");
    expect(LABEL_BASE).toContain("tracking-widest");
  });

  it("exports INPUT_BASE as a non-empty string", () => {
    expect(typeof INPUT_BASE).toBe("string");
    expect(INPUT_BASE.length).toBeGreaterThan(0);
    expect(INPUT_BASE).toContain("rounded-2xl");
  });
});
