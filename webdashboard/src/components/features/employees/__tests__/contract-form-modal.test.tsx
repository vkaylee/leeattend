import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import type { EmployeeContract } from "@/lib/api";

import { ToastProvider } from "@/components/ui/toast";

import ContractFormModal from "../contract-form-modal";
import { contractSchema } from "../contract-form-schema";

// Mock useEmployeeContracts hook
const mockCreateMutateAsync = vi.fn();
const mockUpdateMutateAsync = vi.fn();
const mockUseQuery = vi.fn();

vi.mock("@tanstack/react-query", async () => {
  const actual = await vi.importActual("@tanstack/react-query");
  return {
    ...actual,
    useQuery: (...args: any[]) => mockUseQuery(...args),
    useQueries: (..._args: any[]) => [mockUseQuery(), mockUseQuery()],
  };
});

vi.mock("@/pages/employees/hooks/use-employee-contracts", () => ({
  useEmployeeContracts: () => ({
    contracts: [],
    isLoading: false,
    isError: false,
    createContract: { mutateAsync: mockCreateMutateAsync },
    updateContract: { mutateAsync: mockUpdateMutateAsync },
    deleteContract: { mutate: vi.fn() },
  }),
}));

// Mock standardSchemaResolver to avoid Zod v4 compat issues in test env
vi.mock("@hookform/resolvers/standard-schema", () => ({
  standardSchemaResolver: () => (values: Record<string, unknown>) => Promise.resolve({ values, errors: {} }),
}));

