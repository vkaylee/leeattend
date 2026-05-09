import { render, screen, fireEvent } from "@testing-library/react";
import React, { createRef } from "react";

import { describe, it, expect, vi } from "vitest";

import FormField, { LABEL_BASE, INPUT_BASE } from "./form-field";

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
    render(<FormField id="custom-id" label="Field" name="field_name" />);
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
    render(<FormField label="Birthday" name="dob" type="date" />);
    const input = document.querySelector('input[type="date"]');
    expect(input).toBeInTheDocument();
  });

  it("renders email input", () => {
    render(<FormField label="Email" name="email" type="email" />);
    const input = document.querySelector('input[type="email"]');
    expect(input).toBeInTheDocument();
  });

  it("renders tel input", () => {
    render(<FormField label="Phone" name="phone" type="tel" />);
    const input = document.querySelector('input[type="tel"]');
    expect(input).toBeInTheDocument();
  });

  it("renders number input", () => {
    render(<FormField label="Age" name="age" type="number" />);
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
    render(<FormField label="Description" rows={3} type="textarea" />);
    const textarea = document.querySelector("textarea");
    expect(textarea).toHaveAttribute("rows", "3");
  });

  // ── Value & onChange ────────────────────────────────────────────

  it("supports controlled value and onChange", () => {
    const handler = vi.fn();
    render(
      <FormField label="Test" onChange={handler} value="hello" />,
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
    render(<FormField data-testid="my-field" label="Test" />);
    expect(screen.getByTestId("my-field")).toBeInTheDocument();
  });

  // ── Custom classNames ──────────────────────────────────────────

  it("merges wrapper className", () => {
    const { container } = render(
      <FormField className="extra-wrapper" label="Wrapper" />,
    );
    expect(container.firstElementChild!.className).toContain("extra-wrapper");
  });

  it("merges inputClassName on input", () => {
    render(<FormField inputClassName="font-mono" label="Font" />);
    expect(screen.getByRole("textbox").className).toContain("font-mono");
  });

  // ── Ref forwarding (React 19) ──────────────────────────────────

  it("forwards ref to input element", () => {
    const ref = createRef<HTMLInputElement>();
    render(<FormField label="Ref test" ref={ref} />);
    expect(ref.current).toBeInstanceOf(HTMLInputElement);
    expect(ref.current?.tagName).toBe("INPUT");
  });

  it("forwards ref to textarea element", () => {
    const ref = createRef<HTMLTextAreaElement>();
    render(<FormField label="Textarea ref" ref={ref as React.Ref<HTMLInputElement | HTMLTextAreaElement>} type="textarea" />);
    expect(ref.current).toBeInstanceOf(HTMLTextAreaElement);
    expect(ref.current?.tagName).toBe("TEXTAREA");
  });

  // ── onBlur ─────────────────────────────────────────────────────

  it("calls onBlur handler when input loses focus", () => {
    const onBlur = vi.fn();
    render(<FormField label="Blur test" onBlur={onBlur} />);
    const input = screen.getByRole("textbox");
    fireEvent.blur(input);
    expect(onBlur).toHaveBeenCalledTimes(1);
  });

  // ── Error display ──────────────────────────────────────────────

  it("displays error message when error prop is provided", () => {
    render(<FormField error="This field is required" label="Error field" />);
    expect(screen.getByText("This field is required")).toBeInTheDocument();
  });

  it("does not display error message when error prop is empty", () => {
    render(<FormField label="No error" />);
    expect(screen.queryByText(/required/i)).not.toBeInTheDocument();
  });

  // ── Disabled ───────────────────────────────────────────────────

  it("disables input when disabled prop is true", () => {
    render(<FormField disabled label="Disabled field" />);
    expect(screen.getByRole("textbox")).toBeDisabled();
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
