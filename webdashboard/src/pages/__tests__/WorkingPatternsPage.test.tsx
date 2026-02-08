import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import WorkingPatternsPage from "../WorkingPatternsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "../../lib/api";
import React from "react";

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
vi.mock("@/components/common/EmployeeSearch", () => ({
  EmployeeSearch: ({ value, onChange, onSelect, multiple }: any) => {
    const handler = onSelect || onChange || (() => {});
    return (
      <div data-testid="employee-search-mock">
        <button 
          type="button"
          onClick={() => {
            if (multiple) {
              handler(["emp-1", "emp-2"]);
            } else {
              handler("emp-1");
            }
          }}
        >
          Select Employees Mock
        </button>
        {multiple && Array.isArray(value) && value.includes("emp-1") && <span>Selected: John Doe</span>}
      </div>
    );
  }
}));

// Mock Toast
const mockToastSuccess = vi.fn();
const mockToastError = vi.fn();
vi.mock("../../components/ui/Toast", () => ({
  useToast: () => ({
    success: mockToastSuccess,
    error: mockToastError,
    warning: vi.fn(),
  }),
}));

// Override i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, options?: any) => {
        if (key === "patterns.messages.bulk_assign_success_count") return `Success: ${options?.count}`;
        if (key === "patterns.messages.bulk_assign_conflict") return `Conflict: ${options?.count}`;
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
    (api.getCurrentWorkspaceId as any).mockReturnValue("ws-1");
    (api.fetchWorkingPatterns as any).mockResolvedValue([
      { id: "pat-1", name: "Standard Office", description: "Mon-Fri", is_active: true }
    ]);
    (api.fetchShifts as any).mockResolvedValue([
      { id: "shift-1", name: "Morning", start_time: "08:00", end_time: "17:00" }
    ]);
    (api.fetchEmployees as any).mockResolvedValue({
      data: [
        { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, hire_date: "2025-01-01", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" },
        { id: "emp-2", full_name: "Jane Smith", employee_id_number: "EMP002", workspace_id: "ws-1", is_active: true, hire_date: "2025-01-02", created_at: "2025-01-02T00:00:00Z", updated_at: "2025-01-02T00:00:00Z" }
      ],
      total: 2,
      page: 1,
      limit: 50,
    });
    (api.fetchPatternDetails as any).mockResolvedValue([]);
  });

  it("renders patterns list", async () => {
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <WorkingPatternsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    expect(await screen.findByText("Standard Office")).toBeDefined();
    expect(screen.getByText("Mon-Fri")).toBeDefined();
  });

  it("opens create modal and submits new pattern", async () => {
    const user = userEvent.setup();
    (api.createWorkingPattern as any).mockResolvedValue({ id: "pat-new" });

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <WorkingPatternsPage />
        </BrowserRouter>
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
    (api.bulkAssignPattern as any).mockResolvedValue({
        success_count: 2,
        failed_employees: []
    });

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <WorkingPatternsPage />
        </BrowserRouter>
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
    (api.bulkAssignPattern as any).mockResolvedValue({
        success_count: 0,
        failed_employees: ["emp-1"]
    });

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <WorkingPatternsPage />
        </BrowserRouter>
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