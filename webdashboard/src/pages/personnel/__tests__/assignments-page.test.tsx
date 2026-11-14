import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import type { EmployeeAssignmentWithEmployee } from "@/lib/api/employee-assignments";

import Button from "@/components/ui/button";
import { ToastProvider } from "@/components/ui/toast";

import PersonnelAssignmentsPage from "../assignments-page";

// Mock i18next translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => options?.defaultValue || key,
  }),
}));


// Mock TanStack Router
let mockSearch: { q?: string; employeeId?: string } = { q: "" };
vi.mock("@tanstack/react-router", () => ({
  useSearch: () => mockSearch,
  Link: ({ to, children, ...props }: { to?: string; children: React.ReactNode }) => (
    <a href={to} {...props}>
      {children}
    </a>
  ),
}));

// Mock useDateTimeFormatter
vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatDate: (date: Date | string) => {
      const d = typeof date === "string" ? new Date(date) : date;
      const day = String(d.getDate()).padStart(2, "0");
      const month = String(d.getMonth() + 1).padStart(2, "0");
      const year = d.getFullYear();
      return `${day}/${month}/${year}`;
    },
  }),
}));

const mockAssignments: EmployeeAssignmentWithEmployee[] = [
  {
    id: "assign-1",
    workspace_id: "ws-1",
    employment_id: "emp-rec-1",
    employee_id: "emp-1",
    employee_name: "Nguyen Van A",
    employee_code: "EMP-001",
    home_location_id: "loc-1",
    host_location_id: "loc-2",
    home_location_name: "Hanoi HQ",
    host_location_name: "Da Nang Branch",
    start_date: "2026-01-01",
    end_date: "2026-06-30",
    reason: "Onsite project support",
    status: "active",
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "assign-2",
    workspace_id: "ws-1",
    employment_id: "emp-rec-2",
    employee_id: "emp-2",
    employee_name: "Tran Thi B",
    employee_code: "EMP-002",
    home_location_id: "loc-1",
    host_location_id: "loc-3",
    home_location_name: "Hanoi HQ",
    host_location_name: "Saigon Office",
    start_date: "2026-07-01",
    end_date: null,
    reason: "Inter-branch transfer",
    status: "scheduled",
    created_at: "2026-02-01T00:00:00Z",
    updated_at: "2026-02-01T00:00:00Z",
  },
];

const mockDeleteMutateAsync = vi.fn();

vi.mock("@/hooks/use-employee-assignments", () => ({
  useAssignments: () => ({
    data: mockAssignments,
    isLoading: false,
  }),
  useDeleteEmployeeAssignment: () => ({
    mutateAsync: mockDeleteMutateAsync,
  }),
}));

const mockConfirm = vi.fn();
vi.mock("@/hooks/use-confirm", () => ({
  useConfirm: () => ({
    confirm: mockConfirm,
    ConfirmationDialog: () => <div data-testid="mock-confirm-dialog" />,
  }),
}));

// Mock EmployeeAssignmentFormModal
vi.mock("@/components/features/employees/employee-assignment-form-modal", () => ({
  default: ({
    isOpen,
    onClose,
    editingAssignment,
  }: {
    isOpen: boolean;
    onClose: () => void;
    editingAssignment: EmployeeAssignmentWithEmployee | null;
  }) => {
    if (!isOpen) return null;
    return (
      <div data-testid="mock-assignment-form-modal">
        <div>Modal Opened</div>
        {editingAssignment ? (
          <div>Editing: {editingAssignment.employee_name}</div>
        ) : (
          <div>Creating New</div>
        )}
        <Button onClick={onClose} type="button">
          Close Form
        </Button>
      </div>
    );
  },
}));

function renderPage() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        <PersonnelAssignmentsPage />
      </ToastProvider>
    </QueryClientProvider>
  );
}

describe("PersonnelAssignmentsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSearch = { q: "" };
  });

  it("renders header and metrics correctly", () => {
    renderPage();
    expect(screen.getByText("Quản lý Phái cử & Luân chuyển")).toBeInTheDocument();
    // Total is 2, active 1, scheduled 1
    expect(screen.getByText("Tổng đợt phái cử")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.getByText("Đang hiệu lực")).toBeInTheDocument();
    expect(screen.getByText("Đã lên lịch")).toBeInTheDocument();
  });

  it("renders assignments in the table", () => {
    renderPage();
    expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
    expect(screen.getAllByText("EMP-001").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Da Nang Branch").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Tran Thi B").length).toBeGreaterThan(0);
  });

  it("filters assignments by employee URL filter", () => {
    mockSearch = { employeeId: "emp-1" };
    renderPage();

    expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
    expect(screen.queryByText("Tran Thi B")).not.toBeInTheDocument();
    expect(screen.getByTestId("employee-filter-chip")).toBeInTheDocument();
  });

  it("renders a clear link for employee URL filter", () => {
    mockSearch = { employeeId: "emp-1" };
    renderPage();

    expect(screen.getByRole("link", { name: "Bỏ bộ lọc" })).toHaveAttribute("href", "/assignments");
  });

  it("filters assignments by search input", async () => {
    renderPage();
    const searchInput = screen.getByRole("textbox", { name: "Tìm kiếm phái cử" });
    fireEvent.change(searchInput, { target: { value: "Nguyen" } });

    expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
    expect(screen.queryByText("Tran Thi B")).not.toBeInTheDocument();
  });

  it("opens create modal when clicking create button", () => {
    renderPage();
    const createButton = screen.getByRole("button", { name: "TẠO PHÁI CỬ MỚI" });
    fireEvent.click(createButton);

    expect(screen.getByTestId("mock-assignment-form-modal")).toBeInTheDocument();
    expect(screen.getByText("Creating New")).toBeInTheDocument();
  });

  it("opens edit modal when clicking edit action", () => {
    renderPage();
    const editButtons = screen.getAllByRole("button", { name: "Chỉnh sửa" });
    fireEvent.click(editButtons[0]);

    expect(screen.getByTestId("mock-assignment-form-modal")).toBeInTheDocument();
    expect(screen.getByText("Editing: Nguyen Van A")).toBeInTheDocument();
  });

  it("triggers delete confirmation and mutation", async () => {
    mockConfirm.mockResolvedValueOnce(true);
    mockDeleteMutateAsync.mockResolvedValueOnce(undefined);

    renderPage();
    const deleteButtons = screen.getAllByRole("button", { name: "Xóa" });
    fireEvent.click(deleteButtons[0]);

    await waitFor(() => {
      expect(mockConfirm).toHaveBeenCalled();
      expect(mockDeleteMutateAsync).toHaveBeenCalledWith({
        employeeId: "emp-1",
        assignmentId: "assign-1",
      });
    });
  });
});
