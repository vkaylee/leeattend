import { render, screen, fireEvent } from "@testing-library/react";

import { describe, it, expect, vi } from "vitest";

import { MonthYearPicker } from "../month-year-picker";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { month: number }) => {
        if (key === "common.month_short" && options) return `Tháng ${options.month}`;
        return key;
    },
  }),
}));

describe("MonthYearPicker", () => {
  const onChange = vi.fn();

  it("renders correctly with provided month and year", () => {
    render(<MonthYearPicker month={5} onChange={onChange} year={2026} />);

    expect(screen.getByText("2026")).toBeDefined();
    // Check if 12 months are rendered as numbers
    for (let index = 1; index <= 12; index++) {
      expect(screen.getByText(index.toString())).toBeDefined();
    }
  });

  it("calls onChange when a month is clicked", () => {
    render(<MonthYearPicker month={5} onChange={onChange} year={2026} />);

    // Select by aria-label since text is just "10"
    fireEvent.click(screen.getByLabelText("Tháng 10"));
    expect(onChange).toHaveBeenCalledWith(10, 2026);
  });

  it("calls onChange with updated year when arrows are clicked", () => {
    render(<MonthYearPicker month={5} onChange={onChange} year={2026} />);

    const buttons = screen.getAllByRole("button");
    const previousYearButton = buttons[0]; // First button is ChevronLeft for year
    const nextYearButton = buttons[1]; // Second button is ChevronRight for year

    fireEvent.click(previousYearButton);
    expect(onChange).toHaveBeenCalledWith(5, 2025);

    fireEvent.click(nextYearButton);
    expect(onChange).toHaveBeenCalledWith(5, 2027);
  });

  it("sets last month when shortcut is clicked", () => {
    render(<MonthYearPicker month={5} onChange={onChange} year={2026} />);

    const d = new Date();
    d.setMonth(d.getMonth() - 1);
    const prevMonth = d.getMonth() + 1;
    const prevYear = d.getFullYear();

    fireEvent.click(screen.getByText(`Tháng ${prevMonth}/${prevYear}`));
    expect(onChange).toHaveBeenCalledWith(prevMonth, prevYear);
  });

  it("sets current month when shortcut is clicked", () => {
    render(<MonthYearPicker month={1} onChange={onChange} year={2020} />);

    const d = new Date();
    const curMonth = d.getMonth() + 1;
    const curYear = d.getFullYear();

    fireEvent.click(screen.getByText(`Tháng ${curMonth}/${curYear}`));
    expect(onChange).toHaveBeenCalledWith(curMonth, curYear);
  });
});
