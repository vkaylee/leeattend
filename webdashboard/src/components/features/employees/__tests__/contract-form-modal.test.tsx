import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import type { EmployeeContract } from "@/lib/api";

import { ToastProvider } from "@/components/ui/toast";

import ContractFormModal from "../contract-form-modal";

// Mock useEmployeeContracts hook
const mockCreateMutateAsync = vi.fn();
const mockUpdateMutateAsync = vi.fn();
const mockUseQuery = vi.fn();

vi.mock("@tanstack/react-query", async () => {
  const actual = await vi.importActual("@tanstack/react-query");
  return {
    ...actual,
    useQuery: (...args: any[]) => mockUseQuery(...args),
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
    editingContract: null as EmployeeContract | null,
    wsId: "ws-1",
    employeeId: "emp-1",
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
});
