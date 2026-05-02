import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";

import { BulkUploadOtDialog } from "../bulk-upload-ot-dialog";


// Mock the hook
vi.mock("@/hooks/use-attendance", () => ({
  useBulkUploadPaperOt: () => ({
    bulkUploadPaperOt: vi.fn().mockResolvedValue({
      total_rows: 2,
      success_count: 1,
      failed_count: 1,
      errors: [{ line_number: 2, employee_id: "E002", error_message: "Error" }]
    }),
    isUploading: false
  })
}));

// Mock translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
 
    t: (key: string, _options?: unknown) => {
      if (key === "approvals.bulk_upload.confirm_close_with_errors") return "Close with errors?";
      if (key === "common.warning") return "Warning";
      if (key === "common.confirm") return "Confirm";
      if (key === "common.cancel") return "Cancel";
      if (key === "common.close") return "Close";
      return key;
    }
  })
}));

 
const renderComponent = (props: { open: boolean; onOpenChange: (open: boolean) => void }) => {
  return render(
    <ToastProvider>
      <BulkUploadOtDialog {...props} />
    </ToastProvider>
  );
};

describe("BulkUploadOtDialog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should show useConfirm dialog when closing with errors", async () => {
    const user = userEvent.setup();
    const onOpenChange = vi.fn();
    
    renderComponent({
      open: true,
      onOpenChange,
    });

    // We need to simulate uploading a file so it gets to the result state with errors
    const file = new File(["test"], "test.csv", { type: "text/csv" });
    const input = screen.getByLabelText(/Select File|approvals.bulk_upload.select_file/i);
    await user.upload(input, file);

    const uploadButton = screen.getByRole("button", { name: /Upload & Process|approvals.bulk_upload.upload/i });
    await user.click(uploadButton);

    // Wait for result to appear
    expect(await screen.findByText("Error")).toBeDefined();

    // Now click the dialog's top-right X button which calls handleClose
    // We can find it by getting the first button (it's the X button at the top)
    const buttons = screen.getAllByRole("button");
    const xButton = buttons[0]; 
    await user.click(xButton);

    // Confirm dialog should appear
    expect(await screen.findByText("Warning")).toBeDefined();
    expect(await screen.findByText("Close with errors?")).toBeDefined();

    // Click confirm
    const confirmButton = screen.getByRole("button", { name: "Confirm" });
    fireEvent.click(confirmButton);

    await waitFor(() => {
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });
});
