import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { fetchPayrollFormulaRules, publishPayrollFormulaRule, retirePayrollFormulaRule, fetchPayrollFormulaVersions } from "@/lib/api/payroll";

import { PayrollFormulaHistory } from "../payroll-formula-history";

vi.mock("@/lib/api/payroll", () => ({
  createPayrollFormulaVersion: vi.fn(),
  fetchPayrollFormulaRules: vi.fn(),
  fetchPayrollFormulaVersions: vi.fn(),
  publishPayrollFormulaRule: vi.fn(),
  retirePayrollFormulaRule: vi.fn(),
}));

vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (_key: string, fallback?: string) => fallback ?? _key }) }));

describe("PayrollFormulaHistory actions", () => {
  it("publishes drafts and retires published versions", async () => {
    vi.mocked(fetchPayrollFormulaRules).mockResolvedValue([{ id: "rule-1", code: "kpi.bonus", name: "KPI Bonus", component_code: "KPI_BONUS", version_id: "published-1", version: 1, expression: "actual", effective_from: "2026-01-01", status: "published" } as never]);
    vi.mocked(fetchPayrollFormulaVersions).mockResolvedValue([
      { version_id: "draft-1", rule_id: "rule-1", version: 2, expression: "actual", effective_from: "2026-11-01", status: "draft", created_by: "actor", created_at: "2026-10-01" },
      { version_id: "published-1", rule_id: "rule-1", version: 1, expression: "actual", effective_from: "2026-01-01", status: "published", created_by: "actor", created_at: "2026-01-01" },
    ]);
    vi.mocked(publishPayrollFormulaRule).mockResolvedValueOnce({} as never);
    vi.mocked(retirePayrollFormulaRule).mockResolvedValueOnce({} as never);

    render(<QueryClientProvider client={new QueryClient()}><PayrollFormulaHistory ruleId="rule-1" /></QueryClientProvider>);
    fireEvent.click(await screen.findByRole("button", { name: "Publish" }));
    await waitFor(() => expect(publishPayrollFormulaRule).toHaveBeenCalledWith("rule-1"));
    fireEvent.click(screen.getByRole("button", { name: "Retire" }));
    await waitFor(() => expect(retirePayrollFormulaRule).toHaveBeenCalledWith("rule-1"));
  });
});
