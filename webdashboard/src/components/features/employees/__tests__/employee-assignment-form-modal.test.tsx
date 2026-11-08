import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import EmployeeAssignmentFormModal from "../employee-assignment-form-modal";

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => options?.defaultValue || key,
  }),
}));

const mockLocations = [
  { id: "loc-1", name: "Hanoi HQ", is_headquarters: true, timezone: "Asia/Bangkok" },
  { id: "loc-2", name: "Da Nang Branch", is_headquarters: false, timezone: "Asia/Bangkok" },
];

vi.mock("@/pages/settings/hooks/use-locations", () => ({
  useLocations: () => ({
    data: mockLocations,
    isLoading: false,
  }),
}));

const mockEmployees = [
  { id: "emp-1", full_name: "Nguyen Van A", employee_id_number: "EMP-001" },
  { id: "emp-2", full_name: "Tran Thi B", employee_id_number: "EMP-002" },
];

vi.mock("@/hooks/use-employees", () => ({
  useAllEmployees: () => ({
    data: mockEmployees,
    isLoading: false,
  }),
}));

const mockCreateMutateAsync = vi.fn();
const mockUpdateMutateAsync = vi.fn();

vi.mock("@/hooks/use-employee-assignments", () => ({
  useCreateEmployeeAssignment: () => ({
    mutateAsync: mockCreateMutateAsync,
  }),
  useUpdateEmployeeAssignment: () => ({
    mutateAsync: mockUpdateMutateAsync,
  }),
}));

function renderModal(props: React.ComponentProps<typeof EmployeeAssignmentFormModal>) {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <EmployeeAssignmentFormModal {...props} />
      </ToastProvider>
    </QueryClientProvider>
  );
}

describe("EmployeeAssignmentFormModal (Dual Mode)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("does not render employee selector when employeeId is fixed (Tab Mode)", () => {
    renderModal({
      isOpen: true,
      onClose: vi.fn(),
      editingAssignment: null,employeeId: "emp-1",
    });

    expect(screen.getByText("Tạo Phái cử Mới")).toBeInTheDocument();
    expect(screen.queryByTestId("assignment-employee-combobox")).not.toBeInTheDocument();
  });

  it("renders employee selector when employeeId is omitted (Registry Mode)", () => {
    renderModal({
      isOpen: true,
      onClose: vi.fn(),
      editingAssignment: null,employeeId: undefined,
    });

    expect(screen.getByText("Tạo Phái cử Mới")).toBeInTheDocument();
    expect(screen.getByTestId("assignment-employee-combobox")).toBeInTheDocument();
  });

  it("renders employee name read-only when editing in Registry Mode", () => {
    renderModal({
      isOpen: true,
      onClose: vi.fn(),
      editingAssignment: {
        id: "assign-1",
        workspace_id: "ws-1",
        employment_id: "em-1",
        home_location_id: "loc-1",
        host_location_id: "loc-2",
        start_date: "2026-01-01",
        end_date: "2026-06-30",
        reason: "Transfer",
        status: "active",
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
        employee_id: "emp-1",
        employee_name: "Nguyen Van A",
        employee_code: "EMP-001",
      },employeeId: undefined,
    });

    expect(screen.getByText("Chỉnh sửa Phái cử")).toBeInTheDocument();
    expect(screen.getByText(/Nguyen Van A/)).toBeInTheDocument();
    expect(screen.getByText(/EMP-001/)).toBeInTheDocument();
  });
});