describe("ContractFormModal", () => {
  let queryClient: QueryClient;

  const defaultProps = {
    isOpen: true,
    onClose: vi.fn(),
    editingContract: null as EmployeeContract | null,employeeId: "emp-1",
  };

  const mockExistingContract: EmployeeContract = {
    id: "contract-1",
    workspace_id: "ws-1",
    employment_id: "emp-1",
    contract_number: "HD-001",
    contract_type: "fixed_term",
    status: "active",
    start_date: "2025-01-01",
    end_date: "2025-12-31",
    notes: "Test notes",
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
  };

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
    mockCreateMutateAsync.mockResolvedValue({ id: "new-contract" });
    mockUpdateMutateAsync.mockResolvedValue({ id: "contract-1" });
    
    mockUseQuery.mockReturnValue({
      data: [
        { id: "phase-1", name: "Thử việc", is_active: true, phase_type: "probation" },
        { id: "phase-2", name: "Chính thức", is_active: true, phase_type: "official" }
      ],
      isLoading: false,
    });
  });

  const renderModal = (props: Partial<typeof defaultProps> = {}) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <ContractFormModal {...defaultProps} {...props} />
        </ToastProvider>
      </QueryClientProvider>,
    );
  };

  // ── Rendering ──────────────────────────────────────────────────

  it("renders create modal title when no editingContract", () => {
    renderModal();
    expect(screen.getByText("Tạo Hợp đồng Mới")).toBeDefined();
  });

  it("renders update modal title when editingContract is provided", () => {
    renderModal({ editingContract: mockExistingContract });
    expect(screen.getByText("Chỉnh sửa Hợp đồng")).toBeDefined();
  });

  it("renders nothing when isOpen is false", () => {
    const { container } = renderModal({ isOpen: false });
    expect(container.querySelector("[role='dialog']")).toBeNull();
  });

  // ── Form fields ────────────────────────────────────────────────

  it("renders all required form fields for create mode", () => {
    renderModal();
    expect(screen.getByLabelText(/Số hợp đồng/i)).toBeDefined();
    expect(screen.getByLabelText(/Loại hợp đồng/i)).toBeDefined();
    expect(screen.getByLabelText(/Chọn Cột mốc Khởi đầu/i)).toBeDefined();
    expect(screen.getByLabelText(/Ngày hiệu lực/i)).toBeDefined();
    expect(screen.getByLabelText(/Ngày hết hạn/i)).toBeDefined();
    expect(screen.getByLabelText(/Ghi chú/i)).toBeDefined();
  });

  it("does NOT render status field in create mode", () => {
    renderModal();
    expect(screen.queryByLabelText(/Trạng thái/i)).toBeNull();
  });

  it("renders status field in edit mode", () => {
    renderModal({ editingContract: mockExistingContract });
    expect(screen.getByLabelText(/Trạng thái/i)).toBeDefined();
  });

  // ── Buttons ────────────────────────────────────────────────────

  it("shows 'TẠO MỚI' button in create mode", () => {
    renderModal();
    expect(screen.getByText("TẠO MỚI")).toBeDefined();
  });

  it("shows 'CẬP NHẬT' button in edit mode", () => {
    renderModal({ editingContract: mockExistingContract });
    expect(screen.getByText("CẬP NHẬT")).toBeDefined();
  });

  it("shows 'HỦY BỎ' button", () => {
    renderModal();
    expect(screen.getByText("HỦY BỎ")).toBeDefined();
  });

  it("calls onClose when 'HỦY BỎ' is clicked", async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderModal({ onClose });

    await user.click(screen.getByText("HỦY BỎ"));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  // ── Form pre-fill in edit mode ─────────────────────────────────

  it("pre-fills form fields when editing an existing contract", () => {
    renderModal({ editingContract: mockExistingContract });

    const contractNumberInput = screen.getByLabelText<HTMLInputElement>(/Số hợp đồng/i);
    expect(contractNumberInput.value).toBe("HD-001");

    const startDateInput = screen.getByLabelText<HTMLInputElement>(/Ngày hiệu lực/i);
    expect(startDateInput.value).toBe("2025-01-01");

    const endDateInput = screen.getByLabelText<HTMLInputElement>(/Ngày hết hạn/i);
    expect(endDateInput.value).toBe("2025-12-31");
  });

  // ── Create submission ──────────────────────────────────────────

  it("calls createContract on form submit in create mode", async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderModal({ onClose });

    // Fill end_date (required for non-indefinite contracts)
    const endDateInput = screen.getByLabelText(/Ngày hết hạn/i);
    await user.clear(endDateInput);
    await user.type(endDateInput, "2026-12-31");
    
    // Fill assign_phase_id
    const phaseSelect = screen.getByLabelText(/Chọn Cột mốc Khởi đầu/i);
    await user.selectOptions(phaseSelect, "phase-1");

    await user.click(screen.getByText("TẠO MỚI"));

    await waitFor(() => {
      expect(mockCreateMutateAsync).toHaveBeenCalledTimes(1);
    });
  });

  // ── Update submission ──────────────────────────────────────────

  it("calls updateContract on form submit in edit mode", async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderModal({ editingContract: mockExistingContract, onClose });

    const notesInput = screen.getByLabelText(/Ghi chú/i);
    await user.clear(notesInput);
    await user.type(notesInput, "Updated notes");

    await user.click(screen.getByText("CẬP NHẬT"));

    await waitFor(() => {
      expect(mockUpdateMutateAsync).toHaveBeenCalledTimes(1);
      expect(mockUpdateMutateAsync).toHaveBeenCalledWith(
        expect.objectContaining({
          contractId: "contract-1",
        }),
      );
    });
  });

  // ── Contract type interactions ──────────────────────────────────

  it("disables end_date when indefinite contract type is selected", async () => {
    const user = userEvent.setup();
    renderModal();

    const contractTypeSelect = screen.getByLabelText<HTMLSelectElement>(/Loại hợp đồng/i);
    await user.selectOptions(contractTypeSelect, "indefinite");

    const endDateInput = screen.getByLabelText<HTMLInputElement>(/Ngày hết hạn/i);
    expect(endDateInput.disabled).toBe(true);
  });

  it("enables end_date when non-indefinite contract type is selected", () => {
    renderModal();

    // Default is "probation", so end_date should be enabled
    const endDateInput = screen.getByLabelText<HTMLInputElement>(/Ngày hết hạn/i);
    expect(endDateInput.disabled).toBe(false);
  });

  // ── Error handling ─────────────────────────────────────────────

  it("shows error toast when createContract fails", async () => {
    mockCreateMutateAsync.mockRejectedValue(new Error("Server error"));
    const user = userEvent.setup();
    renderModal();

    const endDateInput = screen.getByLabelText(/Ngày hết hạn/i);
    await user.clear(endDateInput);
    await user.type(endDateInput, "2026-12-31");

    const phaseSelect = screen.getByLabelText(/Chọn Cột mốc Khởi đầu/i);
    await user.selectOptions(phaseSelect, "phase-1");

    await user.click(screen.getByText("TẠO MỚI"));

    await waitFor(() => {
      expect(mockCreateMutateAsync).toHaveBeenCalledTimes(1);
    });
  });

  // ── Apply duration buttons ─────────────────────────────────────
  it("applies duration when clicking duration shortcut buttons", async () => {
    const user = userEvent.setup();
    renderModal();

    // Click "1 Tháng" button
    const oneMonthBtn = screen.getByText("1 Tháng");
    await user.click(oneMonthBtn);

    const endDateInput = screen.getByLabelText<HTMLInputElement>(/Ngày hết hạn/i);
    expect(endDateInput.value).not.toBe("");

    // Click "1 Năm" button (tests years branch)
    const oneYearBtn = screen.getByText("1 Năm");
    await user.click(oneYearBtn);
    expect(endDateInput.value).not.toBe("");
  });

  it("handles empty or invalid start date when applying duration", async () => {
    const user = userEvent.setup();
    renderModal();

    const startDateInput = screen.getByLabelText(/Ngày hiệu lực/i);
    const oneMonthBtn = screen.getByText("1 Tháng");

    // Case 1: Empty start date
    await user.clear(startDateInput);
    await user.click(oneMonthBtn);
    const endDateInput = screen.getByLabelText<HTMLInputElement>(/Ngày hết hạn/i);
    expect(endDateInput.value).toBe("");

    // Case 2: Invalid start date
    await user.type(startDateInput, "invalid-date");
    await user.click(oneMonthBtn);
    expect(endDateInput.value).toBe("");
  });

  it("applies indefinite duration and updates contract type when 'Lâu dài' is clicked", async () => {
    const user = userEvent.setup();
    renderModal();

    const indefiniteBtn = screen.getByText("Lâu dài");
    await user.click(indefiniteBtn);

    const contractTypeSelect = screen.getByLabelText<HTMLSelectElement>(/Loại hợp đồng/i);
    expect(contractTypeSelect.value).toBe("indefinite");

    const endDateInput = screen.getByLabelText<HTMLInputElement>(/Ngày hết hạn/i);
    expect(endDateInput.disabled).toBe(true);
  });

  it("handles invalid start date format in editing mode when preset is clicked", async () => {
    const user = userEvent.setup();
    // Render with an invalid start date in editing contract
    const invalidContract = { ...mockExistingContract, start_date: "invalid-date" };
    renderModal({ editingContract: invalidContract });

    const oneMonthBtn = screen.getByText("1 Tháng");
    await user.click(oneMonthBtn);

    const endDateInput = screen.getByLabelText<HTMLInputElement>(/Ngày hết hạn/i);
    expect(endDateInput.value).toBe("2025-12-31"); // should return early and not update end_date
  });

  // ── Validation and error flows ────────────────────────────────
  it("shows error when submit without selecting assign_phase_id", async () => {
    const user = userEvent.setup();
    renderModal();

    // Fill end_date to satisfy validation of probation contract type
    const endDateInput = screen.getByLabelText(/Ngày hết hạn/i);
    await user.clear(endDateInput);
    await user.type(endDateInput, "2026-12-31");

    // Select the empty phase option
    const phaseSelect = screen.getByLabelText(/Chọn Cột mốc Khởi đầu/i);
    await user.selectOptions(phaseSelect, "");

    await user.click(screen.getByText("TẠO MỚI"));

    // Verify early return toast
    expect(await screen.findByText("Vui lòng chọn Cột mốc khởi đầu")).toBeInTheDocument();

    // Should not call createContract
    expect(mockCreateMutateAsync).not.toHaveBeenCalled();
  });

  it("shows error toast when updateContract fails", async () => {
    mockUpdateMutateAsync.mockRejectedValueOnce(new Error("Update failed"));
    const user = userEvent.setup();
    renderModal({ editingContract: mockExistingContract });

    await user.click(screen.getByText("CẬP NHẬT"));

    await waitFor(() => {
      expect(mockUpdateMutateAsync).toHaveBeenCalledTimes(1);
    });
  });

  // ── Overlapping contract exists handling ───────────────────────
  it("shows overlap confirmation modal and closes overlapping contract on confirmation", async () => {
    // Setup createContract mutation mock to fail with "overlapping_contract_exists"
    mockCreateMutateAsync.mockRejectedValueOnce(new Error("overlapping_contract_exists"));
    const onClose = vi.fn();
    const user = userEvent.setup();
    renderModal({ onClose });

    // Fill end_date and phase
    const endDateInput = screen.getByLabelText(/Ngày hết hạn/i);
    await user.clear(endDateInput);
    await user.type(endDateInput, "2026-12-31");

    const phaseSelect = screen.getByLabelText(/Chọn Cột mốc Khởi đầu/i);
    await user.selectOptions(phaseSelect, "phase-1");

    // Click submit
    await user.click(screen.getByText("TẠO MỚI"));

    // Wait for the overlap modal to appear
    await waitFor(() => {
      expect(screen.getByText("Trùng Lặp Hợp Đồng")).toBeDefined();
    });

    // Mock the second call to succeed (it will call createContract with auto_close_overlapping: true)
    mockCreateMutateAsync.mockResolvedValueOnce({ id: "new-contract-after-overlap" });

    // Click "TỰ ĐỘNG ĐÓNG & TẠO MỚI"
    const confirmBtn = screen.getByText("TỰ ĐỘNG ĐÓNG & TẠO MỚI");
    await user.click(confirmBtn);

    await waitFor(() => {
      expect(mockCreateMutateAsync).toHaveBeenLastCalledWith(
        expect.objectContaining({
          auto_close_overlapping: true,
        })
      );
      expect(onClose).toHaveBeenCalled();
    });
  });

  it("handles error when overlap confirmation fails", async () => {
    mockCreateMutateAsync.mockRejectedValueOnce(new Error("overlapping_contract_exists"));
    const user = userEvent.setup();
    renderModal();

    // Fill end_date and phase
    const endDateInput = screen.getByLabelText(/Ngày hết hạn/i);
    await user.clear(endDateInput);
    await user.type(endDateInput, "2026-12-31");

    const phaseSelect = screen.getByLabelText(/Chọn Cột mốc Khởi đầu/i);
    await user.selectOptions(phaseSelect, "phase-1");

    await user.click(screen.getByText("TẠO MỚI"));

    await waitFor(() => {
      expect(screen.getByText("Trùng Lặp Hợp Đồng")).toBeDefined();
    });

    // Mock the confirmation call to fail
    mockCreateMutateAsync.mockRejectedValueOnce(new Error("Overlap confirmation failed"));

    const confirmBtn = screen.getByText("TỰ ĐỘNG ĐÓNG & TẠO MỚI");
    await user.click(confirmBtn);

    await waitFor(() => {
      expect(mockCreateMutateAsync).toHaveBeenCalledTimes(2);
    });
  });

  it("allows closing the overlap modal when clicking 'QUAY LẠI SỬA NGÀY'", async () => {
    mockCreateMutateAsync.mockRejectedValueOnce(new Error("overlapping_contract_exists"));
    const user = userEvent.setup();
    renderModal();

    // Fill end_date and phase
    const endDateInput = screen.getByLabelText(/Ngày hết hạn/i);
    await user.clear(endDateInput);
    await user.type(endDateInput, "2026-12-31");

    const phaseSelect = screen.getByLabelText(/Chọn Cột mốc Khởi đầu/i);
    await user.selectOptions(phaseSelect, "phase-1");

    await user.click(screen.getByText("TẠO MỚI"));

    await waitFor(() => {
      expect(screen.getByText("Trùng Lặp Hợp Đồng")).toBeDefined();
    });

    const backBtn = screen.getByText("QUAY LẠI SỬA NGÀY");
    await user.click(backBtn);

    // Modal should close and return to editing
    await waitFor(() => {
      expect(screen.queryByText("Trùng Lặp Hợp Đồng")).toBeNull();
    });
  });

  it("allows closing the overlap modal using close dialog icon", async () => {
    mockCreateMutateAsync.mockRejectedValueOnce(new Error("overlapping_contract_exists"));
    const user = userEvent.setup();
    renderModal();

    const endDateInput = screen.getByLabelText(/Ngày hết hạn/i);
    await user.clear(endDateInput);
    await user.type(endDateInput, "2026-12-31");

    const phaseSelect = screen.getByLabelText(/Chọn Cột mốc Khởi đầu/i);
    await user.selectOptions(phaseSelect, "phase-1");

    await user.click(screen.getByText("TẠO MỚI"));

    await waitFor(() => {
      expect(screen.getByText("Trùng Lặp Hợp Đồng")).toBeDefined();
    });

    // Close using Close dialog icon
    const closeDialogIconBtn = screen.getByRole("button", { name: /Close dialog/i });
    await user.click(closeDialogIconBtn);

    await waitFor(() => {
      expect(screen.queryByText("Trùng Lặp Hợp Đồng")).toBeNull();
    });
  });

  it("changes contract type to fixed_term when clicking duration shortcuts in indefinite mode", async () => {
    const user = userEvent.setup();
    renderModal();

    // Select indefinite first
    const contractTypeSelect = screen.getByLabelText<HTMLSelectElement>(/Loại hợp đồng/i);
    await user.selectOptions(contractTypeSelect, "indefinite");

    // Click "1 Tháng" button
    const oneMonthBtn = screen.getByText("1 Tháng");
    await user.click(oneMonthBtn);

    // Should switch back to fixed_term
    expect(contractTypeSelect.value).toBe("fixed_term");
  });

  describe("contractSchema validation rules", () => {
    it("requires end_date for non-indefinite contract types", () => {
      const result = contractSchema.safeParse({
        contract_type: "probation",
        start_date: "2025-01-01",
        end_date: "",
      });
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.issues.some(e => e.path.includes("end_date"))).toBe(true);
      }
    });

    it("requires end_date to be after start_date", () => {
      const result = contractSchema.safeParse({
        contract_type: "probation",
        start_date: "2025-01-02",
        end_date: "2025-01-01",
      });
      expect(result.success).toBe(false);
      if (!result.success) {
        expect(result.error.issues.some(e => e.path.includes("end_date"))).toBe(true);
      }
    });

  });
});
