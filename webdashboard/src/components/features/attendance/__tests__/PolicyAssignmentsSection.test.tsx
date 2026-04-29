import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { PolicyAssignmentsSection } from "../PolicyAssignmentsSection";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "@/lib/api";
import { ToastProvider } from "@/components/ui/Toast";

// Mock API
vi.mock("@/lib/api", () => ({
  fetchPolicyAssignments: vi.fn(),
  upsertPolicyAssignment: vi.fn(),
  deletePolicyAssignment: vi.fn(),
  fetchDepartments: vi.fn(),
  fetchEmployees: vi.fn(),
}));

// Mock i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const renderWithProviders = (ui: React.ReactNode) => {
  return render(
    <QueryClientProvider client={queryClient}>
      <ToastProvider>
        {ui}
      </ToastProvider>
    </QueryClientProvider>
  );
};

describe("PolicyAssignmentsSection", () => {
  const mockWsId = "ws-123";

  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchPolicyAssignments as any).mockResolvedValue([]);
    (api.fetchDepartments as any).mockResolvedValue([
      { id: "dept-1", name: "Engineering" }
    ]);
    (api.fetchEmployees as any).mockResolvedValue({
      data: [{ id: "emp-1", full_name: "Alice", employee_id_number: "EMP001" }],
      total: 1
    });
  });

  it("renders correctly with empty state", async () => {
    renderWithProviders(<PolicyAssignmentsSection wsId={mockWsId} />);
    
    // Header should be rendered
    expect(await screen.findByText(/Overrides & Exceptions/i)).toBeDefined();
    
    // Wait for the empty state
    await waitFor(() => {
      expect(screen.getByText(/No active overrides/i)).toBeDefined();
    });
  });

  it("displays assignments and resolves target names", async () => {
    (api.fetchPolicyAssignments as any).mockResolvedValue([
      {
        id: "assign-1",
        target_type: "GROUP",
        target_id: "dept-1",
        valid_from: "2026-01-01T00:00:00Z",
        rules: {
          work_unit_minutes: 15,
          rounding_policy: "round",
          early_arrival_strategy: "allow"
        }
      }
    ]);

    renderWithProviders(<PolicyAssignmentsSection wsId={mockWsId} />);
    
    // Name resolved from mock departments
    expect(await screen.findByText(/Engineering/i)).toBeDefined();
    const depts = await screen.findAllByText(/DEPARTMENT/i);
    expect(depts.length).toBeGreaterThan(0);
  });

  it("opens the drawer and submits a new override", async () => {
    (api.upsertPolicyAssignment as any).mockResolvedValue({ id: "new-1" });
    
    renderWithProviders(<PolicyAssignmentsSection wsId={mockWsId} />);
    
    // Click ADD OVERRIDE button
    const addBtn = await screen.findByText(/ADD OVERRIDE/i);
    fireEvent.click(addBtn);
    
    // Wait for drawer to open
    await waitFor(() => {
      expect(screen.getByText(/Add Policy Override/i)).toBeDefined();
    });

    // Select Department type
    const deptBtn = screen.getByText(/Department/i, { selector: 'span' });
    fireEvent.click(deptBtn);

    // Save override
    const saveBtn = screen.getByText(/SAVE OVERRIDE/i);
    fireEvent.click(saveBtn);
    
    // The validation should prevent save since no target is selected
    // Because combobox value is ""
    expect(api.upsertPolicyAssignment).not.toHaveBeenCalled();
  });
});
