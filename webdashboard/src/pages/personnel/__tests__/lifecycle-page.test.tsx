import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import type * as LibApi from "@/lib/api";
import type { LifecycleInstanceSummary } from "@/lib/api";
import PersonnelLifecyclePage from "../lifecycle-page";

// Mock i18next translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => options?.defaultValue || key,
    i18n: { language: "vi" },
  }),
}));

// Mock standalone organization ID
let mockWorkspaceId = "ws-1";
vi.mock("@/lib/api/helpers", () => ({
  getStandaloneOrganizationId: () => mockWorkspaceId,
}));

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useSearch: () => ({ q: "" }),
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

const mockInstances: LifecycleInstanceSummary[] = [
  {
    id: "cl-1",
    workspace_id: "ws-1",
    employment_id: "emp-rec-1",
    employee_id: "emp-1",
    employee_name: "Nguyen Van A",
    employee_code: "EMP-001",
    department_name: "Engineering",
    template_id: "tpl-1",
    template_name: "Kỹ sư phần mềm mới",
    checklist_type: "onboarding",
    status: "in_progress",
    progress_percent: 60,
    total_items: 5,
    completed_items: 3,
    started_at: "2026-03-01T00:00:00Z",
    completed_at: null,
    created_at: "2026-03-01T00:00:00Z",
    updated_at: "2026-03-02T00:00:00Z",
  },
  {
    id: "cl-2",
    workspace_id: "ws-1",
    employment_id: "emp-rec-2",
    employee_id: "emp-2",
    employee_name: "Tran Thi B",
    employee_code: "EMP-002",
    department_name: "Human Resources",
    template_id: "tpl-2",
    template_name: "Bàn giao tài sản nghỉ việc",
    checklist_type: "offboarding",
    status: "in_progress",
    progress_percent: 25,
    total_items: 4,
    completed_items: 1,
    started_at: "2026-03-05T00:00:00Z",
    completed_at: null,
    created_at: "2026-03-05T00:00:00Z",
    updated_at: "2026-03-05T00:00:00Z",
  },
];

vi.mock("@/lib/api", async (importOriginal) => {
  const actual = await importOriginal<typeof LibApi>();
  return {
    ...actual,
    getStandaloneOrganizationId: () => mockWorkspaceId,
  };
});

vi.mock("@/hooks/use-lifecycle", () => ({
  useWorkspaceLifecycleInstances: () => ({
    data: mockInstances,
    isLoading: false,
    error: null,
    refetch: vi.fn(),
  }),
  useLifecycleTemplates: () => ({
    data: [
      { id: "tpl-1", name: "Kỹ sư phần mềm mới", checklist_type: "onboarding", is_active: true },
      { id: "tpl-2", name: "Bàn giao tài sản nghỉ việc", checklist_type: "offboarding", is_active: true },
    ],
    isLoading: false,
  }),
  lifecycleKeys: {
    all: ["lifecycle"] as const,
  },
}));

// Mock LifecycleAssignModal
vi.mock("@/components/features/employees/lifecycle-assign-modal", () => ({
  default: ({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) => {
    if (!isOpen) return null;
    return (
      <div data-testid="mock-lifecycle-assign-modal">
        <span>Khởi tạo Quy trình Vòng đời</span>
        <button onClick={onClose} type="button">
          Close
        </button>
      </div>
    );
  },
  formatChecklistTypeLabel: (type: string) => type.toUpperCase(),
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

describe("PersonnelLifecyclePage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockWorkspaceId = "ws-1";
  });

  it("renders page header and metric cards accurately", async () => {
    renderWithProviders(<PersonnelLifecyclePage />);

    expect(screen.getByText("Quản lý Quy trình & Vòng đời Nhân sự")).toBeInTheDocument();
    expect(screen.getByText("Tổng quy trình")).toBeInTheDocument();
    expect(screen.getAllByText("Đang tiến hành").length).toBeGreaterThan(0);
    expect(screen.getByText("Đang Onboarding")).toBeInTheDocument();
    expect(screen.getByText("Đang Bàn giao")).toBeInTheDocument();
  });

  it("renders lifecycle workflow rows for employees", async () => {
    renderWithProviders(<PersonnelLifecyclePage />);

    await waitFor(() => {
      expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
      expect(screen.getAllByText("Tran Thi B").length).toBeGreaterThan(0);
      expect(screen.getAllByText("EMP-001").length).toBeGreaterThan(0);
      expect(screen.getAllByText("EMP-002").length).toBeGreaterThan(0);
    });
  });

  it("filters lifecycle rows by search input", async () => {
    renderWithProviders(<PersonnelLifecyclePage />);

    await waitFor(() => {
      expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
    });

    const searchInput = screen.getByPlaceholderText("Tìm nhân sự, mã NV, phòng ban, mẫu checklist...");
    fireEvent.change(searchInput, { target: { value: "Nguyen" } });

    await waitFor(() => {
      expect(screen.getAllByText("Nguyen Van A").length).toBeGreaterThan(0);
      expect(screen.queryByText("Tran Thi B")).not.toBeInTheDocument();
    });
  });

  it("opens lifecycle assign modal on KHỞI TẠO QUY TRÌNH button click", () => {
    renderWithProviders(<PersonnelLifecyclePage />);

    const createButton = screen.getByRole("button", { name: /KHỞI TẠO QUY TRÌNH/i });
    fireEvent.click(createButton);

    expect(screen.getByTestId("mock-lifecycle-assign-modal")).toBeInTheDocument();
    expect(screen.getByText("Khởi tạo Quy trình Vòng đời")).toBeInTheDocument();
  });
});
