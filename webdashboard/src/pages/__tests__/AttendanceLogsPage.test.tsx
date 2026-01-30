import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import AttendanceLogsPage from "../AttendanceLogsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import React from "react";

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchRawAttendanceLogs: vi.fn(),
  fetchEmployees: vi.fn(),
  fetchWorkspace: vi.fn(),
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

describe("AttendanceLogsPage", () => {
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
    (api.fetchWorkspace as any).mockResolvedValue({ id: "test-ws-id", name: "Test Workspace" });
    (api.fetchEmployees as any).mockResolvedValue({
      data: [
        { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001", workspace_id: "ws-1", is_active: true, hire_date: "2025-01-01", created_at: "2025-01-01T00:00:00Z", updated_at: "2025-01-01T00:00:00Z" }
      ],
      total: 1,
      page: 1,
      limit: 50,
    });
    (api.fetchRawAttendanceLogs as any).mockResolvedValue({
      data: [
        { id: "log-1", employee_id: "emp-1", employee_name: "John Doe", punch_time: new Date().toISOString(), source: "DEVICE", device_id: "dev-1" }
      ],
      total: 1,
      page: 1,
      page_size: 50
    });
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

  it("renders the page and logs", async () => {
    renderWithProviders(<AttendanceLogsPage />);
    expect(await screen.findByText(/John Doe/i)).toBeDefined();
  });
});
