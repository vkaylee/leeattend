import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AttendanceAdjustmentsPage from "../AttendanceAdjustmentsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/Toast";
import * as api from "../../lib/api";
import React from "react";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchWorkspaceAttendanceAdjustments: vi.fn(),
  fetchEmployees: vi.fn(),
  approveAttendanceAdjustment: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
}));

// Override i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("AttendanceAdjustmentsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    (api.fetchEmployees as any).mockResolvedValue({
      data: [
        { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, hire_date: "2025-01-01", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }
      ],
      total: 1,
      page: 1,
      limit: 50,
    });
    (api.fetchWorkspaceAttendanceAdjustments as any).mockResolvedValue([
      { 
        id: "adj-1", 
        employee_id: "emp-1", 
        requested_date: "2026-01-01", 
        requested_punch_in: "08:00", 
        requested_punch_out: "17:00", 
        status: "pending", 
        reason: "Forgot to punch" 
      }
    ]);
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            {ui}
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders list of adjustment requests", async () => {
    renderWithProviders(<AttendanceAdjustmentsPage />);
    expect(await screen.findByText(/John Doe/i)).toBeDefined();
    expect(screen.getByText(/Forgot to punch/i)).toBeDefined();
  });

  it("handles approve action", async () => {
    const user = userEvent.setup();
    (api.approveAttendanceAdjustment as any).mockResolvedValue({});

    renderWithProviders(<AttendanceAdjustmentsPage />);
    
    const approveBtn = await screen.findByTitle(/common.approve/i);
    await user.click(approveBtn);

    await waitFor(() => {
      expect(api.approveAttendanceAdjustment).toHaveBeenCalledWith("test-ws-id", "adj-1", { status: "approved" });
    });
  });
});
