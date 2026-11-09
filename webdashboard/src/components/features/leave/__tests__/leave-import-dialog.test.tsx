import { render, screen, fireEvent, waitFor } from "@testing-library/react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, afterEach, type Mock } from "vitest";

import * as api from "@/lib/api";

import LeaveImportDialog from "../leave-import-dialog";

// Mock API
vi.mock("@/lib/api", () => ({
  uploadLeaveExcel: vi.fn(),
  fetchLatestLeaveImport: vi.fn().mockResolvedValue(null),
  downloadLeaveImportTemplate: vi.fn().mockResolvedValue(undefined),
  downloadLeaveImportErrors: vi.fn().mockResolvedValue(undefined),
}));

// Mock i18n — return key as text for testability
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback?: string) => fallback ?? key,
    i18n: { language: "vi" },
  }),
}));

const createFile = (name = "test.xlsx", size = 1024) => {
  const content = new ArrayBuffer(size);
  return new File([content], name, {
    type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  });
};

describe("LeaveImportDialog", () => {
  let onClose: () => void;
  let onImportComplete: () => void;

  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers({ shouldAdvanceTime: true });
    onClose = vi.fn();
    onImportComplete = vi.fn();
 
    (api.fetchLatestLeaveImport as Mock).mockResolvedValue(null);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  const renderDialog = (props: Partial<Parameters<typeof LeaveImportDialog>[0]> = {}) =>
    render(
      <LeaveImportDialog
        isOpen={true}
        onClose={onClose}
        onImportComplete={onImportComplete}
        {...props}
      />
    );

  // ═══════════════════════════════════════════════════════════════════
  // Rendering
  // ═══════════════════════════════════════════════════════════════════

  describe("Rendering", () => {
    it("renders nothing when isOpen is false", () => {
      renderDialog({ isOpen: false });
      expect(screen.queryByTestId("leave-import-dialog")).toBeNull();
    });

    it("renders modal with title and subtitle when isOpen", () => {
      renderDialog();
      expect(screen.getByText(/Import đơn nghỉ phép/i)).toBeDefined();
      expect(screen.getByText(/Upload file Excel/i)).toBeDefined();
    });

    it("renders instructions panel with all 6 column requirements", () => {
      renderDialog();
      expect(screen.getByText(/Yêu cầu file/i)).toBeDefined();
      expect(screen.getByText(/Cột employee_code: Mã nhân viên/i)).toBeDefined();
      expect(screen.getByText(/Cột leave_type_code: Mã loại nghỉ/i)).toBeDefined();
      expect(screen.getByText(/Cột start_date: Ngày bắt đầu/i)).toBeDefined();
      expect(screen.getByText(/Cột end_date: Ngày kết thúc/i)).toBeDefined();
      expect(screen.getByText(/Cột reason: Lý do/i)).toBeDefined();
      expect(screen.getByText(/Cột duration_type: Loại thời gian/i)).toBeDefined();
      expect(screen.getAllByText(/\.xlsx, \.xls/i).length).toBeGreaterThanOrEqual(1);
    });

    it("renders download template button", () => {
      renderDialog();
      expect(screen.getByText(/Tải file mẫu Excel/i)).toBeDefined();
    });

    it("renders file drop zone with prompt", () => {
      renderDialog();
      expect(screen.getByText(/Nhấn để chọn file Excel/i)).toBeDefined();
      expect(screen.getByText(/Hỗ trợ .xlsx, .xls/i)).toBeDefined();
    });

    it("renders cancel and import buttons", () => {
      renderDialog();
      expect(screen.getByText("Hủy")).toBeDefined();
      expect(screen.getByText(/BẮT ĐẦU IMPORT/i)).toBeDefined();
    });

    it("disables import button when no file selected", () => {
      renderDialog();
      const importButton = screen.getByText(/BẮT ĐẦU IMPORT/i).closest("button");
      expect(importButton).toBeDisabled();
    });
  });

  // ═══════════════════════════════════════════════════════════════════
  // File Selection
  // ═══════════════════════════════════════════════════════════════════

  describe("File Selection", () => {
    it("shows file info when a file is selected", async () => {
      renderDialog();
      const fileInput = document.getElementById("leave-import-file-input") as HTMLInputElement;
      expect(fileInput).toBeDefined();

      const file = createFile("leave_data.xlsx", 2048);
      fireEvent.change(fileInput, { target: { files: [file] } });

      expect(await screen.findByText("leave_data.xlsx")).toBeDefined();
      expect(screen.getByText(/2\.0 KB/i)).toBeDefined();
      expect(screen.getByText(/SẴN SÀNG UPLOAD/i)).toBeDefined();
    });

    it("enables import button after file selection", () => {
      renderDialog();
      const fileInput = document.getElementById("leave-import-file-input") as HTMLInputElement;
      const file = createFile();
      fireEvent.change(fileInput, { target: { files: [file] } });

      const importButton = screen.getByText(/BẮT ĐẦU IMPORT/i).closest("button");
      expect(importButton).not.toBeDisabled();
    });

    it("shows 'select another file' button and allows file removal", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      renderDialog();
      const fileInput = document.getElementById("leave-import-file-input") as HTMLInputElement;
      fireEvent.change(fileInput, { target: { files: [createFile()] } });

      const removeButton = await screen.findByText(/Chọn file khác/i);
      expect(removeButton).toBeDefined();

      await user.click(removeButton);
      // Drop zone prompt should reappear
      expect(screen.getByText(/Nhấn để chọn file Excel/i)).toBeDefined();
    });
  });

  // ═══════════════════════════════════════════════════════════════════
  // Template Download
  // ═══════════════════════════════════════════════════════════════════

  describe("Template Download", () => {
    it("calls downloadLeaveImportTemplate when button is clicked", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
 
      (api.downloadLeaveImportTemplate as Mock).mockResolvedValue(undefined);
      renderDialog();

      const downloadButton = screen.getByText(/Tải file mẫu Excel/i).closest("button")!;
      await user.click(downloadButton);

      await waitFor(() => {
        expect(api.downloadLeaveImportTemplate).toHaveBeenCalledWith("vi");
      });
    });

    it("handles template download error gracefully", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
 
      (api.downloadLeaveImportTemplate as Mock).mockRejectedValue(new Error("Network error"));
      renderDialog();

      const downloadButton = screen.getByText(/Tải file mẫu Excel/i).closest("button")!;
      // Should not throw
      await user.click(downloadButton);
      
      // Button should still be clickable after error
      await waitFor(() => {
        expect(downloadButton).not.toBeDisabled();
      });
    });
  });

  // ═══════════════════════════════════════════════════════════════════
  // Upload Flow
  // ═══════════════════════════════════════════════════════════════════

  describe("Upload Flow", () => {
    it("calls uploadLeaveExcel with correct args when import is clicked", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
 
      (api.uploadLeaveExcel as Mock).mockResolvedValue({ job_id: "job-123" });
      renderDialog();

      const fileInput = document.getElementById("leave-import-file-input") as HTMLInputElement;
      const file = createFile("data.xlsx");
      fireEvent.change(fileInput, { target: { files: [file] } });

      const importButton = screen.getByText(/BẮT ĐẦU IMPORT/i).closest("button")!;
      await user.click(importButton);

      await waitFor(() => {
        expect(api.uploadLeaveExcel).toHaveBeenCalledWith(file);
      });
    });

    it("transitions to progress view after successful upload", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
 
      (api.uploadLeaveExcel as Mock).mockResolvedValue({ job_id: "job-456" });
      renderDialog();

      const fileInput = document.getElementById("leave-import-file-input") as HTMLInputElement;
      fireEvent.change(fileInput, { target: { files: [createFile()] } });

      await user.click(screen.getByText(/BẮT ĐẦU IMPORT/i).closest("button")!);

      // Should switch to progress view
      await waitFor(() => {
        expect(screen.getByText(/Đang xử lý.../i)).toBeDefined();
      });
      // Upload view elements should be gone
      expect(screen.queryByText(/Nhấn để chọn file Excel/i)).toBeNull();
    });

    it("handles upload error gracefully (stays on upload view)", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
 
      (api.uploadLeaveExcel as Mock).mockRejectedValue(new Error("Upload failed"));
      renderDialog();

      const fileInput = document.getElementById("leave-import-file-input") as HTMLInputElement;
      fireEvent.change(fileInput, { target: { files: [createFile()] } });

      await user.click(screen.getByText(/BẮT ĐẦU IMPORT/i).closest("button")!);

      // Should stay on upload view after error
      await waitFor(() => {
        expect(screen.getByText(/BẮT ĐẦU IMPORT/i)).toBeDefined();
      });
    });

    it("does not upload when no file is selected", () => {
      renderDialog();
      const importButton = screen.getByText(/BẮT ĐẦU IMPORT/i).closest("button")!;
      expect(importButton).toBeDisabled();
      expect(api.uploadLeaveExcel).not.toHaveBeenCalled();
    });

  });

  // ═══════════════════════════════════════════════════════════════════
  // Progress / Status Views
  // ═══════════════════════════════════════════════════════════════════

  describe("Progress View", () => {
    it("shows processing status with progress rows", async () => {
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-1",
        status: "processing",
        total_rows: 100,
        processed_rows: 45,
        error_log: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      await waitFor(() => {
        expect(screen.getByText(/Đang xử lý.../i)).toBeDefined();
      });
      expect(screen.getByText(/45 \/ 100/i)).toBeDefined();
      expect(screen.getByText("45%")).toBeDefined();
    });

    it("shows completed status without errors", async () => {
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-2",
        status: "completed",
        total_rows: 50,
        processed_rows: 50,
        error_log: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      await waitFor(() => {
        expect(screen.getByText(/Import thành công/i)).toBeDefined();
      });
      expect(screen.getByText("100%")).toBeDefined();
    });

    it("shows partial success status with error details and download", async () => {
      const errorLog = JSON.stringify({
        success_count: 40,
        error_count: 10,
        error_file_path: "/tmp/errors.xlsx",
      });

 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-3",
        status: "completed",
        total_rows: 50,
        processed_rows: 40,
        error_log: errorLog,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      await waitFor(() => {
        expect(screen.getByText(/Import có lỗi/i)).toBeDefined();
      });
      // Partial detail text
      expect(screen.getByText(/40.*10/)).toBeDefined();
      // Error file download button
      expect(screen.getByText(/Tải file lỗi/i)).toBeDefined();
    });

    it("calls downloadLeaveImportErrors when clicking download errors button", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      const errorLog = JSON.stringify({
        success_count: 40,
        error_count: 10,
        error_file_path: "/tmp/errors.xlsx",
      });

 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-err",
        status: "completed",
        total_rows: 50,
        processed_rows: 40,
        error_log: errorLog,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      const errorButton = await screen.findByText(/Tải file lỗi/i);
      await user.click(errorButton);

      expect(api.downloadLeaveImportErrors).toHaveBeenCalledWith("job-err");
    });

    it("shows failed status with error detail", async () => {
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-4",
        status: "failed",
        total_rows: 0,
        processed_rows: 0,
        error_log: "Connection timeout to database",
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      await waitFor(() => {
        expect(screen.getByText(/Import thất bại/i)).toBeDefined();
      });
      expect(screen.getByText(/Chi tiết lỗi/i)).toBeDefined();
      expect(screen.getByText(/Connection timeout/i)).toBeDefined();
    });

    it("shows footer buttons (upload another + close) when finished", async () => {
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-5",
        status: "completed",
        total_rows: 50,
        processed_rows: 50,
        error_log: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      await waitFor(() => {
        expect(screen.getByText(/Upload file khác/i)).toBeDefined();
      });
      expect(screen.getByText(/ĐÓNG/i)).toBeDefined();
    });

    it("hides footer buttons while processing", async () => {
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-6",
        status: "processing",
        total_rows: 100,
        processed_rows: 10,
        error_log: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      await waitFor(() => {
        expect(screen.getByText(/Đang xử lý.../i)).toBeDefined();
      });
      expect(screen.queryByText(/Upload file khác/i)).toBeNull();
      expect(screen.queryByText(/ĐÓNG/i)).toBeNull();
    });

    it("resets to upload view when 'upload another' is clicked", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-7",
        status: "completed",
        total_rows: 10,
        processed_rows: 10,
        error_log: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      const uploadAnotherButton = await screen.findByText(/Upload file khác/i);
      await user.click(uploadAnotherButton);

      // Should show upload view again
      await waitFor(() => {
        expect(screen.getByText(/Nhấn để chọn file Excel/i)).toBeDefined();
      });
    });
  });

  // ═══════════════════════════════════════════════════════════════════
  // Callbacks
  // ═══════════════════════════════════════════════════════════════════

  describe("Callbacks", () => {
    it("calls onClose when cancel button is clicked", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
      renderDialog();

      await user.click(screen.getByText("Hủy"));
      expect(onClose).toHaveBeenCalled();
    });

    it("calls onClose when close button is clicked after completion", async () => {
      const user = userEvent.setup({ advanceTimers: vi.advanceTimersByTime });
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-done",
        status: "completed",
        total_rows: 10,
        processed_rows: 10,
        error_log: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();
      const closeButton = await screen.findByText(/ĐÓNG/i);
      await user.click(closeButton);
      expect(onClose).toHaveBeenCalled();
    });
  });

  // ═══════════════════════════════════════════════════════════════════
  // Latest Job Recovery (offline resilience)
  // ═══════════════════════════════════════════════════════════════════

  describe("Job Recovery", () => {
    it("fetches latest import job on mount when open", async () => {
      renderDialog();
      await waitFor(() => {
        expect(api.fetchLatestLeaveImport).toHaveBeenCalledWith();
      });
    });

    it("does NOT fetch latest job when isOpen is false", () => {
      renderDialog({ isOpen: false });
      expect(api.fetchLatestLeaveImport).not.toHaveBeenCalled();
    });

    it("ignores jobs older than 24 hours", async () => {
      const oldDate = new Date(Date.now() - 25 * 60 * 60 * 1000).toISOString();
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-old",
        status: "completed",
        total_rows: 10,
        processed_rows: 10,
        error_log: null,
        created_at: oldDate,
        updated_at: oldDate,
      });

      renderDialog();

      // Should show upload view, not progress view
      await waitFor(() => {
        expect(screen.getByText(/Nhấn để chọn file Excel/i)).toBeDefined();
      });
    });

    it("shows existing processing job when dialog opens", async () => {
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-active",
        status: "processing",
        total_rows: 200,
        processed_rows: 80,
        error_log: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      await waitFor(() => {
        expect(screen.getByText(/Đang xử lý.../i)).toBeDefined();
      });
      expect(screen.getByText(/80 \/ 200/i)).toBeDefined();
    });

    it("handles API error when fetching latest job", async () => {
 
      (api.fetchLatestLeaveImport as Mock).mockRejectedValue(new Error("Server error"));
      renderDialog();
      // Should fall back to upload view
      await waitFor(() => {
        expect(screen.getByText(/Nhấn để chọn file Excel/i)).toBeDefined();
      });
    });
  });

  // ═══════════════════════════════════════════════════════════════════
  // Edge Cases
  // ═══════════════════════════════════════════════════════════════════

  describe("Edge Cases", () => {
    it("resets file state when dialog closes and reopens", () => {
      const { rerender } = render(
        <LeaveImportDialog isOpen={true} onClose={onClose} />
      );

      // Select a file
      const fileInput = document.getElementById("leave-import-file-input") as HTMLInputElement;
      fireEvent.change(fileInput, { target: { files: [createFile()] } });
      expect(screen.getByText("test.xlsx")).toBeDefined();

      // Close dialog
      rerender(<LeaveImportDialog isOpen={false} onClose={onClose} />);
      // Reopen
      rerender(<LeaveImportDialog isOpen={true} onClose={onClose} />);

      // Should be reset
      expect(screen.getByText(/Nhấn để chọn file Excel/i)).toBeDefined();
    });

    it("handles completed job with unparseable error_log", async () => {
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-bad-log",
        status: "completed",
        total_rows: 20,
        processed_rows: 15,
        error_log: "not-json-just-text",
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      await waitFor(() => {
        expect(screen.getByText(/Import có lỗi/i)).toBeDefined();
      });
      // Should show partial detail with fallback values
      expect(screen.getAllByText(/15/).length).toBeGreaterThanOrEqual(1);
    });

    it("handles pending job with zero total_rows", async () => {
 
      (api.fetchLatestLeaveImport as Mock).mockResolvedValue({
        id: "job-pending",
        status: "pending",
        total_rows: 0,
        processed_rows: 0,
        error_log: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
      });

      renderDialog();

      await waitFor(() => {
        expect(screen.getByText(/Đang xử lý.../i)).toBeDefined();
      });
      // Should show "0 / ?" for rows
      expect(screen.getByText(/0 \/ \?/)).toBeDefined();
    });
  });
});
