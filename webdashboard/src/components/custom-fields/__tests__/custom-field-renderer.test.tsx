import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, it, expect } from "vitest";

import type { CustomFieldValueResponse } from "@/lib/api";

import { CustomFieldRenderer } from "../custom-field-renderer";

// ── Helpers ──────────────────────────────────────────────────────────────────

function buildValue(
  overrides: Partial<CustomFieldValueResponse> = {},
): CustomFieldValueResponse {
  return {
    field_type: "text",
    field_key: "test_field",
    field_label: "Test Field",
    value: "hello",
    ...overrides,
  } as CustomFieldValueResponse;
}

// ── Tests ────────────────────────────────────────────────────────────────────

describe("CustomFieldRenderer", () => {
  // ── Null / Empty / Undefined ──────────────────────────────────────────────

  describe("empty states", () => {
    it("renders dash when value is null", () => {
      render(<CustomFieldRenderer value={buildValue({ value: null })} />);
      expect(screen.getByText("—")).toBeInTheDocument();
    });

    it("renders dash when value is undefined", () => {
      render(<CustomFieldRenderer value={buildValue({ value: undefined })} />);
      expect(screen.getByText("—")).toBeInTheDocument();
    });

    it("renders dash when value is an empty string", () => {
      render(<CustomFieldRenderer value={buildValue({ value: "" })} />);
      expect(screen.getByText("—")).toBeInTheDocument();
    });
  });

  // ── Boolean ───────────────────────────────────────────────────────────────

  describe("boolean field type", () => {
    it("renders 'Yes' badge with check icon when value is true", () => {
      render(<CustomFieldRenderer value={buildValue({ field_type: "boolean", value: true })} />);
      expect(screen.getByText("Yes")).toBeInTheDocument();
    });

    it("renders 'No' badge with X icon when value is false", () => {
      render(<CustomFieldRenderer value={buildValue({ field_type: "boolean", value: false })} />);
      expect(screen.getByText("No")).toBeInTheDocument();
    });

    it('treats string "true" as boolean true', () => {
      render(<CustomFieldRenderer value={buildValue({ field_type: "boolean", value: "true" })} />);
      expect(screen.getByText("Yes")).toBeInTheDocument();
    });

    it('treats string "false" as boolean false', () => {
      render(<CustomFieldRenderer value={buildValue({ field_type: "boolean", value: "false" })} />);
      expect(screen.getByText("No")).toBeInTheDocument();
    });
  });

  // ── URL ───────────────────────────────────────────────────────────────────

  describe("url field type", () => {
    it("renders an anchor with the full URL when value starts with http", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "url", value: "https://example.com/profile" })}
        />,
      );
      const link = screen.getByRole("link");
      expect(link).toHaveAttribute("href", "https://example.com/profile");
      expect(link).toHaveTextContent("https://example.com/profile");
    });

    it("prefixes with https:// when value does not already have a protocol", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "url", value: "example.com" })}
        />,
      );
      const link = screen.getByRole("link");
      expect(link).toHaveAttribute("href", "https://example.com");
    });

    it("opens links in a new tab with noopener noreferrer", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "url", value: "https://safe.site" })}
        />,
      );
      const link = screen.getByRole("link");
      expect(link).toHaveAttribute("target", "_blank");
      expect(link).toHaveAttribute("rel", "noopener noreferrer");
    });
  });

  // ── Email ─────────────────────────────────────────────────────────────────

  describe("email field type", () => {
    it("renders a mailto link with the email address", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "email", value: "user@domain.com" })}
        />,
      );
      const link = screen.getByRole("link");
      expect(link).toHaveAttribute("href", "mailto:user@domain.com");
      expect(link).toHaveTextContent("user@domain.com");
    });
  });

  // ── Phone ─────────────────────────────────────────────────────────────────

  describe("phone field type", () => {
    it("renders a tel link with the phone number", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "phone", value: "+84123456789" })}
        />,
      );
      const link = screen.getByRole("link");
      expect(link).toHaveAttribute("href", "tel:+84123456789");
      expect(link).toHaveTextContent("+84123456789");
    });
  });

  // ── Number ────────────────────────────────────────────────────────────────

  describe("number field type", () => {
    it("renders a locale-formatted number", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "number", value: 1234567 })}
        />,
      );
      // Number.toLocaleString() varies by env; just verify it renders something numeric
      const el = screen.getByText((content) => /\d/.test(content));
      expect(el).toBeInTheDocument();
    });

    it("handles numeric strings by parsing them as numbers", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "number", value: "42" })}
        />,
      );
      const el = screen.getByText("42");
      expect(el).toBeInTheDocument();
    });
  });

  // ── Multiselect ───────────────────────────────────────────────────────────

  describe("multiselect field type", () => {
    it("renders chips for each item in an array value", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({
            field_type: "multiselect",
            value: ["Option A", "Option B", "Option C"],
          })}
        />,
      );
      expect(screen.getByText("Option A")).toBeInTheDocument();
      expect(screen.getByText("Option B")).toBeInTheDocument();
      expect(screen.getByText("Option C")).toBeInTheDocument();
    });

    it("parses JSON string array values into chips", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({
            field_type: "multiselect",
            value: '["Alpha","Beta"]',
          })}
        />,
      );
      expect(screen.getByText("Alpha")).toBeInTheDocument();
      expect(screen.getByText("Beta")).toBeInTheDocument();
    });

    it("renders dash when the parsed array is empty", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "multiselect", value: [] })}
        />,
      );
      expect(screen.getByText("—")).toBeInTheDocument();
    });

    it("falls back to plain text when JSON parsing fails", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({
            field_type: "multiselect",
            value: "{not-valid-json}",
          })}
        />,
      );
      // Should render the original string as fallback
      expect(screen.getByText("{not-valid-json}")).toBeInTheDocument();
    });

    it("renders raw value as string when JSON parse returns a non-array (isParseError=true)", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({
            field_type: "multiselect",
            value: '"not-an-array-json"',
          })}
        />,
      );
      // JSON.parse succeeds but returns a string, not an array → isParseError=true → renders as plain text
      expect(screen.getByText('"not-an-array-json"')).toBeInTheDocument();
    });
  });

  // ── Unknown / Fallback ────────────────────────────────────────────────────

  describe("unknown field type", () => {
    it("renders the string representation for an unrecognised field type", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "custom_unknown", value: "some value" })}
        />,
      );
      expect(screen.getByText("some value")).toBeInTheDocument();
    });

    it("renders numeric values as plain string when type is unrecognised", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "decimal", value: 3.14 })}
        />,
      );
      expect(screen.getByText("3.14")).toBeInTheDocument();
    });
  });

  // ── Text (default) ────────────────────────────────────────────────────────

  describe("text field type", () => {
    it("renders plain text for a standard text field", () => {
      render(
        <CustomFieldRenderer
          value={buildValue({ field_type: "text", value: "Plain text content" })}
        />,
      );
      expect(screen.getByText("Plain text content")).toBeInTheDocument();
    });
  });
});
