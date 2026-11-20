import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi, beforeEach } from "vitest";

import { exportAttendanceReport } from "@/lib/api/attendance-monitoring";

import ReportExportDialog from "../report-export-dialog";

vi.mock("@/lib/api/attendance-monitoring", () => ({
  exportAttendanceReport: vi.fn(),
}));

vi.mock("@/components/ui/modal", () => ({
  default: ({ isOpen, title, children, testId }: { isOpen?: boolean; title?: React.ReactNode; children?: React.ReactNode; testId?: string }) =>
    isOpen ? <div data-testid={testId ?? "modal"}><h2>{title}</h2>{children}</div> : null,
}));

describe("ReportExportDialog", () => {
  const onClose = vi.fn();
  const props = {
    isOpen: true,
    onClose,
    startDate: "2024-01-01",
    endDate: "2024-01-31",
    reportType: "summary" as const,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(exportAttendanceReport).mockResolvedValue(new Blob(["xlsx"]));
  });

  it("renders export details and actions", () => {
    render(<ReportExportDialog {...props} />);
    expect(screen.getByTestId("report-export-dialog")).toBeInTheDocument();
    expect(screen.getByTestId("report-export-info")).toHaveTextContent("2024-01-01");
    expect(screen.getByTestId("report-export-start-button")).toBeInTheDocument();
    expect(screen.getByTestId("report-export-cancel-button")).toBeInTheDocument();
  });

  it("posts synchronous export request and closes after download", async () => {
    const user = userEvent.setup();
    render(<ReportExportDialog {...props} />);

    await user.click(screen.getByTestId("report-export-start-button"));

    await waitFor(() => expect(exportAttendanceReport).toHaveBeenCalledWith({
      start_date: "2024-01-01",
      end_date: "2024-01-31",
      format: "xlsx",
      report_type: "summary",
      employee_ids: undefined,
    }));
    expect(onClose).toHaveBeenCalled();
  });

  it("passes selected employee IDs", async () => {
    const user = userEvent.setup();
    render(<ReportExportDialog {...props} employeeIds={["emp-1", "emp-2"]} />);

    await user.click(screen.getByTestId("report-export-start-button"));

    await waitFor(() => expect(exportAttendanceReport).toHaveBeenCalledWith(expect.objectContaining({
      employee_ids: ["emp-1", "emp-2"],
    })));
  });

  it("shows failure and retry action when export fails", async () => {
    vi.mocked(exportAttendanceReport).mockRejectedValueOnce(new Error("Server error"));
    const user = userEvent.setup();
    render(<ReportExportDialog {...props} />);

    await user.click(screen.getByTestId("report-export-start-button"));

    expect(await screen.findByTestId("report-export-error")).toHaveTextContent("Server error");
    expect(screen.getByTestId("report-export-retry-button")).toBeInTheDocument();
  });

  it("resets to idle when reopened", async () => {
    const { rerender } = render(<ReportExportDialog {...props} isOpen={false} />);
    rerender(<ReportExportDialog {...props} isOpen />);
    expect(screen.getByTestId("report-export-start-button")).toBeInTheDocument();
  });
});
