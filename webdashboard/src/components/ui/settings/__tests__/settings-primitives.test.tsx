import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi } from "vitest";

import {
  SettingsGroup,
  SettingsRow,
  SettingsSwitch,
  SettingsSegmentedControl,
} from "../index";

describe("Apple HIG Settings UI Primitives", () => {
  describe("SettingsGroup", () => {
    it("renders title, description and children correctly", () => {
      render(
        <SettingsGroup
          description="Footer caption note"
          testId="group-test"
          title="Section Title"
        >
          <div>Group content</div>
        </SettingsGroup>
      );

      expect(screen.getByTestId("group-test")).toBeInTheDocument();
      expect(screen.getByText("Section Title")).toBeInTheDocument();
      expect(screen.getByText("Group content")).toBeInTheDocument();
      expect(screen.getByText("Footer caption note")).toBeInTheDocument();
    });
  });

  describe("SettingsRow", () => {
    it("renders label, description, and control", () => {
      render(
        <SettingsRow
          control={<span data-testid="custom-control">Control</span>}
          description="Row description text"
          label="Row Label"
        />
      );

      expect(screen.getByText("Row Label")).toBeInTheDocument();
      expect(screen.getByText("Row description text")).toBeInTheDocument();
      expect(screen.getByTestId("custom-control")).toBeInTheDocument();
    });

    it("triggers onClick when clicked or on Enter/Space key", () => {
      const handleClick = vi.fn();
      render(
        <SettingsRow
          disclosure
          label="Clickable Item"
          onClick={handleClick}
          testId="clickable-row"
        />
      );

      const row = screen.getByTestId("clickable-row");
      fireEvent.click(row);
      expect(handleClick).toHaveBeenCalledTimes(1);

      fireEvent.keyDown(row, { key: "Enter" });
      expect(handleClick).toHaveBeenCalledTimes(2);

      fireEvent.keyDown(row, { key: " " });
      expect(handleClick).toHaveBeenCalledTimes(3);
    });

    it("does not trigger onClick when disabled", () => {
      const handleClick = vi.fn();
      render(
        <SettingsRow
          disabled
          label="Disabled Item"
          onClick={handleClick}
          testId="disabled-row"
        />
      );

      const row = screen.getByTestId("disabled-row");
      fireEvent.click(row);
      expect(handleClick).not.toHaveBeenCalled();
    });

    it("applies destructive styling when destructive is true", () => {
      render(<SettingsRow destructive label="Delete Item" />);
      const label = screen.getByText("Delete Item");
      expect(label.className).toContain("text-rose-600");
    });
  });

  describe("SettingsSwitch", () => {
    it("renders role switch with checked state and toggles", () => {
      const handleChange = vi.fn();
      render(
        <SettingsSwitch
          ariaLabel="Toggle notifications"
          checked={false}
          onCheckedChange={handleChange}
          testId="apple-switch"
        />
      );

      const toggle = screen.getByTestId("apple-switch");
      expect(toggle).toHaveAttribute("role", "switch");
      expect(toggle).toHaveAttribute("aria-checked", "false");

      fireEvent.click(toggle);
      expect(handleChange).toHaveBeenCalledWith(true);
    });

    it("does not toggle when disabled", () => {
      const handleChange = vi.fn();
      render(
        <SettingsSwitch
          checked={true}
          disabled
          onCheckedChange={handleChange}
          testId="disabled-switch"
        />
      );

      const toggle = screen.getByTestId("disabled-switch");
      expect(toggle).toBeDisabled();
      fireEvent.click(toggle);
      expect(handleChange).not.toHaveBeenCalled();
    });
  });

  describe("SettingsSegmentedControl", () => {
    it("renders options and triggers onChange when clicked", () => {
      const handleChange = vi.fn();
      const options = [
        { value: "day", label: "Day" },
        { value: "week", label: "Week" },
        { value: "month", label: "Month", disabled: true },
      ];

      render(
        <SettingsSegmentedControl
          onChange={handleChange}
          options={options}
          value="day"
        />
      );

      expect(screen.getByRole("tablist")).toBeInTheDocument();
      const dayTab = screen.getByRole("tab", { name: "Day" });
      const weekTab = screen.getByRole("tab", { name: "Week" });
      const monthTab = screen.getByRole("tab", { name: "Month" });

      expect(dayTab).toHaveAttribute("aria-selected", "true");
      expect(weekTab).toHaveAttribute("aria-selected", "false");
      expect(monthTab).toBeDisabled();

      fireEvent.click(weekTab);
      expect(handleChange).toHaveBeenCalledWith("week");

      fireEvent.click(monthTab);
      expect(handleChange).toHaveBeenCalledTimes(1);
    });
  });
});
