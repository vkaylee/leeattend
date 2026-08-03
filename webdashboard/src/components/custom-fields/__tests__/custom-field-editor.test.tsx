import { fireEvent, render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

import type { CustomFieldDefinition } from "@/lib/api";

import { CustomFieldEditor } from "../custom-field-editor";

// ── Mocks ────────────────────────────────────────────────────────────────────

vi.mock("../multiselect-field-editor", () => ({
  MultiselectFieldEditor: ({
    definition,
    value,
    onChange,
    options,
    error,
  }: {
    definition: CustomFieldDefinition;
    value: unknown;
    onChange: (v: unknown) => void;
    options: { value: string; label: string }[];
    error?: string;
  }) => (
    <div data-testid="multiselect-editor">
      <span data-testid="ms-label">{definition.field_label}</span>
      <span data-testid="ms-options-count">{options.length}</span>
      {!!error && <span data-testid="ms-error">{error}</span>}
      {/* eslint-disable-next-line react/forbid-elements -- test mock, native button is fine */}
      <button
        data-testid="ms-toggle"
        onClick={() => { onChange(Array.isArray(value) ? [] : ["opt-1"]); }}
        type="button"
      >
        Toggle
      </button>
    </div>
  ),
}));

// ── Helpers ──────────────────────────────────────────────────────────────────

function buildDef(
  overrides: Partial<CustomFieldDefinition> = {},
): CustomFieldDefinition {
  return {
    id: "cf-def-1",
    field_key: "test_key",
    field_label: "Test Field",
    field_type: "text",
    is_required: false,
    workspace_id: "ws-1",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  } as CustomFieldDefinition;
}

function renderEditor(
  overrides: Partial<CustomFieldDefinition> = {},
  props: { value?: unknown; onChange?: (v: unknown) => void; error?: string } = {},
) {
  const onChange = props.onChange ?? vi.fn();
  const utils = render(
    <CustomFieldEditor
      definition={buildDef(overrides)}
      error={props.error}
      onChange={onChange}
      value={props.value}
    />,
  );
  return { ...utils, onChange };
}

// ── Tests ────────────────────────────────────────────────────────────────────

describe("CustomFieldEditor", () => {
  // ── Boolean ───────────────────────────────────────────────────────────────

  describe("boolean field type", () => {
    it("renders a checkbox with the field label", () => {
      renderEditor({ field_type: "boolean", field_label: "Is Active" });
      expect(screen.getByRole("checkbox")).toBeInTheDocument();
      expect(screen.getByText("Is Active")).toBeInTheDocument();
    });

    it("calls onChange with true when checkbox is checked", () => {
      const onChange = vi.fn();
      renderEditor({ field_type: "boolean" }, { value: false, onChange });

      fireEvent.click(screen.getByRole("checkbox"));
      expect(onChange).toHaveBeenCalledWith(true);
    });

    it("renders checked when value is true", () => {
      renderEditor({ field_type: "boolean" }, { value: true });
      expect(screen.getByRole("checkbox")).toBeChecked();
    });

    it("renders unchecked when value is false", () => {
      renderEditor({ field_type: "boolean" }, { value: false });
      expect(screen.getByRole("checkbox")).not.toBeChecked();
    });

    it('treats string "true" value as checked', () => {
      renderEditor({ field_type: "boolean" }, { value: "true" });
      expect(screen.getByRole("checkbox")).toBeChecked();
    });

    it("displays error message when error prop is set", () => {
      renderEditor({ field_type: "boolean", field_label: "Flag" }, { error: "Required field" });
      expect(screen.getByText("Required field")).toBeInTheDocument();
    });

    it("shows required asterisk when is_required is true", () => {
      renderEditor({ field_type: "boolean", is_required: true });
      expect(screen.getByText("*")).toBeInTheDocument();
    });

    it("does not show required asterisk when is_required is false", () => {
      renderEditor({ field_type: "boolean", is_required: false });
      // The asterisk only appears when is_required is true
      const label = screen.getByText("Test Field");
      expect(label.querySelector(".text-rose-500")).toBeNull();
    });
  });

  // ── Select ────────────────────────────────────────────────────────────────

  describe("select field type", () => {
    it("renders a combobox (mocked as select) for select fields", () => {
      renderEditor({
        field_type: "select",
        field_label: "Department",
        options: [
          { value: "hr", label: "HR" },
          { value: "eng", label: "Engineering" },
        ],
      });
      // Combobox is mocked globally as a <select> in test/setup.tsx
      expect(screen.getByRole("combobox")).toBeInTheDocument();
    });

    it("displays the label for a select field", () => {
      renderEditor({ field_type: "select", field_label: "Choose Role" });
      expect(screen.getByText("Choose Role")).toBeInTheDocument();
    });

    it("displays error text for a select field with validation error", () => {
      renderEditor({ field_type: "select", field_label: "Role" }, { error: "Selection required" });
      expect(screen.getByText("Selection required")).toBeInTheDocument();
    });
  });

  // ── Multiselect ───────────────────────────────────────────────────────────

  describe("multiselect field type", () => {
    it("delegates to MultiselectFieldEditor with parsed options", () => {
      renderEditor({
        field_type: "multiselect",
        field_label: "Skills",
        field_key: "skills",
        options: [
          { value: "rust", label: "Rust" },
          { value: "ts", label: "TypeScript" },
        ],
      });

      expect(screen.getByTestId("multiselect-editor")).toBeInTheDocument();
      expect(screen.getByTestId("ms-label")).toHaveTextContent("Skills");
      expect(screen.getByTestId("ms-options-count")).toHaveTextContent("2");
    });

    it("passes error prop through to MultiselectFieldEditor", () => {
      renderEditor(
        { field_type: "multiselect", field_label: "Tags", options: [] },
        { error: "At least one tag required" },
      );

      expect(screen.getByTestId("ms-error")).toHaveTextContent("At least one tag required");
    });
  });

  // ── Text ──────────────────────────────────────────────────────────────────

  describe("text field type", () => {
    it("renders a text input with the field label", () => {
      renderEditor({ field_type: "text", field_label: "Full Name" });
      const input = screen.getByRole("textbox");
      expect(input).toBeInTheDocument();
      expect(screen.getByText("Full Name")).toBeInTheDocument();
    });

    it("displays the current value in the input", () => {
      renderEditor({ field_type: "text" }, { value: "John Doe" });
      expect(screen.getByDisplayValue("John Doe")).toBeInTheDocument();
    });

    it("calls onChange with the new value on input", () => {
      const onChange = vi.fn();
      renderEditor({ field_type: "text", field_key: "name" }, { value: "", onChange });

      fireEvent.change(screen.getByRole("textbox"), { target: { value: "Alice" } });
      expect(onChange).toHaveBeenCalledWith("Alice");
    });

    it("calls onChange with null when the input is cleared", () => {
      const onChange = vi.fn();
      renderEditor({ field_type: "text" }, { value: "old", onChange });

      fireEvent.change(screen.getByRole("textbox"), { target: { value: "" } });
      expect(onChange).toHaveBeenCalledWith(null);
    });
  });

  // ── Number ────────────────────────────────────────────────────────────────

  describe("number field type", () => {
    it("renders a number input", () => {
      renderEditor({ field_type: "number", field_label: "Age" });
      const input = screen.getByRole("spinbutton");
      expect(input).toBeInTheDocument();
      expect(input).toHaveAttribute("type", "number");
    });

    it("calls onChange with parsed Number on input", () => {
      const onChange = vi.fn();
      renderEditor({ field_type: "number" }, { value: "", onChange });

      fireEvent.change(screen.getByRole("spinbutton"), { target: { value: "42" } });
      expect(onChange).toHaveBeenCalledWith(42);
    });

    it("calls onChange with null when number input is cleared", () => {
      const onChange = vi.fn();
      renderEditor({ field_type: "number" }, { value: 5, onChange });

      fireEvent.change(screen.getByRole("spinbutton"), { target: { value: "" } });
      expect(onChange).toHaveBeenCalledWith(null);
    });
  });

  // ── Date ──────────────────────────────────────────────────────────────────

  describe("date field type", () => {
    it("renders a date input", () => {
      renderEditor({ field_type: "date", field_label: "Birth Date" });
      const input = screen.getByDisplayValue(""); // date input with no value
      expect(input).toHaveAttribute("type", "date");
    });

    it("passes value prop to the date input", () => {
      renderEditor({ field_type: "date" }, { value: "2026-08-01" });
      expect(screen.getByDisplayValue("2026-08-01")).toBeInTheDocument();
    });
  });

  // ── Email ─────────────────────────────────────────────────────────────────

  describe("email field type", () => {
    it("renders an email input", () => {
      renderEditor({ field_type: "email", field_label: "Work Email" });
      const input = screen.getByRole("textbox");
      expect(input).toHaveAttribute("type", "email");
    });

    it("has example@domain.com as the placeholder", () => {
      renderEditor({ field_type: "email" });
      const input = screen.getByPlaceholderText("example@domain.com");
      expect(input).toBeInTheDocument();
    });
  });

  // ── Phone ─────────────────────────────────────────────────────────────────

  describe("phone field type", () => {
    it("renders a tel input with +84... placeholder", () => {
      renderEditor({ field_type: "phone", field_label: "Mobile" });
      const input = screen.getByPlaceholderText("+84...");
      expect(input).toHaveAttribute("type", "tel");
    });
  });

  // ── default_value used as placeholder ─────────────────────────────────────

  describe("default_value placeholder", () => {
    it("uses default_value as the input placeholder when provided", () => {
      renderEditor({ field_type: "text", default_value: "Enter here" });
      expect(screen.getByPlaceholderText("Enter here")).toBeInTheDocument();
    });

    it("falls back to the type-specific placeholder when default_value is absent", () => {
      renderEditor({ field_type: "text" });
      expect(screen.getByPlaceholderText("Enter text")).toBeInTheDocument();
    });
  });

  // ── Error display ─────────────────────────────────────────────────────────

  describe("error display across types", () => {
    it("shows error for text field", () => {
      renderEditor({ field_type: "text", field_label: "Name" }, { error: "Cannot be empty" });
      expect(screen.getByText("Cannot be empty")).toBeInTheDocument();
    });

    it("shows error for number field", () => {
      renderEditor({ field_type: "number", field_label: "Count" }, { error: "Must be positive" });
      expect(screen.getByText("Must be positive")).toBeInTheDocument();
    });

    it("shows error for email field", () => {
      renderEditor({ field_type: "email" }, { error: "Invalid email" });
      expect(screen.getByText("Invalid email")).toBeInTheDocument();
    });
  });

  // ── Default / Unknown type ────────────────────────────────────────────────

  describe("unknown field type (fallback)", () => {
    it("falls back to a text input for unrecognised field types", () => {
      renderEditor({ field_type: "custom_type" as never, field_label: "Custom" });
      expect(screen.getByRole("textbox")).toBeInTheDocument();
    });
  });
});
