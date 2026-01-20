import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { MonthYearPicker } from "../MonthYearPicker";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: any) => {
        if (key === "common.month_short") return `Tháng ${options.month}`;
        if (key === "common.filters.last_month") return "Tháng trước";
        if (key === "common.filters.this_month") return "Tháng này";
        return key;
    },
  }),
}));

describe("MonthYearPicker", () => {
  const onChange = vi.fn();

  it("renders correctly with provided month and year", () => {
    render(<MonthYearPicker month={5} year={2026} onChange={onChange} />);

    expect(screen.getByText("2026")).toBeDefined();
    // Check if 12 months are rendered
    for (let i = 1; i <= 12; i++) {
      expect(screen.getByText(`Tháng ${i}`)).toBeDefined();
    }
  });

  it("calls onChange when a month is clicked", () => {
    render(<MonthYearPicker month={5} year={2026} onChange={onChange} />);

    fireEvent.click(screen.getByText("Tháng 10"));
    expect(onChange).toHaveBeenCalledWith(10, 2026);
  });

  it("calls onChange with updated year when arrows are clicked", () => {
    render(<MonthYearPicker month={5} year={2026} onChange={onChange} />);

    const buttons = screen.getAllByRole("button");
    const prevYearBtn = buttons[0]; // First button is ChevronLeft for year
    const nextYearBtn = buttons[1]; // Second button is ChevronRight for year

    fireEvent.click(prevYearBtn);
    expect(onChange).toHaveBeenCalledWith(5, 2025);

    fireEvent.click(nextYearBtn);
    expect(onChange).toHaveBeenCalledWith(5, 2027);
  });

  it("sets last month when shortcut is clicked", () => {
    render(<MonthYearPicker month={5} year={2026} onChange={onChange} />);

    fireEvent.click(screen.getByText("Tháng trước"));
    
    const d = new Date();
    d.setMonth(d.getMonth() - 1);
    expect(onChange).toHaveBeenCalledWith(d.getMonth() + 1, d.getFullYear());
  });

  it("sets current month when shortcut is clicked", () => {
    render(<MonthYearPicker month={1} year={2020} onChange={onChange} />);

    fireEvent.click(screen.getByText("Tháng này"));
    
    const d = new Date();
    expect(onChange).toHaveBeenCalledWith(d.getMonth() + 1, d.getFullYear());
  });
});
