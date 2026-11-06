import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, expect, it, vi } from "vitest";

import { AttendanceStatusBadge } from "../attendance-status-badge";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
  }),
}));

describe("AttendanceStatusBadge", () => {
  it("renders on_time status with green semantic style and label", () => {
    render(<AttendanceStatusBadge status="on_time" />);
    const badge = screen.getByTestId("attendance-status-on_time");
    expect(badge).toBeInTheDocument();
    expect(badge).toHaveTextContent("Đúng giờ");
    expect(badge.className).toContain("text-emerald-700");
  });

  it("renders late status with amber semantic style and optional minutes", () => {
    render(<AttendanceStatusBadge minutes={15} status="late" />);
    const badge = screen.getByTestId("attendance-status-late");
    expect(badge).toBeInTheDocument();
    expect(badge).toHaveTextContent("Đi muộn (15p)");
    expect(badge.className).toContain("text-amber-700");
  });

  it("renders absent status with rose semantic style", () => {
    render(<AttendanceStatusBadge status="absent" />);
    const badge = screen.getByTestId("attendance-status-absent");
    expect(badge).toBeInTheDocument();
    expect(badge).toHaveTextContent("Vắng mặt");
    expect(badge.className).toContain("text-rose-700");
  });

  it("renders overtime status with violet semantic style and duration", () => {
    render(<AttendanceStatusBadge minutes={120} status="overtime" />);
    const badge = screen.getByTestId("attendance-status-overtime");
    expect(badge).toBeInTheDocument();
    expect(badge).toHaveTextContent("Tăng ca (OT) (120p)");
    expect(badge.className).toContain("text-violet-700");
  });

  it("renders custom label when provided", () => {
    render(
      <AttendanceStatusBadge
        customLabel="Nghỉ thai sản (Hưởng BHXH)"
        status="leave_paid"
      />
    );
    const badge = screen.getByTestId("attendance-status-leave_paid");
    expect(badge).toBeInTheDocument();
    expect(badge).toHaveTextContent("Nghỉ thai sản (Hưởng BHXH)");
    expect(badge.className).toContain("text-sky-700");
  });
});
