import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent } from "@testing-library/react";
import React from "react";
import { describe, it, expect, vi } from "vitest";

import OvertimeRuleEditor from "../overtime-rule-editor";

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { children: React.ReactNode; to: string }) => <a href={to} {...props}>{children}</a>,
}));

vi.mock("@/lib/api/payroll", () => ({
  createOvertimePolicyVersion: vi.fn().mockResolvedValue({}),
  patchOvertimePolicyVersion: vi.fn().mockResolvedValue({}),
  publishOvertimePolicyVersion: vi.fn().mockResolvedValue({}),
  fetchOvertimePolicies: vi.fn().mockResolvedValue([]),
}));

vi.mock("@/components/ui/toast", () => ({
  useToast: () => ({ success: vi.fn(), error: vi.fn() }),
}));

const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });

const renderWithProviders = (ui: React.ReactElement) =>
  render(<QueryClientProvider client={queryClient}>{ui}</QueryClientProvider>);

describe("OvertimeRuleEditor (v2 versions)", () => {
  it("renders draft version rules, caps, and publish button", () => {
    renderWithProviders(
      <OvertimeRuleEditor
        policyId="pol-1"
        versions={[
          {
            id: "ver-1",
            policy_id: "pol-1",
            version_no: 1,
            status: "draft",
            row_version: 1,
            daily_cap_minutes: 240,
            monthly_cap_hours: "40",
            yearly_cap_hours: "200",
            rules: [{ priority: 50, day_type: "weekday", days_of_week: 127, multiplier: "1.50" }],
            created_by: "u-1",
            created_at: "2026-01-01T00:00:00Z",
            updated_at: "2026-01-01T00:00:00Z",
          },
        ]}
      />,
    );

    expect(screen.getByText("1.50x")).toBeDefined();
    expect(screen.getByText(/v1 · draft/)).toBeDefined();
    expect(screen.getByRole("button", { name: /Publish|Phát hành/i })).toBeDefined();
  });

  it("allows appending a rule to draft rules list", () => {
    renderWithProviders(<OvertimeRuleEditor policyId="pol-1" versions={[]} />);
    fireEvent.click(screen.getByRole("button", { name: /Add|Thêm/i }));
    expect(screen.getByText("1.50x")).toBeDefined();
  });
});
