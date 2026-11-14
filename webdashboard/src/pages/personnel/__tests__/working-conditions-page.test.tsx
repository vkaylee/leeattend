import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import type * as LibApi from "@/lib/api";
import type { ConditionTypeDefinition } from "@/lib/api";
import type { WorkingConditionWithEmployee } from "@/lib/api/employee-conditions";

import Button from "@/components/ui/button";
import { ToastProvider } from "@/components/ui/toast";

import PersonnelWorkingConditionsPage from "../working-conditions-page";

// Mock i18next translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => options?.defaultValue || key,
    i18n: { language: "vi" },
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

const mockConditions: WorkingConditionWithEmployee[] = [
  {
    id: "cond-1",
    employee_id: "emp-1",
    employee_name: "Nguyen Van A",
    employee_code: "EMP-001",
    type: "CHILDCARE",
    start_date: "2026-01-01",
    end_date: "2026-12-31",
    is_paid: true,
    condition_type_id: "def-1",
  },
  {
    id: "cond-2",
    employee_id: "emp-2",
    employee_name: "Tran Thi B",
    employee_code: "EMP-002",
    type: "MATERNITY",
    start_date: "2026-03-01",
    end_date: null,
    is_paid: false,
    condition_type_id: "def-2",
  },
];

const mockTypeDefs: ConditionTypeDefinition[] = [
  {
    id: "def-1",
    code: "CHILDCARE",
    icon: "Baby",
    is_active: true,
    name_i18n: { vi: "Con nhỏ dưới 12 tháng", en: "Childcare" },
    affects_rules: {},
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "def-2",
    code: "MATERNITY",
    icon: "Heart",
    is_active: true,
    name_i18n: { vi: "Thai sản", en: "Maternity" },
    affects_rules: {},
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

const mockDeleteWorkingCondition = vi.fn().mockResolvedValue(undefined);

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof LibApi>();
  return {
    ...actual,
    fetchConditionTypes: vi.fn().mockImplementation(() => Promise.resolve(mockTypeDefs)),
    deleteWorkingCondition: (...args: unknown[]) => mockDeleteWorkingCondition(...args),
  };
});

vi.mock("@/hooks/use-employees", () => ({
  useOrganizationWorkingConditions: () => ({
    data: mockConditions,
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  }),
  useAllEmployees: () => ({
    employees: [
      { id: "emp-1", full_name: "Nguyen Van A", employee_code: "EMP-001" },
      { id: "emp-2", full_name: "Tran Thi B", employee_code: "EMP-002" },
    ],
    isLoading: false,
  }),
}));

const mockConfirm = vi.fn();
vi.mock("@/hooks/use-confirm", () => ({
  useConfirm: () => ({
    confirm: mockConfirm,
    ConfirmationDialog: () => <div data-testid="mock-confirm-dialog" />,
  }),
}));

// Mock WorkingConditionFormModal
vi.mock("@/components/features/employees/working-condition-form-modal", () => ({
  default: ({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) => {
    if (!isOpen) return null;
    return (
      <div data-testid="mock-working-condition-modal">
        <span>Gán Điều kiện Làm việc Đặc thù</span>
        <Button onClick={onClose} type="button">
          Close
        </Button>
      </div>
    );
  },
}));

function renderWithProviders(ui: React.ReactElement) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <ToastProvider>{ui}</ToastProvider>
    </QueryClientProvider>
  );
}

describe("PersonnelWorkingConditionsPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockSearch = { q: "" };
  });

  it("renders page header and metric cards accurately", async () => {
    renderWithProviders(<PersonnelWorkingConditionsPage />);

    expect(screen.getByText("Quản lý Điều kiện Làm việc Đặc thù")).toBeInTheDocument();
    expect(screen.getByText("Tổng hồ sơ áp dụng")).toBeInTheDocument();
    expect(screen.getByText("Đang hiệu lực")).toBeInTheDocument();
    expect(screen.getAllByText("Có hưởng lương").length).toBeGreaterThan(0);
    expect(screen.getAllByText("Không hưởng lương").length).toBeGreaterThan(0);
  });

  it("renders working condition rows for employees", async () => {
    renderWithProviders(<PersonnelWorkingConditionsPage />);

    await waitFor(() => {
      expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
      expect(screen.getAllByText("Tran Thi B").length).toBeGreaterThan(0);
      expect(screen.getAllByText("EMP-001").length).toBeGreaterThan(0);
      expect(screen.getAllByText("EMP-002").length).toBeGreaterThan(0);
    });
  });

  it("filters working conditions by employee URL filter", async () => {
    mockSearch = { employeeId: "emp-1" };
    renderWithProviders(<PersonnelWorkingConditionsPage />);

    await waitFor(() => {
      expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
      expect(screen.queryByText("Tran Thi B")).not.toBeInTheDocument();
    });
    expect(screen.getByTestId("employee-filter-chip")).toBeInTheDocument();
  });

  it("renders a clear link for employee URL filter", () => {
    mockSearch = { employeeId: "emp-1" };
    renderWithProviders(<PersonnelWorkingConditionsPage />);

    expect(screen.getByRole("link", { name: "Bỏ bộ lọc" })).toHaveAttribute("href", "/working-conditions");
  });

  it("filters working conditions by search input", async () => {
    renderWithProviders(<PersonnelWorkingConditionsPage />);

    await waitFor(() => {
      expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
    });

    const searchInput = screen.getByPlaceholderText("Tìm nhân sự, mã NV, loại chế độ...");
    fireEvent.change(searchInput, { target: { value: "Nguyen" } });

    await waitFor(() => {
      expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
      expect(screen.queryByText("Tran Thi B")).not.toBeInTheDocument();
    });
  });

  it("opens working condition modal on GÁN ĐIỀU KIỆN MỚI button click", () => {
    renderWithProviders(<PersonnelWorkingConditionsPage />);

    const assignButton = screen.getByRole("button", { name: /GÁN ĐIỀU KIỆN MỚI/i });
    fireEvent.click(assignButton);

    expect(screen.getByTestId("mock-working-condition-modal")).toBeInTheDocument();
    expect(screen.getByText("Gán Điều kiện Làm việc Đặc thù")).toBeInTheDocument();
  });
});
