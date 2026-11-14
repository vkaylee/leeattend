import { fireEvent, render, screen } from "@testing-library/react";
import React from "react";

import { describe, expect, it, vi } from "vitest";

import { EmployeeIdentityCell, getEmployeeInitials } from "../employee-identity-cell";

describe("getEmployeeInitials", () => {
  it("calculates initials correctly for single and multi-word names", () => {
    expect(getEmployeeInitials("John")).toBe("JO");
    expect(getEmployeeInitials("Nguyen Van A")).toBe("NA");
    expect(getEmployeeInitials("")).toBe("?");
  });
});

describe("EmployeeIdentityCell", () => {
  it("renders employee name and code", () => {
    render(<EmployeeIdentityCell code="EMP001" name="Nguyen Van A" />);

    expect(screen.getByText("Nguyen Van A")).toBeInTheDocument();
    expect(screen.getByText("EMP001")).toBeInTheDocument();
    expect(screen.getByText("NA")).toBeInTheDocument();
  });

  it("triggers onClick when clicked as a button", () => {
    const handleClick = vi.fn();
    render(<EmployeeIdentityCell code="EMP002" name="Tran Thi B" onClick={handleClick} />);

    const button = screen.getByRole("button", { name: /Chi tiết nhân sự: Tran Thi B/i });
    expect(button).toBeInTheDocument();

    fireEvent.click(button);
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it("renders employer name when provided", () => {
    render(
      <EmployeeIdentityCell
        code="EMP003"
        employerName="Tech Corp"
        name="Le Van C"
      />
    );

    expect(screen.getByText("Tech Corp")).toBeInTheDocument();
  });
});
