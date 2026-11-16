import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";

import { describe, expect, it, vi } from "vitest";

import { ApiError } from "@/lib/api/errors";
import { createPayrollFormulaVersion, fetchPayrollFormulaVersions } from "@/lib/api/payroll";

import { PayrollFormulaHistory } from "../payroll-formula-history";

vi.mock("@/lib/api/payroll", () => ({
  createPayrollFormulaVersion: vi.fn(),
  fetchPayrollFormulaRules: vi.fn(),
  fetchPayrollFormulaVersions: vi.fn(),
  publishPayrollFormulaRule: vi.fn(),
  retirePayrollFormulaRule: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string, fallback?: string) => fallback ?? key }),
}));

describe("PayrollFormulaHistory", () => {
  function renderHistory() {
    return render(
      <QueryClientProvider client={new QueryClient()}>
        <PayrollFormulaHistory ruleId="rule-1" />
      </QueryClientProvider>,
    );
  }

  it("creates a draft version and refreshes history", async () => {
    vi.mocked(fetchPayrollFormulaVersions).mockResolvedValueOnce([]).mockResolvedValueOnce([{
      version_id: "version-2",
      rule_id: "rule-1",
      version: 2,
      expression: "actual * rate",
      effective_from: "2026-11-01",
      status: "draft",
      created_by: "actor-1",
      created_at: "2026-10-31T00:00:00Z",
    }]);
    vi.mocked(createPayrollFormulaVersion).mockResolvedValueOnce({
      version_id: "version-2",
      rule_id: "rule-1",
      version: 2,
      expression: "actual * rate",
      effective_from: "2026-11-01",
      status: "draft",
      created_by: "actor-1",
      created_at: "2026-10-31T00:00:00Z",
    });

    renderHistory();
    fireEvent.change(await screen.findByLabelText("Expression"), { target: { value: "actual * rate" } });
    fireEvent.change(screen.getByLabelText("Effective from"), { target: { value: "2026-11-01" } });
    fireEvent.click(screen.getByRole("button", { name: "Create draft version" }));

    await waitFor(() => expect(createPayrollFormulaVersion).toHaveBeenCalledWith("rule-1", {
      expression: "actual * rate",
      effective_from: "2026-11-01",
    }));
    expect(await screen.findByText("v2 · draft")).toBeInTheDocument();
  });

  it("shows conflict error without clearing draft input", async () => {
    vi.mocked(fetchPayrollFormulaVersions).mockResolvedValueOnce([]);
    vi.mocked(createPayrollFormulaVersion).mockRejectedValueOnce(new ApiError(409, "Conflict", "CONFLICT"));

    renderHistory();
    const expression = await screen.findByLabelText("Expression");
    fireEvent.change(expression, { target: { value: "actual * rate" } });
    fireEvent.change(screen.getByLabelText("Effective from"), { target: { value: "2026-11-01" } });
    fireEvent.click(screen.getByRole("button", { name: "Create draft version" }));

    expect(await screen.findByText("Formula version conflicts with an existing version.")).toBeInTheDocument();
    expect(expression).toHaveValue("actual * rate");
  });
});
