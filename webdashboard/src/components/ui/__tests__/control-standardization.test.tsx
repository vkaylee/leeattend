import React from "react";

import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import Button from "../button";
import { Combobox } from "../combobox";
import { CONTROL_SIZE_STYLES, type ControlSize } from "../control-styles";
import FormField from "../form-field";
import { SearchInput } from "../search-input";

vi.unmock("@/components/ui/combobox");

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));

describe("Shared control standards", () => {
  it.each<ControlSize>(["sm", "md", "lg"])("shares %s height and typography across controls", (size) => {
    render(<>
      <Button size={size}>Save</Button>
      <FormField label="Name" size={size} />
      <FormField label="Date" size={size} type="date" />
      <Combobox data-testid="choice" onChange={vi.fn()} options={[]} size={size} />
      <SearchInput aria-label="Search" containerClassName="search-wrapper" size={size} />
    </>);
    const controls = [
      screen.getByRole("button", { name: "Save" }),
      screen.getByLabelText("Name"),
      screen.getByLabelText("Date"),
      screen.getByTestId("choice"),
      screen.getByRole("textbox", { name: "Search" }).parentElement!,
    ];
    for (const control of controls) {
      expect(control).toHaveClass(`h-[var(--ui-height-${size})]`);
      expect(control).toHaveClass("font-medium", "normal-case", "tracking-normal");
    }
    expect(CONTROL_SIZE_STYLES[size]).toContain(size === "lg" ? "text-base" : "text-sm");
  });

  it("generates unique accessible IDs without requiring name or id", () => {
    render(<><FormField label="First" /><FormField label="Second" /></>);
    expect(screen.getByLabelText("First").id).not.toBe(screen.getByLabelText("Second").id);
  });

  it("associates errors with inputs and textareas, clearing invalid state on recovery", () => {
    const { rerender } = render(<FormField error="Required" label="Notes" type="textarea" />);
    const field = screen.getByLabelText("Notes");
    expect(field).toHaveAttribute("aria-invalid", "true");
    expect(document.getElementById(field.getAttribute("aria-describedby")!)).toHaveTextContent("Required");
    expect(field).not.toHaveClass("h-[var(--ui-height-md)]");
    rerender(<FormField label="Notes" type="textarea" />);
    expect(field).not.toHaveAttribute("aria-invalid");
    expect(field).not.toHaveAttribute("aria-describedby");
  });
});
