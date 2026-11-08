import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { act, renderHook, waitFor } from "@testing-library/react";
import React from "react";

import { beforeEach, describe, expect, it, vi } from "vitest";

import * as employeesApi from "@/lib/api/employees";
import * as leavesApi from "@/lib/api/leaves";

import { useImportStateMachine } from "../hooks/use-import-state-machine";

vi.mock("@/lib/api/employees", () => ({
  uploadTerminationImportPreview: vi.fn(),
  confirmTerminationImport: vi.fn(),
}));

vi.mock("@/lib/api/leaves", () => ({
  fetchJobStatus: vi.fn(),
}));

vi.mock("@/components/ui/toast", () => ({
  toast: { error: vi.fn(), success: vi.fn(), warning: vi.fn() },
}));

vi.mock("@/lib/logger", () => ({
  logError: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { defaultValue?: string }) => opts?.defaultValue ?? key,
  }),
}));

function createWrapper() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const Wrapper = ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
  Wrapper.displayName = "QueryClientWrapper";
  return Wrapper;
}

const PREVIEW = {
  valid_rows: [
    {
      row_number: 1,
      employee_code: "EMP001",
      full_name: "Alice",
      employment_id: "e-1",
      start_date: "2022-01-01",
      termination_date: "2025-06-01",
      termination_reason: "resignation",
      termination_notes: "",
      prorated_leaves: [],
    },
  ],
  invalid_rows: [],
  prorated_leave_configs: [],
};

const XLSX_FILE = new File(["x"], "terminations.xlsx");

function uploadFile(result: { current: { setFile: (f: File | null) => void } }) {
  act(() => {
    result.current.setFile(XLSX_FILE);
  });
}

describe("useImportStateMachine", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("starts in the upload step with empty state", () => {
    const { result } = renderHook(() => useImportStateMachine("ws-1", vi.fn()), {
      wrapper: createWrapper(),
    });

    expect(result.current.file).toBeNull();
    expect(result.current.step).toBe(1);
    expect(result.current.loading).toBe(false);
    expect(result.current.confirming).toBe(false);
    expect(result.current.importSuccess).toBe(false);
    expect(result.current.previewData).toBeNull();
    expect(result.current.jobError).toBeNull();
  });

  it("uploads the file and moves to the preview step with the valid tab", async () => {
    vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValue(PREVIEW);
    const { result } = renderHook(() => useImportStateMachine("ws-1", vi.fn()), {
      wrapper: createWrapper(),
    });

    uploadFile(result);
    await act(async () => {
      await result.current.handleUpload();
    });

    expect(employeesApi.uploadTerminationImportPreview).toHaveBeenCalledWith(XLSX_FILE);
    expect(result.current.previewData).toEqual(PREVIEW);
    expect(result.current.step).toBe(2);
    expect(result.current.activeTab).toBe("valid");
  });

  it("selects the invalid tab when the preview has no valid rows", async () => {
    vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValue({
      ...PREVIEW,
      valid_rows: [],
      invalid_rows: [
        { row_number: 2, employee_code: "EMP002", raw_termination_date: "bad", reason: "Invalid date" },
      ],
    });
    const { result } = renderHook(() => useImportStateMachine("ws-1", vi.fn()), {
      wrapper: createWrapper(),
    });

    uploadFile(result);
    await act(async () => {
      await result.current.handleUpload();
    });

    expect(result.current.step).toBe(2);
    expect(result.current.activeTab).toBe("invalid");
  });

  it("does not upload when no file is selected", async () => {
    const { result } = renderHook(() => useImportStateMachine("ws-1", vi.fn()), {
      wrapper: createWrapper(),
    });

    await act(async () => {
      await result.current.handleUpload();
    });

    expect(employeesApi.uploadTerminationImportPreview).not.toHaveBeenCalled();
  });

  it("completes synchronously when confirmation returns no queued job", async () => {
    vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValue(PREVIEW);
    vi.mocked(employeesApi.confirmTerminationImport).mockResolvedValue({
      job_id: null,
      success_count: 1,
      errors: [],
    });
    const { result } = renderHook(() => useImportStateMachine("ws-1", vi.fn()), {
      wrapper: createWrapper(),
    });

    uploadFile(result);
    await act(async () => {
      await result.current.handleUpload();
    });
    await act(async () => {
      await result.current.handleConfirm();
    });

    expect(employeesApi.confirmTerminationImport).toHaveBeenCalledTimes(1);
    expect(result.current.importSuccess).toBe(true);
    expect(result.current.successCount).toBe(1);
    expect(result.current.confirming).toBe(false);
  });

  it("polls a queued job and marks success when the job completes", async () => {
    vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValue(PREVIEW);
    vi.mocked(employeesApi.confirmTerminationImport).mockResolvedValue({
      job_id: "job-1",
      success_count: 3,
      errors: [],
    });
    vi.mocked(leavesApi.fetchJobStatus).mockResolvedValue({ status: "completed" });
    const { result } = renderHook(() => useImportStateMachine("ws-1", vi.fn()), {
      wrapper: createWrapper(),
    });

    uploadFile(result);
    await act(async () => {
      await result.current.handleUpload();
    });
    await act(async () => {
      await result.current.handleConfirm();
    });

    await waitFor(() => {
      expect(result.current.importSuccess).toBe(true);
    });
    expect(leavesApi.fetchJobStatus).toHaveBeenCalledWith("job-1");
    expect(result.current.successCount).toBe(3);
    expect(result.current.jobId).toBeNull();
  });

  it("surfaces the error message when the job fails", async () => {
    vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValue(PREVIEW);
    vi.mocked(employeesApi.confirmTerminationImport).mockResolvedValue({
      job_id: "job-1",
      success_count: 0,
      errors: [],
    });
    vi.mocked(leavesApi.fetchJobStatus).mockResolvedValue({
      status: "failed",
      error_message: "Boom",
    });
    const { result } = renderHook(() => useImportStateMachine("ws-1", vi.fn()), {
      wrapper: createWrapper(),
    });

    uploadFile(result);
    await act(async () => {
      await result.current.handleUpload();
    });
    await act(async () => {
      await result.current.handleConfirm();
    });

    await waitFor(() => {
      expect(result.current.jobError).toBe("Boom");
    });
    expect(result.current.confirming).toBe(false);
  });

  it("clears the error and stops confirming when confirmation fails", async () => {
    vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValue(PREVIEW);
    vi.mocked(employeesApi.confirmTerminationImport).mockRejectedValue(new Error("network"));
    const { result } = renderHook(() => useImportStateMachine("ws-1", vi.fn()), {
      wrapper: createWrapper(),
    });

    uploadFile(result);
    await act(async () => {
      await result.current.handleUpload();
    });
    await act(async () => {
      await result.current.handleConfirm();
    });

    expect(result.current.jobError).toBe("Lỗi kết nối hoặc yêu cầu không hợp lệ.");
    expect(result.current.confirming).toBe(false);
  });

  it("resets state and calls onClose from handleClose", async () => {
    const onClose = vi.fn();
    vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValue(PREVIEW);
    const { result } = renderHook(() => useImportStateMachine("ws-1", onClose), {
      wrapper: createWrapper(),
    });

    uploadFile(result);
    await act(async () => {
      await result.current.handleUpload();
    });
    act(() => {
      result.current.handleClose();
    });

    expect(onClose).toHaveBeenCalled();
    expect(result.current.step).toBe(1);
    expect(result.current.file).toBeNull();
    expect(result.current.previewData).toBeNull();
    expect(result.current.importSuccess).toBe(false);
  });
});
