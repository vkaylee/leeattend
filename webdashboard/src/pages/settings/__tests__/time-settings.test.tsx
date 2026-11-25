import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "@/components/ui/toast";

import TimeSettings from "../time-settings";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (k: string, d?: string) => d ?? k }),
}));

vi.mock("@/lib/api", () => ({
  reprocessAllAttendance: vi.fn(),
  checkReprocessStatus: vi.fn(),
}));

describe("TimeSettings (Work Policy v2)", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  });

  it("renders Work Policy v2 pointer and bulk attendance reprocess section", () => {
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <TimeSettings />
        </ToastProvider>
      </QueryClientProvider>,
    );
    expect(screen.getByText(/Time & Attendance Settings/i)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: /Manage Work Policies/i })).toBeInTheDocument();
    expect(screen.getByText(/Bulk Attendance Reprocess/i)).toBeInTheDocument();
  });
});
