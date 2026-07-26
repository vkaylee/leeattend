import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { beforeAll, describe, expect, it, vi } from "vitest";

import * as employeesApi from "@/lib/api/employees";

import EmployeeTerminationImportDialog from "../employee-termination-import-dialog";

// ---------------------------------------------------------------------------
// Mocks
// ---------------------------------------------------------------------------

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { defaultValue?: string }) =>
      opts?.defaultValue ?? key,
    i18n: { changeLanguage: () => Promise.resolve(), language: "en" },
  }),
}));

vi.mock("@/lib/api/employees", () => ({
  uploadTerminationImportPreview: vi.fn(),
  confirmTerminationImport: vi.fn(),
  downloadTerminationTemplate: vi.fn(),
}));

vi.mock("@/lib/api/leaves", () => ({
  fetchJobStatus: vi.fn(),
}));

// Mock toast
vi.mock("@/components/ui/toast", () => ({
  toast: { error: vi.fn(), success: vi.fn(), warning: vi.fn() },
  useToast: () => ({ error: vi.fn(), success: vi.fn(), warning: vi.fn() }),
}));

// Mock lazy exceljs + file-saver used by exportErrorsToExcel
vi.mock("exceljs", () => {
  const Workbook = vi.fn(() => ({
    creator: "",
    created: null,
    addWorksheet: vi.fn(() => ({
      columns: [],
      getRow: vi.fn(() => ({
        eachCell: vi.fn(),
      })),
      addRow: vi.fn(),
    })),
    eachRow: vi.fn(),
    xlsx: { writeBuffer: vi.fn().mockResolvedValue(new ArrayBuffer(8)) },
  }));
  return { default: { Workbook }, Workbook };
});
vi.mock("file-saver", () => ({ saveAs: vi.fn() }));

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const defaultPreview = {
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

function renderDialog(overrides = {}) {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const props = { isOpen: true, onClose: vi.fn(), wsId: "ws-1", ...overrides };
  return {
    onClose: props.onClose,
    ...render(
      <QueryClientProvider client={queryClient}>
        <EmployeeTerminationImportDialog {...props} />
      </QueryClientProvider>,
    ),
  };
}

// Helper: select a file in the file input
async function selectFile() {
  const file = new File(["test"], "termination.xlsx", {
    type: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  });
  const input = screen.getByLabelText("employees.bulk_termination.uploading_btn");
  await userEvent.upload(input, file);
  return file;
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

describe("EmployeeTerminationImportDialog", () => {
  beforeAll(() => {
    // Mock scrollIntoView for jsdom
    Element.prototype.scrollIntoView = vi.fn();
    Element.prototype.scrollTo = vi.fn();
  });

  describe("Step 1 — File Upload", () => {
    it("renders the file upload area when open", () => {
      renderDialog();
      expect(screen.getByText("employees.bulk_termination.dialog_title")).toBeTruthy();
      expect(screen.getByText("employees.bulk_termination.drag_drop_placeholder")).toBeTruthy();
      expect(screen.getByText("employees.bulk_termination.download_template_btn")).toBeTruthy();
    });

    it("does NOT render when isOpen=false", () => {
      renderDialog({ isOpen: false });
      expect(screen.queryByText("employees.bulk_termination.dialog_title")).toBeNull();
    });

    it("shows file name after file selected and shows 'choose other' button", async () => {
      renderDialog();
      await selectFile();
      expect(screen.getByText(/termination\.xlsx/)).toBeTruthy();
      expect(screen.getByText("employees.bulk_termination.choose_other_file")).toBeTruthy();
    });

    it("calls uploadTerminationImportPreview and transitions to step 2 on Continue", async () => {
      vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValueOnce(defaultPreview);
      renderDialog();
      await selectFile();

      const continueBtn = screen.getByText("employees.bulk_termination.continue_btn");
      await userEvent.click(continueBtn);

      await waitFor(() => {
        expect(employeesApi.uploadTerminationImportPreview).toHaveBeenCalledWith("ws-1", expect.any(File));
      });
      // Summary cards should appear (step 2)
      await waitFor(() => {
        expect(screen.getByText("employees.bulk_termination.total_rows")).toBeTruthy();
      });
    });

    it("shows continue button disabled when no file is selected", () => {
      renderDialog();
      const btn = screen.getByText("employees.bulk_termination.continue_btn").closest("button");
      expect(btn).toBeDisabled();
    });

    it("calls downloadTerminationTemplate on template button click", async () => {
      vi.mocked(employeesApi.downloadTerminationTemplate).mockResolvedValueOnce(undefined);
      renderDialog();
      const btn = screen.getByText("employees.bulk_termination.download_template_btn");
      await userEvent.click(btn);
      await waitFor(() => {
        expect(employeesApi.downloadTerminationTemplate).toHaveBeenCalledWith("ws-1", "en");
      });
    });
  });

  describe("Step 2 — Preview", () => {
    it("shows valid row in valid tab", async () => {
      vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValueOnce(defaultPreview);
      renderDialog();
      await selectFile();
      await userEvent.click(screen.getByText("employees.bulk_termination.continue_btn"));

      await waitFor(() => {
        expect(screen.getByText("EMP001")).toBeTruthy();
      });
    });

    it("shows invalid rows in invalid tab when invalid rows exist", async () => {
      const previewWithInvalid = {
        ...defaultPreview,
        invalid_rows: [
          { row_number: 2, employee_code: "EMP002", raw_termination_date: "bad-date", reason: "Invalid date format" },
        ],
      };
      vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValueOnce(previewWithInvalid);
      renderDialog();
      await selectFile();
      await userEvent.click(screen.getByText("employees.bulk_termination.continue_btn"));

      // Switch to invalid tab
      const invalidTab = screen.getByText((content) => content.startsWith("employees.bulk_termination.tab_invalid"));
      await userEvent.click(invalidTab);

      await waitFor(() => {
        expect(screen.getByText("Invalid date format")).toBeTruthy();
      });
    });
  });

  describe("Confirm Import", () => {
    it("calls confirmTerminationImport and shows success on immediate completion", async () => {
      vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValueOnce(defaultPreview);
      vi.mocked(employeesApi.confirmTerminationImport).mockResolvedValueOnce({
        job_id: null,
        success_count: 1,
        errors: [],
      });

      renderDialog();
      await selectFile();
      await userEvent.click(screen.getByText("employees.bulk_termination.continue_btn"));

      await waitFor(() => {
        expect(screen.getByText("employees.bulk_termination.confirm_btn")).toBeTruthy();
      });

      await userEvent.click(screen.getByText("employees.bulk_termination.confirm_btn"));

      await waitFor(() => {
        expect(employeesApi.confirmTerminationImport).toHaveBeenCalled();
      });
      await waitFor(() => {
        expect(screen.getByText("employees.bulk_termination.import_success_title")).toBeTruthy();
      });
    });

    it("shows error state when confirmTerminationImport rejects", async () => {
      vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValueOnce(defaultPreview);
      vi.mocked(employeesApi.confirmTerminationImport).mockRejectedValueOnce(new Error("Network error"));

      renderDialog();
      await selectFile();
      await userEvent.click(screen.getByText("employees.bulk_termination.continue_btn"));

      await waitFor(() => {
        expect(screen.getByText("employees.bulk_termination.confirm_btn")).toBeTruthy();
      });

      await userEvent.click(screen.getByText("employees.bulk_termination.confirm_btn"));

      await waitFor(() => {
        expect(screen.getByText("employees.bulk_termination.processing_error_title")).toBeTruthy();
      });
    });
  });

  describe("Error Handling & State Reset", () => {
    it("calls onClose and resets state when close button clicked", async () => {
      const { onClose } = renderDialog();
      const closeBtn = screen.getByText("employees.bulk_termination.cancel_btn");
      await userEvent.click(closeBtn);
      expect(onClose).toHaveBeenCalled();
    });

    it("shows back button on error and clears error on click, returning to step 2", async () => {
      vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValueOnce(defaultPreview);
      vi.mocked(employeesApi.confirmTerminationImport).mockRejectedValueOnce(new Error("fail"));
      renderDialog();
      await selectFile();
      await userEvent.click(screen.getByText("employees.bulk_termination.continue_btn"));

      await waitFor(() => {
        expect(screen.getByText("employees.bulk_termination.confirm_btn")).toBeTruthy();
      });
      await userEvent.click(screen.getByText("employees.bulk_termination.confirm_btn"));

      await waitFor(() => {
        expect(screen.getByText("employees.bulk_termination.processing_error_title")).toBeTruthy();
      });

      // Click back (clears error, returns to step 2 preview)
      const backBtn = screen.getByText("employees.bulk_termination.back_btn");
      await userEvent.click(backBtn);

      await waitFor(() => {
        expect(screen.getByText("employees.bulk_termination.total_rows")).toBeTruthy();
      });
    });

    it("returns to step 1 from step 2 via back button", async () => {
      vi.mocked(employeesApi.uploadTerminationImportPreview).mockResolvedValueOnce(defaultPreview);
      renderDialog();
      await selectFile();
      await userEvent.click(screen.getByText("employees.bulk_termination.continue_btn"));

      await waitFor(() => {
        expect(screen.getByText("employees.bulk_termination.total_rows")).toBeTruthy();
      });

      await userEvent.click(screen.getByText("employees.bulk_termination.cancel_btn"));

      await waitFor(() => {
        expect(screen.getByText("employees.bulk_termination.download_template_btn")).toBeTruthy();
      });
    });
  });
});
