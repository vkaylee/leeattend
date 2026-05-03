import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../../lib/api";
import { WorkingPattern, Shift, PaginatedResponse, Employee, WorkingPatternDetail, BulkAssignResponse } from "../../lib/api/types";
import WorkingPatternsPage from "../working-patterns-page";


// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
 
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock API
vi.mock("../../lib/api", () => ({
  fetchWorkingPatterns: vi.fn(),
  createWorkingPattern: vi.fn(),
  fetchPatternDetails: vi.fn(),
  fetchShifts: vi.fn(),
  fetchEmployees: vi.fn(),
  bulkAssignPattern: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
}));

// Mock EmployeeSearch
 
vi.mock("@/components/common/employee-search", () => ({
  EmployeeSearch: ({ value, onChange, onSelect, multiple }: { value: unknown; onChange: (_v: unknown) => void; onSelect: (_v: unknown) => void; multiple: boolean }) => {
    const handler = onSelect || onChange || (() => { /* empty */ });
    return (
      <div data-testid="employee-search-mock">
        <button 
          onClick={() => {
            if (multiple) {
              handler(["emp-1", "emp-2"]);
            } else {
              handler("emp-1");
            }
          }}
          type="button"
        >
          Select Employees Mock
        </button>
        {!!multiple && Array.isArray(value) && value.includes("emp-1") && <span>Selected: John Doe</span>}
      </div>
    );
  }
}));

// Mock Toast
const mockToastSuccess = vi.fn();
const mockToastError = vi.fn();
vi.mock("../../components/ui/toast", () => ({
  useToast: () => ({
    success: mockToastSuccess,
    error: mockToastError,
    warning: vi.fn(),
  }),
}));

// Override i18n
vi.mock('react-i18next', () => ({
 
  useTranslation: () => ({
    t: (key: string, options?: Record<string, unknown>) => {
        if (key === "patterns.messages.bulk_assign_success_count") return `Success: ${options?.count ?? ""}`;
        if (key === "patterns.messages.bulk_assign_conflict") return `Conflict: ${options?.count ?? ""}`;
        return key;
    },
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("WorkingPatternsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0, staleTime: 0 },
      },
 
    });
 
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("ws-1");
    vi.mocked(api.fetchWorkingPatterns).mockResolvedValue([
      { id: "pat-1", name: "Standard Office", description: "Mon-Fri", is_active: true }
 
    ] as unknown as WorkingPattern[]);
    vi.mocked(api.fetchShifts).mockResolvedValue([
      { id: "shift-1", name: "Morning", start_time: "08:00", end_time: "17:00" }
 
    ] as unknown as Shift[]);
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [
        { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, hire_date: "2025-01-01", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" },
        { id: "emp-2", full_name: "Jane Smith", employee_id_number: "EMP002", workspace_id: "ws-1", is_active: true, hire_date: "2025-01-02", created_at: "2025-01-02T00:00:00Z", updated_at: "2025-01-02T00:00:00Z" }
      ],
      total: 2,
      page: 1,
      limit: 50,
 
    });
    vi.mocked(api.fetchPatternDetails).mockResolvedValue([]);
  });

  it("renders patterns list", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <WorkingPatternsPage />
      </QueryClientProvider>
    );

    expect(await screen.findByText("Standard Office")).toBeDefined();
    expect(screen.getByText("Mon-Fri")).toBeDefined();
  });

  it("opens create modal and submits new pattern", async () => {
 
    const user = userEvent.setup();
    vi.mocked(api.createWorkingPattern).mockResolvedValue({ id: "pat-new" } as unknown as WorkingPattern);

    render(
      <QueryClientProvider client={queryClient}>
        <WorkingPatternsPage />
      </QueryClientProvider>
    );

    await user.click(screen.getByText(/patterns.create_btn/i));

    const nameInput = await screen.findByPlaceholderText(/patterns.form.name_placeholder/i);
    await user.type(nameInput, "New Rotation");
    await user.click(screen.getByText(/patterns.form.prefill_standard/i));
    await user.click(screen.getByText(/patterns.modal.submit_btn/i));

    await waitFor(() => {
      expect(api.createWorkingPattern).toHaveBeenCalled();
    }, { timeout: 20000 });
  }, 30000);

  it("opens bulk assign modal and handles success response", async () => {
    const user = userEvent.setup();
 
    // Mock Success Response
    vi.mocked(api.bulkAssignPattern).mockResolvedValue({
        success_count: 2,
        failed_employees: []
    });

    render(
      <QueryClientProvider client={queryClient}>
        <WorkingPatternsPage />
      </QueryClientProvider>
    );

    // Open Modal
    await user.click(screen.getByText(/patterns.bulk_assign_btn/i));

    // Select Pattern
    const dialog = await screen.findByRole("dialog");
    const patternOption = await within(dialog).findByText("Standard Office", { selector: 'span' });
    await user.click(patternOption.closest("button")!);

    // Select Employees (using mock button)
    await user.click(screen.getByText("Select Employees Mock"));

    // Submit
    await user.click(screen.getByText(/patterns.modal.assign_btn/i));

    await waitFor(() => {
      expect(api.bulkAssignPattern).toHaveBeenCalledWith(
        "ws-1",
        expect.objectContaining({
          employee_ids: ["emp-1", "emp-2"],
          pattern_id: "pat-1"
        })
      );
    });

    // Check Toast Success Message
    expect(mockToastSuccess).toHaveBeenCalledWith("Success: 2");
  });

  it("opens bulk assign modal and handles conflict error", async () => {
    const user = userEvent.setup();
 
    // Mock Conflict Response
    vi.mocked(api.bulkAssignPattern).mockResolvedValue({
        success_count: 0,
        failed_employees: ["emp-1"]
    });

    render(
      <QueryClientProvider client={queryClient}>
        <WorkingPatternsPage />
      </QueryClientProvider>
    );

    // Open Modal
    await user.click(screen.getByText(/patterns.bulk_assign_btn/i));

    // Select Pattern
    const dialog = await screen.findByRole("dialog");
    const patternOption = await within(dialog).findByText("Standard Office", { selector: 'span' });
    await user.click(patternOption.closest("button")!);

    // Select Employees
    await user.click(screen.getByText("Select Employees Mock"));

    // Submit
    await user.click(screen.getByText(/patterns.modal.assign_btn/i));

    await waitFor(() => {
        expect(api.bulkAssignPattern).toHaveBeenCalled();
    });

    // Check Toast Error Message
    expect(mockToastError).toHaveBeenCalledWith("Conflict: 1");
  });
});