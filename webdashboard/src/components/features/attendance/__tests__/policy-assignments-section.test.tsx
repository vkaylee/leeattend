import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi, beforeEach } from "vitest";

import * as payrollApi from "@/lib/api/payroll";

import { PolicyAssignmentsSection } from "../policy-assignments-section";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (k: string, d?: string) => d ?? k }),
}));

vi.mock("@/lib/api/payroll", () => ({
  fetchWorkPolicyGroups: vi.fn(),
}));

describe("PolicyAssignmentsSection (Work Policy v2)", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  });

  const renderSection = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <PolicyAssignmentsSection />
      </QueryClientProvider>,
    );

  it("shows empty state when no work policy groups exist", async () => {
    vi.mocked(payrollApi.fetchWorkPolicyGroups).mockResolvedValue([]);
    renderSection();
    expect(await screen.findByText(/No work policy groups configured/i)).toBeInTheDocument();
  });

  it("renders work policy groups with membership counts", async () => {
    vi.mocked(payrollApi.fetchWorkPolicyGroups).mockResolvedValue([
      {
        id: "g-1",
        code: "STD-OFFICE",
        name: "Standard Office",
        active_membership_count: 12,
        current_published: { version_no: 2 } as unknown as never,
        created_at: "2026-01-01T00:00:00Z",
        updated_at: "2026-01-01T00:00:00Z",
      },
    ]);
    renderSection();
    expect(await screen.findByText("STD-OFFICE")).toBeInTheDocument();
    expect(screen.getByText("Standard Office")).toBeInTheDocument();
    expect(screen.getByText(/12 assigned/)).toBeInTheDocument();
  });
});
