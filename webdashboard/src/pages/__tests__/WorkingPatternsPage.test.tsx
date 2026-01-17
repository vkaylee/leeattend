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
      <button 
        onClick={() => {
          if (multiple) {
            onChange(["emp-1"]);
          } else {
            onChange("emp-1");
          }
        }}
      >
        Select John Doe
      </button>
      {multiple && Array.isArray(value) && value.includes("emp-1") && <span>Selected: John Doe</span>}
    </div>
  )
}));

// Mock Toast
vi.mock("../../components/Toast", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
  }),
}));

// Override i18n for this test to return keys
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
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
      { id: "emp-1", first_name: "John", last_name: "Doe", employee_id_number: "EMP001" }
    ]);
    (api.fetchPatternDetails as any).mockResolvedValue([
      { day_of_week: 1, shift_id: "shift-1" },
      { day_of_week: 2, shift_id: "shift-1" },
      { day_of_week: 3, shift_id: "shift-1" },
      { day_of_week: 4, shift_id: "shift-1" },
      { day_of_week: 5, shift_id: "shift-1" },
      { day_of_week: 6, shift_id: null },
      { day_of_week: 0, shift_id: null },
    ]);
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

    // Open Modal
    await user.click(screen.getByText("patterns.create_btn"));

    // Fill Form
    const nameInput = await screen.findByPlaceholderText("e.g., Office Standard, Night Shift Rotation");
    await user.type(nameInput, "New Rotation");

    // Click Prefill
    await user.click(screen.getByText("patterns.actions.prefill_standard"));

    // Submit
    await user.click(screen.getByText("patterns.create_submit"));

    await waitFor(() => {
      expect(api.createWorkingPattern).toHaveBeenCalledWith(
        "ws-1",
        expect.objectContaining({
          name: "New Rotation",
          details: expect.arrayContaining([
            { day_of_week: 1, shift_id: "shift-1" }, // Mon
            { day_of_week: 5, shift_id: "shift-1" }, // Fri
            { day_of_week: 0, shift_id: null },      // Sun
          ])
        })
      );
    });
  });

  it("expands pattern details", async () => {
    const user = userEvent.setup();
    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <WorkingPatternsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    // Initial state: details hidden
    expect(screen.queryByText("days.monday")).toBeNull();

    // Click Expand
    await user.click(await screen.findByText("common.expand"));

    // Wait for details
    expect(await screen.findByText("days.monday")).toBeDefined();
    expect(screen.getAllByText("Morning")[0]).toBeDefined();
  });

  it("opens bulk assign modal and submits", async () => {
    const user = userEvent.setup();
    (api.bulkAssignPattern as any).mockResolvedValue({});

    render(
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <WorkingPatternsPage />
        </BrowserRouter>
      </QueryClientProvider>
    );

    // Open bulk assign modal
    const bulkBtn = screen.getByText("patterns.bulk_assign_btn");
    await user.click(bulkBtn);

    // Select Pattern
    const patternSelect = await screen.findByDisplayValue("patterns.form.select_pattern_placeholder");
    await user.selectOptions(patternSelect, "pat-1");

    // Select Employee (via mock button)
    const select = screen.getByTestId("employee-search-mock");
    fireEvent.change(select, { target: { value: "emp-1" } });

    // Submit
    const submitBtn = screen.getByText("patterns.bulk_assign_submit");

    await waitFor(() => {
      expect(api.bulkAssignPattern).toHaveBeenCalledWith(
        "ws-1",
        expect.objectContaining({
          employee_ids: expect.arrayContaining(["emp-1"]),
          pattern_id: "pat-1"
        })
      );
    });
  });
});
