import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import { fetchEmployers } from "@/lib/api/employers";
import { useEmployeeContracts } from "@/pages/employees/hooks/use-employee-contracts";

import EmployeeContractsTab from "../employee-contracts-tab";

// Mock dependent hooks and API calls
vi.mock("@/pages/employees/hooks/use-employee-contracts", () => ({
  useEmployeeContracts: vi.fn(() => ({
    contracts: [],
    isLoading: false,
    isError: false,
    createContract: { mutate: vi.fn(), isPending: false },
    updateContract: { mutate: vi.fn(), isPending: false },
    deleteContract: { mutate: vi.fn(), isPending: false },
  })),
}));

vi.mock("@/lib/api/employers", () => ({
  fetchEmployers: vi.fn(),
}));

vi.mock("@/hooks/use-permissions", () => ({
  useMyPermissions: () => ({
    hasPermission: () => true,
    hasAnyPermission: () => true,
    hasScopedPermission: () => true,
    isLoading: false,
  }),
}));

const mockContracts = [
  {
    id: "contract-1",
    workspace_id: "ws-1",
    employment_id: "emp-1",
    contract_number: "HD-001",
    contract_type: "fixed_term" as const,
    status: "active" as const,
    start_date: "2025-01-01",
    end_date: "2025-12-31",
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
  },
  {
    id: "contract-2",
    workspace_id: "ws-1",
    employment_id: "emp-1",
    contract_number: "HD-002",
    contract_type: "probation" as const,
    status: "expired" as const,
    start_date: "2024-01-01",
    end_date: "2024-03-01",
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
  },
  {
    id: "contract-3",
    workspace_id: "ws-1",
    employment_id: "emp-1",
    contract_number: "HD-003",
    contract_type: "training" as const,
    status: "terminated" as const,
    start_date: "2024-04-01",
    end_date: "2024-06-01",
    created_at: "2024-04-01T00:00:00Z",
    updated_at: "2024-04-01T00:00:00Z",
  },
  {
    id: "contract-4",
    workspace_id: "ws-1",
    employment_id: "emp-1",
    contract_number: "HD-004",
    contract_type: "indefinite" as const,
    status: "draft" as const,
    start_date: "2025-07-01",
    end_date: null,
    created_at: "2025-07-01T00:00:00Z",
    updated_at: "2025-07-01T00:00:00Z",
  },
  {
    id: "contract-5",
    workspace_id: "ws-1",
    employment_id: "emp-1",
    contract_number: "HD-005",
    contract_type: "unknown-type" as any,
    status: "unknown-status" as any,
    start_date: "2025-08-01",
    end_date: null,
    created_at: "2025-08-01T00:00:00Z",
    updated_at: "2025-08-01T00:00:00Z",
  },
];

const mockEmployers = [
  { id: "employer-1", name: "Mock Employer" }
];

describe("EmployeeContractsTab Component", () => {
  let queryClient: QueryClient;

  const defaultProps = {
    employeeId: "emp-1",formatDate: (date: string) => date,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    (useEmployeeContracts as Mock).mockReturnValue({
      contracts: [],
      isLoading: false,
      isError: false,
      createContract: { mutate: vi.fn(), isPending: false },
      updateContract: { mutate: vi.fn(), isPending: false },
      deleteContract: { mutate: vi.fn(), isPending: false },
    });

    (fetchEmployers as Mock).mockResolvedValue(mockEmployers);
  });

  const renderTab = () => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <EmployeeContractsTab {...defaultProps} />
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders skeleton loader when isLoading is true", () => {
    (useEmployeeContracts as Mock).mockReturnValue({
      contracts: [],
      isLoading: true,
      isError: false,
      createContract: { mutate: vi.fn(), isPending: false },
      updateContract: { mutate: vi.fn(), isPending: false },
      deleteContract: { mutate: vi.fn(), isPending: false },
    });

    const { container } = renderTab();
    expect(container.querySelector(".animate-pulse")).toBeInTheDocument();
  });

  it("renders empty state when no contracts exist", () => {
    renderTab();
    expect(screen.getByText(/Chưa có hợp đồng/i)).toBeInTheDocument();
  });

  it("renders contracts list with contract metadata", async () => {
    (useEmployeeContracts as Mock).mockReturnValue({
      contracts: mockContracts,
      isLoading: false,
      isError: false,
      createContract: { mutate: vi.fn(), isPending: false },
      updateContract: { mutate: vi.fn(), isPending: false },
      deleteContract: { mutate: vi.fn(), isPending: false },
    });

    renderTab();

    // Verify contract metadata renders
    expect(screen.getByText("HD-001", { exact: false })).toBeInTheDocument();

    // e-sign badges are disabled until backend is wired
    expect(screen.queryByText(/e-Sign:/i)).toBeNull();
  });

  it("opens edit modal and triggers delete mutation when clicked", async () => {
    const mockDelete = vi.fn();
    (useEmployeeContracts as Mock).mockReturnValue({
      contracts: mockContracts,
      isLoading: false,
      isError: false,
      createContract: { mutate: vi.fn(), isPending: false },
      updateContract: { mutate: vi.fn(), isPending: false },
      deleteContract: { mutate: mockDelete, isPending: false },
    });

    const { container } = renderTab();

    // The component renders action buttons in group-hover. Let's find the Pencil and Trash2 buttons.
    // In JSDOM, since they are lucide React components, we can query by selector:
    const editBtn = container.querySelector("button:has(.lucide-pencil)") || container.querySelectorAll("button")[1];
    expect(editBtn).toBeDefined();
    if (editBtn) {
      fireEvent.click(editBtn);
      // Wait for Modal to be open in edit mode (will show "Chỉnh sửa Hợp đồng")
      await waitFor(() => {
        expect(screen.getByText("Chỉnh sửa Hợp đồng")).toBeDefined();
      });
      // Click HỦY BỎ to close the modal
      const cancelBtn = screen.getByText("HỦY BỎ");
      fireEvent.click(cancelBtn);
    }

    const deleteBtn = container.querySelector("button:has(.lucide-trash2)") || container.querySelectorAll("button")[2];
    expect(deleteBtn).toBeDefined();
    if (deleteBtn) {
      fireEvent.click(deleteBtn);
      expect(mockDelete).toHaveBeenCalledWith("contract-1");
    }
  });

  it("opens create modal when clicking sign contract button in header", async () => {
    (useEmployeeContracts as Mock).mockReturnValue({
      contracts: [],
      isLoading: false,
      isError: false,
      createContract: { mutate: vi.fn(), isPending: false },
      updateContract: { mutate: vi.fn(), isPending: false },
      deleteContract: { mutate: vi.fn(), isPending: false },
    });

    renderTab();

    const signContractBtn = screen.getByText(/KÝ HỢP ĐỒNG/i);
    expect(signContractBtn).toBeDefined();
    fireEvent.click(signContractBtn);

    // Form modal should open showing "Tạo Hợp đồng Mới"
    await waitFor(() => {
      expect(screen.getByText("Tạo Hợp đồng Mới")).toBeDefined();
    });
  });
});
