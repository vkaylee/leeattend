import { render, screen } from "@testing-library/react";
import { createRef } from "react";

import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";

import type { StagingLog } from "@/lib/api/attendance";

import Button from "@/components/ui/button";

import { StagingLogsTable } from "../staging-logs-table";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => (typeof fallback === "string" ? fallback : key),
  }),
}));

vi.mock("@/components/common/employee-search", () => ({
  EmployeeSearch: ({ onChange }: { onChange: (value: string) => void }) => (
    <Button
      data-testid="mock-employee-search-pick"
      onClick={() => { onChange("emp-existing-1"); }}
      type="button"
    >
      Pick Existing Employee
    </Button>
  ),
}));

const sampleLog: StagingLog = {
  id: "99999999-9999-9999-9999-999999999999",
  raw_user_id: "BADGE-7788",
  punch_time: "2026-11-25T08:05:00Z",
  status: "unmatched",
  raw_data: null,
  created_at: "2026-11-25T08:05:00Z",
};

describe("StagingLogsTable", () => {
  it("routes Create through preflight with safe enrollment ID and staging log ID while preserving Assign and Dismiss", async () => {
    const user = userEvent.setup();
    const onCreateEmployee = vi.fn();
    const onResolve = vi.fn();
    const onDismiss = vi.fn();

    render(
      <StagingLogsTable
        formatDate={() => "2026-11-25"}
        formatTime={() => "08:05"}
        isDismissing={false}
        isFetchingNextPage={false}
        isLoading={false}
        loadMoreRef={createRef<HTMLTableRowElement>()}
        logs={[sampleLog]}
        onCreateEmployee={onCreateEmployee}
        onDismiss={onDismiss}
        onResolve={onResolve}
      />,
    );

    expect(screen.getAllByText("BADGE-7788").length).toBeGreaterThan(0);

    // Create forwards raw_user_id + staging log id to /employees/new preflight (never auto-resolves).
    await user.click(screen.getAllByText("attendance_logs.create_employee")[0]);
    expect(onCreateEmployee).toHaveBeenCalledWith("BADGE-7788", "99999999-9999-9999-9999-999999999999");

    // Assign existing employee still resolves staging row cleanly.
    await user.click(screen.getAllByText("attendance_logs.assign")[0]);
    await user.click(screen.getAllByTestId("mock-employee-search-pick")[0]);
    expect(onResolve).toHaveBeenCalledWith("99999999-9999-9999-9999-999999999999", "emp-existing-1");

    // Dismiss preserves staging row identity.
    await user.click(screen.getAllByText("common.delete")[0]);
    expect(onDismiss).toHaveBeenCalledWith("99999999-9999-9999-9999-999999999999");
  });
});
