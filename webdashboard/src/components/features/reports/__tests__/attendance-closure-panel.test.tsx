import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ApiError } from "@/lib/api/errors";

import AttendanceClosurePanel from "../attendance-closure-panel";

const mocks = vi.hoisted(() => ({
  close: vi.fn(),
  reopen: vi.fn(),
  refetch: vi.fn(),
  closureState: {
    data: {
      employer_id: "emp-org-1",
      year: 2026,
      month: 11,
      status: "open",
      payroll_status: "open",
      version: 1,
      closure_number: null,
      day_count: null,
      snapshot_hash: null,
    } as Record<string, unknown> | null,
    isLoading: false,
    isError: false,
    isClosing: false,
    isReopening: false,
    error: null as unknown,
  },
  permissions: ["attendance:closure:read", "attendance:closure:close", "attendance:closure:reopen"],
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasPermission: (p: string) => mocks.permissions.includes(p),
    isLoading: false,
  }),
}));

vi.mock("@/hooks/use-attendance-closure", () => ({
  useAttendanceClosure: () => ({
    ...mocks.closureState,
    close: mocks.close,
    reopen: mocks.reopen,
    refetch: mocks.refetch,
  }),
}));

describe("AttendanceClosurePanel", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.permissions = ["attendance:closure:read", "attendance:closure:close", "attendance:closure:reopen"];
    mocks.closureState = {
      data: {
        employer_id: "emp-org-1",
        year: 2026,
        month: 11,
        status: "open",
        payroll_status: "open",
        version: 1,
        closure_number: null,
        day_count: null,
        snapshot_hash: null,
      },
      isLoading: false,
      isError: false,
      isClosing: false,
      isReopening: false,
      error: null,
    };
  });

  it.each([
    ["error.missing_coverage", "closure.errors.missing_coverage"],
    ["error.open_punch", "closure.errors.open_punch"],
    ["error.stale_period", "closure.errors.stale_period"],
    ["error.locked_period", "closure.errors.locked_period"],
    ["error.closure_conflict", "closure.errors.closure_conflict"],
  ])("renders localized close error for backend wire message %s", async (messageKey, expectedKey) => {
    const user = userEvent.setup();
    mocks.close.mockRejectedValueOnce(new ApiError(409, "Conflict", "CONFLICT", { error_code: "CONFLICT", message: messageKey }));

    render(<AttendanceClosurePanel employerId="emp-org-1" month={11} year={2026} />);
    await user.type(screen.getByLabelText("closure.reason"), "Close month");
    await user.click(screen.getByRole("button", { name: "closure.actions.close" }));

    expect(await screen.findByRole("alert")).toHaveTextContent(expectedKey);
  });

  it("renders closed evidence and localized self-reopen forbidden message", async () => {
    const user = userEvent.setup();
    mocks.closureState.data = {
      employer_id: "emp-org-1",
      year: 2026,
      month: 11,
      status: "closed",
      payroll_status: "open",
      version: 2,
      closure_number: 1,
      day_count: 30,
      snapshot_hash: "abc123",
    };
    mocks.reopen.mockRejectedValueOnce(new ApiError(403, "Forbidden", "FORBIDDEN", { error_code: "FORBIDDEN", message: "error.self_reopen_forbidden" }));

    render(<AttendanceClosurePanel employerId="emp-org-1" month={11} year={2026} />);
    expect(screen.getByTestId("closure-evidence")).toHaveTextContent("abc123");

    await user.type(screen.getByLabelText("closure.reason"), "Attempt self reopen");
    await user.click(screen.getByRole("button", { name: "closure.actions.reopen" }));

    expect(await screen.findByRole("alert")).toHaveTextContent("closure.errors.self_reopen_forbidden");
  });

  it("disables close and reopen controls and displays locked status when payroll is locked", () => {
    mocks.closureState.data = {
      employer_id: "emp-org-1",
      year: 2026,
      month: 11,
      status: "closed",
      payroll_status: "locked",
      version: 3,
      closure_number: 1,
      day_count: 30,
      snapshot_hash: "lockedhash",
    };

    render(<AttendanceClosurePanel employerId="emp-org-1" month={11} year={2026} />);
    expect(screen.getByRole("status")).toHaveTextContent("closure.errors.locked_period");
    expect(screen.queryByRole("button", { name: "closure.actions.close" })).toBeNull();
    expect(screen.queryByRole("button", { name: "closure.actions.reopen" })).toBeNull();
  });
});
