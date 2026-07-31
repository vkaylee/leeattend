import { render, screen, fireEvent } from "@testing-library/react";

import { afterEach, describe, it, expect, vi } from "vitest";

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

    const d = new Date(); d.setDate(1); d.setMonth(d.getMonth() - 1);
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

describe("MonthYearPicker — day-31 month arithmetic", () => {
  const onChange = vi.fn();

  afterEach(() => {
    vi.useRealTimers();
  });

  it("last-month shortcut shows June when today is July 31", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-07-31T12:00:00"));

    render(<MonthYearPicker month={7} onChange={onChange} year={2026} />);

    // Previous month should be June (6), NOT July (7)
    expect(screen.getByText("Tháng 6/2026")).toBeDefined();
    expect(screen.getByText("Tháng 7/2026")).toBeDefined();

    fireEvent.click(screen.getByText("Tháng 6/2026"));
    expect(onChange).toHaveBeenCalledWith(6, 2026);
  });

  it("last-month shortcut shows February when today is March 31", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-03-31T12:00:00"));

    render(<MonthYearPicker month={3} onChange={onChange} year={2026} />);

    expect(screen.getByText("Tháng 2/2026")).toBeDefined();

    fireEvent.click(screen.getByText("Tháng 2/2026"));
    expect(onChange).toHaveBeenCalledWith(2, 2026);
  });

  it("last-month shortcut shows December of previous year when today is January 31", () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-01-31T12:00:00"));

    render(<MonthYearPicker month={1} onChange={onChange} year={2026} />);

    expect(screen.getByText("Tháng 12/2025")).toBeDefined();

    fireEvent.click(screen.getByText("Tháng 12/2025"));
    expect(onChange).toHaveBeenCalledWith(12, 2025);
  });
});
