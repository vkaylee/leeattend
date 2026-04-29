import React from "react";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/Toast";

import * as api from "@/lib/api/employees";

import EmploymentPhaseManager from "../EmploymentPhaseManager";


// Mock API
vi.mock("@/lib/api/employees", () => ({
  fetchEmploymentPhases: vi.fn(),
  createEmploymentPhase: vi.fn(),
}));

// Standard override for i18n
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("EmploymentPhaseManager", () => {
  let queryClient: QueryClient;
  const mockWsId = "ws-123";
  const mockPhases = [
    { id: "1", name: "Thử việc 2 tháng", phase_type: "probation", default_duration_days: 60, is_active: true },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
    (api.fetchEmploymentPhases as any).mockResolvedValue(mockPhases);
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders list of employment phases", async () => {
    renderWithProviders(<EmploymentPhaseManager wsId={mockWsId} />);

    expect(await screen.findByText(/Thử việc 2 tháng/i)).toBeInTheDocument();
    expect(screen.getAllByText(/probation/i)[0]).toBeInTheDocument();
    expect(screen.getByText(/ACTIVE/i)).toBeInTheDocument();
  });

  it("opens modal and submits new employment phase", async () => {
    (api.createEmploymentPhase as any).mockResolvedValue({ id: "2" });
    renderWithProviders(<EmploymentPhaseManager wsId={mockWsId} />);

    const addBtn = screen.getByText(/common.add/i);
    fireEvent.click(addBtn);

    const nameInput = await screen.findByLabelText(/common.name/i);
    fireEvent.change(nameInput, { target: { value: "Internship" } });
    
    const typeSelect = screen.getByLabelText(/Type/i);
    fireEvent.change(typeSelect, { target: { value: "internship" } });

    const saveBtn = await screen.findByRole("button", { name: /common.save/i });
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.createEmploymentPhase).toHaveBeenCalledWith(mockWsId, expect.objectContaining({
        name: "Internship",
        phase_type: "internship"
      }));
    });
  });
});
