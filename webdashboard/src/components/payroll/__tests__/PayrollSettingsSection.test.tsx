import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import PayrollSettingsSection from "../PayrollSettingsSection";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "@/lib/api/payroll";

// Mock dependencies
vi.mock("@/lib/api/payroll", () => ({
  fetchPayrollSettings: vi.fn(),
  updatePayrollSettings: vi.fn(),
}));

vi.mock("@/components/Toast", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}));

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

describe("PayrollSettingsSection", () => {
  const mockWsId = "ws-123";
  const mockSettings = {
    workspace_id: mockWsId,
    std_working_day_mode: "fixed",
    fixed_std_days: "26.00",
    working_days_per_week: 6,
    hours_per_day: "8.00",
    updated_at: new Date().toISOString(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchPayrollSettings as any).mockResolvedValue(mockSettings);
  });

  it("renders settings correctly and switches between modes", async () => {
    render(<PayrollSettingsSection wsId={mockWsId} />, { wrapper });

    await waitFor(() => {
      expect(screen.getByDisplayValue("26")).toBeInTheDocument();
    });

    // Switch to Actual Month mode
    const modeSelect = screen.getByDisplayValue(/Số ngày cố định/i);
    fireEvent.change(modeSelect, { target: { value: "actual_month" } });

    await waitFor(() => {
      expect(screen.getByText(/Số ngày làm việc\/tuần/i)).toBeInTheDocument();
    });
  });

  it("calls update API when save button is clicked", async () => {
    (api.updatePayrollSettings as any).mockResolvedValue(mockSettings);
    render(<PayrollSettingsSection wsId={mockWsId} />, { wrapper });

    await waitFor(() => screen.getByDisplayValue("26"));

    const saveBtn = screen.getByTestId("payroll-settings-save");
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.updatePayrollSettings).toHaveBeenCalled();
    });
  });
});
