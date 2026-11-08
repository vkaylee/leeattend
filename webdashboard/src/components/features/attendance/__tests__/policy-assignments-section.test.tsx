import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api";

import { PolicyAssignmentsSection } from "../policy-assignments-section";


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
    (api.fetchPolicyAssignments as Mock).mockResolvedValue([]);
    (api.fetchDepartments as Mock).mockResolvedValue([
      { id: "dept-1", name: "Engineering" }
    ]);
    (api.fetchEmployees as Mock).mockResolvedValue({
      data: [{ id: "emp-1", full_name: "Alice", employee_id_number: "EMP001" }],
      total: 1
    });
  });

  it("renders correctly with empty state", async () => {
    renderWithProviders(<PolicyAssignmentsSection   />);
    
    // Header should be rendered
    expect(await screen.findByText(/attendance_policy_overrides.title/i)).toBeDefined();
    
    // Wait for the empty state
    await waitFor(() => {
      expect(screen.getByText(/attendance_policy_overrides.empty_state.title/i)).toBeDefined();
    });
  });

  it("displays assignments and resolves target names", async () => {
    (api.fetchPolicyAssignments as Mock).mockResolvedValue([
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

    renderWithProviders(<PolicyAssignmentsSection   />);
    
    // Name resolved from mock departments
    expect(await screen.findByText(/Engineering/i)).toBeDefined();
    const depts = await screen.findAllByText(/attendance_policy_overrides.labels.department/i);
    expect(depts.length).toBeGreaterThan(0);
  });

  it("opens the drawer and submits a new override", async () => {
    (api.upsertPolicyAssignment as Mock).mockResolvedValue({ id: "new-1" });
    
    renderWithProviders(<PolicyAssignmentsSection   />);
    
    // Click ADD OVERRIDE button
    const addButton = await screen.findByText(/attendance_policy_overrides.add_button/i);
    fireEvent.click(addButton);
    
    // Wait for drawer to open
    await waitFor(() => {
      expect(screen.getByText(/attendance_policy_overrides.drawer.add_title/i)).toBeDefined();
    });

    // Select Department type
    const deptButton = screen.getByText(/Department/i, { selector: 'span' });
    fireEvent.click(deptButton);

    // Save override
    const saveButton = screen.getByText(/attendance_policy_overrides.drawer.save_button/i);
    fireEvent.click(saveButton);
    
    // The validation should prevent save since no target is selected
    // Because combobox value is ""
    expect(api.upsertPolicyAssignment).not.toHaveBeenCalled();
  });
});
