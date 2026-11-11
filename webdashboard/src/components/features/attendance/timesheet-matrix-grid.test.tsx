import "@testing-library/jest-dom";
import { render, screen } from "@testing-library/react";
import React from "react";

import { describe, expect, it } from "vitest";

import type { Employee } from "@/lib/api/types";

import { TimesheetMatrixGrid } from "./timesheet-matrix-grid";

const employee = {
  id: "employee-1",
  full_name: "Test Employee",
  employee_id_number: "EMP-001",
  department_name: "Operations",
} as Employee;

describe("TimesheetMatrixGrid", () => {
  it("keeps monthly matrix scrollable with pinned identity and summary columns", () => {
    render(
      <TimesheetMatrixGrid
        employees={[employee]}
        month={1}
        year={2026}
      />,
    );

    const grid = screen.getByTestId("timesheet-matrix-grid");
    const table = screen.getByRole("table");
    const headers = screen.getAllByRole("columnheader");

    expect(grid.querySelector("div.overflow-auto")).toHaveClass("overscroll-contain");
    expect(headers).toHaveLength(35);
    expect(headers[0]).toHaveClass("sticky", "left-0");
    expect(headers.at(-1)).toHaveClass("sticky", "right-0");
    expect(table.querySelector("td.sticky.right-\\[112px\\]"))
      .toBeInTheDocument();
  });
});
