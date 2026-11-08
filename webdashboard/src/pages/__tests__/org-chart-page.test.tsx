import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import OrgChartPage from "../org-chart-page";

// ── Mocks ──────────────────────────────────────────────────────────────────

// Mock child components
vi.mock("@/components/features/departments/department-config-modal", () => ({
  DepartmentConfigModal: ({ isOpen }: { isOpen: boolean }) =>
    isOpen ? <div data-testid="department-config-modal">Config Modal</div> : null,
}));

vi.mock("@/components/features/departments/department-detail-drawer", () => ({
  DepartmentDetailDrawer: ({ isOpen }: { isOpen: boolean }) =>
    isOpen ? <div data-testid="department-detail-drawer">Detail Drawer</div> : null,
}));

vi.mock("@/components/features/departments/department-form-modal", () => ({
  DepartmentFormModal: ({ isOpen }: { isOpen: boolean }) =>
    isOpen ? <div data-testid="department-form-modal">Form Modal</div> : null,
}));

vi.mock("@/components/features/departments/department-org-chart", () => ({
  DepartmentOrgChart: () => <div data-testid="department-org-chart">Org Chart</div>,
}));

// Mock confirm hook
vi.mock("@/hooks/use-confirm", () => ({
  useConfirm: () => ({
    confirm: vi.fn().mockResolvedValue(true),
    ConfirmationDialog: () => <div data-testid="confirmation-dialog" />,
  }),
}));

// Mock react-query (use useMutation + useQuery directly in component)
vi.mock("@tanstack/react-query", async () => {
  const actual = await vi.importActual<typeof import("@tanstack/react-query")>(
    "@tanstack/react-query",
  );
  return actual;
});

// ── API and workspace state ────────────────────────────────────────────────


vi.mock("@/lib/api", () => ({
  fetchDepartments: vi.fn().mockResolvedValue([
    { id: "dept-1", name: "Engineering", manager_id: "emp-1" },
  ]),
  fetchEmployees: vi.fn().mockResolvedValue({
    data: [{ id: "emp-1", full_name: "Alice", department_id: "dept-1" }],
    total: 1,
  }),
  deleteDepartment: vi.fn().mockResolvedValue(undefined),
}));

function renderOrgChart() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false } },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <OrgChartPage />
    </QueryClientProvider>,
  );
}

describe("OrgChartPage", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ── Happy path (renders with workspace) ─────────────────────────────────

  it("renders org chart when workspace is selected", () => {
    renderOrgChart();

    expect(screen.getByTestId("department-org-chart")).toBeDefined();
  });

  it("renders search input", () => {
    renderOrgChart();

    // The placeholder "departments.search_placeholder" resolves to Vietnamese
    const input = screen.getByPlaceholderText(/phòng ban/i);
    expect(input).toBeDefined();
  });

  // ── Loading state ───────────────────────────────────────────────────────

  it("renders without crashing while queries are loading", () => {
    renderOrgChart();

    // During loading, the org chart component still renders
    expect(screen.getByTestId("department-org-chart")).toBeDefined();
  });

  // ── Modal state management ──────────────────────────────────────────────

  it("config modal is closed by default", () => {
    renderOrgChart();

    expect(screen.queryByTestId("department-config-modal")).toBeNull();
  });

  it("form modal is closed by default", () => {
    renderOrgChart();

    expect(screen.queryByTestId("department-form-modal")).toBeNull();
  });

  it("detail drawer is closed by default (no selected dept)", () => {
    renderOrgChart();

    expect(screen.queryByTestId("department-detail-drawer")).toBeNull();
  });

  // ── Delete flow ─────────────────────────────────────────────────────────

  it("renders confirmation dialog", () => {
    renderOrgChart();

    expect(screen.getByTestId("confirmation-dialog")).toBeDefined();
  });

  // ── Search input ───────────────────────────────────────────────────────

  it("allows user to type in the search input", async () => {
    const user = userEvent.setup();
    renderOrgChart();

    const input = screen.getByPlaceholderText(/phòng ban/i);
    await user.type(input, "Engineering");

    expect((input as HTMLInputElement).value).toBe("Engineering");
  });
});
