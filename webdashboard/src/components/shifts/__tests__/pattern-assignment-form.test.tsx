import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "@/lib/api";
import { Employee, Shift, WorkingPattern } from "@/lib/api/types";

import { PatternAssignmentForm } from "../pattern-assignment-form";

// Mock API
vi.mock("@/lib/api", () => ({
  fetchPatternDetails: vi.fn(),
  getCurrentWorkspaceId: vi.fn(() => "ws-1"),
}));

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("PatternAssignmentForm", () => {
  let queryClient: QueryClient;
  const mockEmployee = {
    id: "emp-1",
    full_name: "John Doe",
    employee_id_number: "EMP001",
    workspace_id: "ws-1",
    tags: [],
    active_employment_id: "employment-1",
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
  } as unknown as Employee;

  const mockPatterns: WorkingPattern[] = [
    {
      id: "pat-1",
      name: "Weekly Office",
      description: "Mon-Fri Standard shift pattern",
      pattern_type: "weekly",
      is_active: true,
    },
    {
      id: "pat-2",
      name: "Rotation 4/4",
      description: "4 days on, 4 days off rotation shift pattern description",
      pattern_type: "rotation",
      cycle_length_days: 8,
      is_active: true,
    }
  ] as unknown as WorkingPattern[];

  const mockShifts: Shift[] = [
    {
      id: "shift-1",
      name: "Morning Shift",
      start_time: "08:00:00",
      end_time: "17:00:00",
      report_symbol: "HC",
      workspace_id: "ws-1",
      created_at: "2025-01-01T00:00:00Z",
      updated_at: "2025-01-01T00:00:00Z",
    }
  ] as unknown as Shift[];

  const defaultProps = {
    selectedEmployee: mockEmployee,
    patterns: mockPatterns,
    selectedPatternId: "pat-1",
    setSelectedPatternId: vi.fn(),
    startDate: "2026-06-01",
    setStartDate: vi.fn(),
    endDate: "2026-06-30",
    setEndDate: vi.fn(),
    isForever: false,
    setIsForever: vi.fn(),
    onAssign: vi.fn(),
    isLoading: false,
    shifts: mockShifts,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0, staleTime: 0 },
      },
    });
    vi.mocked(api.fetchPatternDetails).mockResolvedValue([
      { schedule_day: 1, shift_id: "shift-1", is_off: false, workspace_id: "ws-1", pattern_id: "pat-1" }
    ] as any);
  });

  const renderComponent = (props = defaultProps) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <PatternAssignmentForm {...props} />
      </QueryClientProvider>
    );
  };

  it("renders the pattern list and effective date picker", () => {
    renderComponent();

    // Check header
    expect(screen.getByText("patterns.select_pattern")).toBeDefined();

    // Check patterns names and descriptions
    expect(screen.getByText("Weekly Office")).toBeDefined();
    expect(screen.getByText("Mon-Fri Standard shift pattern")).toBeDefined();
    expect(screen.getByText("Rotation 4/4")).toBeDefined();

    // Check effective from date field
    expect(screen.getByText("common.effective_from")).toBeDefined();
    expect(screen.getByDisplayValue("2026-06-01")).toBeDefined();
  });

  it("calls setSelectedPatternId when a pattern card is clicked", async () => {
    const user = userEvent.setup();
    const setSelectedPatternId = vi.fn();
    renderComponent({
      ...defaultProps,
      setSelectedPatternId,
    });

    const secondPatternCard = screen.getByText("Rotation 4/4").closest("div[role='button']");
    expect(secondPatternCard).toBeDefined();
    await user.click(secondPatternCard!);

    expect(setSelectedPatternId).toHaveBeenCalledWith("pat-2");
  });

  it("toggles end date state between definite and indefinite on infinity button click", async () => {
    const user = userEvent.setup();
    const setIsForever = vi.fn();
    renderComponent({
      ...defaultProps,
      setIsForever,
    });

    const infinityButton = screen.getByRole("button", { name: "∞" });
    await user.click(infinityButton);

    expect(setIsForever).toHaveBeenCalledWith(true);
  });

  it("shows indefinite input and button state when isForever is true", () => {
    renderComponent({
      ...defaultProps,
      isForever: true,
    });

    expect(screen.getByDisplayValue("common.indefinite")).toBeDefined();
    expect(screen.getByRole("button", { name: "∞" })).toBeDefined();
  });

  it("calls onAssign when assign button is clicked", async () => {
    const user = userEvent.setup();
    const onAssign = vi.fn();
    renderComponent({
      ...defaultProps,
      onAssign,
    });

    const assignButton = screen.getByRole("button", { name: /shifts\.assign_btn/i });
    await user.click(assignButton);

    expect(onAssign).toHaveBeenCalled();
  });

  it("disables assign button when isLoading is true", () => {
    renderComponent({
      ...defaultProps,
      isLoading: true,
    });

    const assignButton = screen.getByRole("button", { name: /shifts\.assign_btn/i });
    expect(assignButton.hasAttribute("disabled")).toBe(true);
  });

  it("applies dynamic styling for custom list heights and compact card styling", () => {
    renderComponent();

    // Check list container classes
    const listContainer = screen.getByText("Weekly Office").closest(".custom-scrollbar");
    expect(listContainer?.className).toContain("max-h-[45dvh]");
    expect(listContainer?.className).toContain("lg:max-h-[50dvh]");
    expect(listContainer?.className).toContain("min-h-[240px]");

    // Check card styling classes on selected card
    const cardElement = screen.getByText("Weekly Office").closest("div[role='button']");
    expect(cardElement?.className).toContain("px-5");
    expect(cardElement?.className).toContain("py-4");
    expect(cardElement?.className).toContain("rounded-[24px]");
    expect(cardElement?.className).not.toContain("min-h-[120px]");
    expect(cardElement?.className).not.toContain("overflow-hidden");
  });
});
