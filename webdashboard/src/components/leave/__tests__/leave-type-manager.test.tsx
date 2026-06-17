import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";

import { vi, describe, it, expect, beforeEach } from "vitest";

import * as api from "@/lib/api/leaves";

import { LeaveTypeManager } from "../leave-type-manager";

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
  const wsId = "test-ws-id";
  
  beforeEach(() => {
    vi.clearAllMocks();
    queryClient.clear();
  });

  it("renders empty state when no leave types exist", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager wsId={wsId} />
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
        <LeaveTypeManager wsId={wsId} />
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
        <LeaveTypeManager wsId={wsId} />
      </QueryClientProvider>
    );

    // Verify leave type card renders basic info
    expect(await screen.findByRole("heading", { name: /Annual Leave/i })).toBeDefined();

    // Verify configurations are rendered as badges on the card
    expect(screen.getByText(/Approval/i)).toBeDefined();
    expect(screen.getByText(/Document/i)).toBeDefined();
    expect(screen.getByText(/Negative Balance/i)).toBeDefined();
  });


  it("shows template gallery immediately when opening add modal", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager wsId={wsId} />
      </QueryClientProvider>
    );

    // 1. Click Get Started to open Add Modal
    fireEvent.click(await screen.findByText(/Get Started/i));
    
    // 2. Verify Step 1: Template Gallery is shown by default
    expect(await screen.findByText(/Choose a Template/i)).toBeDefined();
    expect(await screen.findByRole("heading", { name: /Annual Leave/i })).toBeDefined();
    expect(screen.getByText(/Mental Health Day/i)).toBeDefined();
  });

  it("applies a template and moves to customization step", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager wsId={wsId} />
      </QueryClientProvider>
    );

    // Open modal (Step 1: Gallery)
    fireEvent.click(await screen.findByText(/Get Started/i));
    
    // Select "Mental Health Day" template
    const templateCard = await screen.findByRole("heading", { name: /Mental Health Day/i });
    fireEvent.click(templateCard.closest('button')!);
    
    // 3. Verify Step 2: Customization Form is shown
    expect(await screen.findByText(/Customize Leave Type/i)).toBeDefined();
    expect(screen.getByText(/Refining Mental Health Day/i)).toBeDefined();

    const nameInput = screen.getByPlaceholderText(/e.g. Annual Leave/i);
    const codeInput = screen.getByPlaceholderText(/e.g. AL, SL, UL/i);
    
    expect((nameInput as HTMLInputElement).value).toBe("Mental Health Day");
    expect((codeInput as HTMLInputElement).value).toBe("MHD");
  });

  it("allows toggling Paid Status in Add Template form via the slider track", async () => {
    vi.mocked(api.fetchLeaveTypes).mockResolvedValue([]);

    render(
      <QueryClientProvider client={queryClient}>
        <LeaveTypeManager wsId={wsId} />
      </QueryClientProvider>
    );

    // Open modal (Step 1: Gallery)
    fireEvent.click(await screen.findByText(/Get Started/i));
    
    // Select "Mental Health Day" template
    const templateCard = await screen.findByRole("heading", { name: /Mental Health Day/i });
    fireEvent.click(templateCard.closest('button')!);
    
    // Customize Leave Type form shown
    expect(await screen.findByText(/Customize Leave Type/i)).toBeDefined();

    // Verify Paid Status checkbox
    const paidInput = screen.getByLabelText(/Paid Status/i) as HTMLInputElement;
    const initialPaidState = paidInput.checked;

    // Click on the track (next element sibling of the checkbox) which is inside the label wrapper
    const paidTrack = paidInput.nextElementSibling!;
    fireEvent.click(paidTrack);

    // Verify it toggled
    expect(paidInput.checked).toBe(!initialPaidState);
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
        <LeaveTypeManager wsId={wsId} />
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

