import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import * as api from "@/lib/api/leaves";

import { LeaveTypeManager } from "../leave-type-manager";

const mockNavigate = vi.fn();
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => mockNavigate,
}));

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, fallback: string) => fallback || key
  }),
}));

// Mock API
vi.mock("@/lib/api/leaves", () => ({
  fetchLeaveTypes: vi.fn(),
  updateLeaveTypeConfig: vi.fn(),
  createLeaveType: vi.fn(),
  deleteLeaveType: vi.fn(),
}));

// Mock Toast
vi.mock("@/components/ui/toast", () => ({ 
  useToast: () => ({ success: vi.fn(), error: vi.fn() }) 
}));

const queryClient = new QueryClient({ 
  defaultOptions: { 
    queries: { retry: false } 
  } 
});

describe("LeaveTypeManager & Template Gallery", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    queryClient.clear();
  });

  it("renders empty state when no leave types exist", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager   />
      </QueryClientProvider>
    );

    expect(await screen.findByText(/No Leave Types Configured/i)).toBeDefined();
    expect(screen.getByText(/Get Started/i)).toBeDefined();
  });

  it("renders leave types list when data exists", async () => {
    const mockLeaveTypes = [
      { id: "1", code: "AL", name: "Annual Leave", is_paid: true, policy_config: {} }
    ];
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue(mockLeaveTypes as never);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager   />
      </QueryClientProvider>
    );

    expect(await screen.findByRole("heading", { name: /Annual Leave/i })).toBeDefined();
    expect(screen.getByText(/Code: AL/i)).toBeDefined();
  });

  it("renders active policy status badges on leave type cards", async () => {
    const mockLeaveTypes = [
      { 
        id: "1", 
        code: "AL", 
        name: "Annual Leave", 
        is_paid: true, 
        policy_config: {
          approval_required: true,
          requires_document: true,
          allow_negative_balance: true,
        }
      }
    ];
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue(mockLeaveTypes as never);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager   />
      </QueryClientProvider>
    );

    // Verify leave type card renders basic info
    expect(await screen.findByRole("heading", { name: /Annual Leave/i })).toBeDefined();

    // Verify configurations are rendered as badges on the card
    expect(screen.getByText(/Approval/i)).toBeDefined();
    expect(screen.getByText(/Document/i)).toBeDefined();
    expect(screen.getByText(/Negative Balance/i)).toBeDefined();
  });


  it("navigates to new leave type page when clicking Get Started in empty state", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager />
      </QueryClientProvider>
    );

    fireEvent.click(await screen.findByText(/Get Started/i));
    expect(mockNavigate).toHaveBeenCalledWith({ to: "/settings/leave/types/new" });
  });

  it("navigates to new leave type page when clicking Add Leave Type button", async () => {
    const mockLeaveTypes = [
      { id: "1", code: "AL", name: "Annual Leave", is_paid: true, policy_config: {} }
    ];
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue(mockLeaveTypes as never);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager />
      </QueryClientProvider>
    );

    fireEvent.click(await screen.findByText(/Add Leave Type/i));
    expect(mockNavigate).toHaveBeenCalledWith({ to: "/settings/leave/types/new" });
  });

  it("allows editing an existing leave type and toggling configuration switches via their tracks", async () => {
    const mockLeaveTypes = [
      { 
        id: "1", 
        code: "AL", 
        name: "Annual Leave", 
        is_paid: true, 
        policy_config: {
          approval_required: false,
          requires_document: false,
          allow_negative_balance: false,
        }
      }
    ];
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue(mockLeaveTypes as never);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager   />
      </QueryClientProvider>
    );

    // Wait for the leave type list to render
    expect(await screen.findByRole("heading", { name: /Annual Leave/i })).toBeDefined();

    // Find the edit button (button containing the pencil/edit icon)
    const editButtons = screen.getAllByRole("button");
    const pencilButton = editButtons.find(btn => btn.querySelector("svg.lucide-pencil"));
    expect(pencilButton).toBeDefined();
    fireEvent.click(pencilButton!);

    // Verify Edit Modal is open
    expect(await screen.findByText(/Leave Policy Configuration/i)).toBeDefined();

    const approvalReqInput = screen.getByLabelText(/Approval Required/i) as HTMLInputElement;
    const docReqInput = screen.getByLabelText(/Requires Document/i) as HTMLInputElement;
    const negativeBalInput = screen.getByLabelText(/Negative Balance/i) as HTMLInputElement;

    expect(approvalReqInput.checked).toBe(false);
    expect(docReqInput.checked).toBe(false);
    expect(negativeBalInput.checked).toBe(false);

    // Click on the slider track of Approval Required
    const approvalTrack = approvalReqInput.nextElementSibling!;
    fireEvent.click(approvalTrack);
    expect(approvalReqInput.checked).toBe(true);

    // Click on the slider track of Requires Document
    const docTrack = docReqInput.nextElementSibling!;
    fireEvent.click(docTrack);
    expect(docReqInput.checked).toBe(true);

    // Click on the slider track of Negative Balance
    const negativeTrack = negativeBalInput.nextElementSibling!;
    fireEvent.click(negativeTrack);
    expect(negativeBalInput.checked).toBe(true);
  });
});

