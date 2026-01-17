import { render, screen, waitFor, fireEvent } from "@testing-library/react";
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
  EmployeeSearch: ({ value, onChange, multiple, testId }: any) => (
    <div className="space-y-2" data-testid={testId || "employee-search-mock"}>
      <input 
        data-testid="employee-search-input"
        value={Array.isArray(value) ? value.join(",") : (value || "")}
        onChange={(e) => {
          const val = e.target.value;
          if (multiple) {
            onChange(val ? val.split(",") : []);
          } else {
            onChange(val);
          }
        }}
      />
      <button 
        type="button"
        onClick={() => {
          if (multiple) {
            onChange(["emp-1", "emp-2"]);
          } else {
            onChange("emp-1");
          }
        }}
      >
        Select Employees Mock
      </button>
      {multiple && Array.isArray(value) && value.includes("emp-1") && <span>Selected: John Doe</span>}
    </div>
  )
}));

// Mock Toast
const mockToastSuccess = vi.fn();
const mockToastError = vi.fn();
vi.mock("../../components/Toast", () => ({
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
    (api.fetchEmployees as any).mockResolvedValue([
      { id: "emp-1", first_name: "John", last_name: "Doe" },
      { id: "emp-2", first_name: "Jane", last_name: "Smith" }
    ]);
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

    await user.click(screen.getByText("patterns.create_btn"));

    const nameInput = await screen.findByPlaceholderText("e.g., Office Standard, Night Shift Rotation");
    await user.type(nameInput, "New Rotation");
    await user.click(screen.getByText("patterns.actions.prefill_standard"));
    await user.click(screen.getByText("patterns.create_submit"));

    await waitFor(() => {
      expect(api.createWorkingPattern).toHaveBeenCalled();
    });
  });

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
    await user.click(screen.getByText("patterns.bulk_assign_btn"));

    // Select Pattern
    const patternSelect = screen.getByDisplayValue("patterns.form.select_pattern_placeholder");
    
    // Select option by value
    await user.selectOptions(patternSelect, "pat-1");

    // Select Employees (using mock button)
    await user.click(screen.getByText("Select Employees Mock"));

    // Submit
    await user.click(screen.getByText("patterns.bulk_assign_submit"));

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
    await user.click(screen.getByText("patterns.bulk_assign_btn"));

    // Select Pattern
    const patternSelect = screen.getByDisplayValue("patterns.form.select_pattern_placeholder");
    await user.selectOptions(patternSelect, "pat-1");

    // Select Employees
    await user.click(screen.getByText("Select Employees Mock"));

    // Submit
    await user.click(screen.getByText("patterns.bulk_assign_submit"));

    await waitFor(() => {
        expect(api.bulkAssignPattern).toHaveBeenCalled();
    });

    // Check Toast Error Message
    expect(mockToastError).toHaveBeenCalledWith("Conflict: 1");
  });
});