import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api/employees";

import EmploymentPhaseManager from "../employment-phase-manager";


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
    const mockPhases = [
    { id: "1", name: "Thử việc 2 tháng", phase_type: "probation", default_duration_days: 60, is_active: true },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0, staleTime: 0 } },
    });
 
    vi.mocked(api.fetchEmploymentPhases).mockResolvedValue(mockPhases as unknown as never);
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
    renderWithProviders(<EmploymentPhaseManager   />);

    expect(await screen.findByText(/Thử việc 2 tháng/i)).toBeInTheDocument();
    expect(screen.getAllByText(/probation/i)[0]).toBeInTheDocument();
    expect(screen.getByText(/ACTIVE/i)).toBeInTheDocument();
  });

  it("opens modal and submits new employment phase", async () => {
 
    vi.mocked(api.createEmploymentPhase).mockResolvedValue({ id: "2" } as unknown as never);
    renderWithProviders(<EmploymentPhaseManager   />);

    const addButton = screen.getByText(/Thêm Cột mốc/i);
    fireEvent.click(addButton);

    const nameInput = await screen.findByLabelText(/Tên Cột mốc/i);
    fireEvent.change(nameInput, { target: { value: "Internship" } });
    
    const typeSelect = screen.getByLabelText(/Type/i);
    fireEvent.change(typeSelect, { target: { value: "internship" } });

    const saveButton = await screen.findByRole("button", { name: /common.save/i });
    fireEvent.click(saveButton);

    await waitFor(() => {
      expect(api.createEmploymentPhase).toHaveBeenCalledWith(expect.objectContaining({
        name: "Internship",
        phase_type: "internship"
      }));
    });
  });
});
