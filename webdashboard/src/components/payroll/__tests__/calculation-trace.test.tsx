import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";

import { describe, expect, it, vi } from "vitest";

import { fetchPayrollCalculationRuns } from "@/lib/api/payroll";

import { CalculationTrace } from "../calculation-trace";

vi.mock("@/lib/api/payroll", () => ({
  fetchPayrollCalculationRuns: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("CalculationTrace", () => {
  function renderTrace() {
    return render(
      <QueryClientProvider client={new QueryClient()}>
        <CalculationTrace employmentId="employment-1" month={10} year={2026} />
      </QueryClientProvider>,
    );
  }

  it("loads trace only after expansion and renders steps", async () => {
    vi.mocked(fetchPayrollCalculationRuns).mockResolvedValueOnce([{
      id: "run-1",
      employment_id: "employment-1",
      period_year: 2026,
      period_month: 10,
      engine_version: "formula-v1",
      formula_version_id: "formula-version-1",
      payslip_id: "payslip-1",
      input_snapshot: { source: "test" },
      output_snapshot: { result: "redacted" },
      calculation_trace: [{ step: "formula", rule: "kpi.bonus", expression: "actual * rate", result: "100" }],
      created_at: "2026-10-31T00:00:00Z",
    }]);

    renderTrace();
    expect(fetchPayrollCalculationRuns).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button"));

    await waitFor(() => {
      expect(fetchPayrollCalculationRuns).toHaveBeenCalledWith("employment-1", 2026, 10);
      expect(screen.getByText("actual * rate")).toBeInTheDocument();
    });
    expect(screen.getByText(/formula-v1/)).toBeInTheDocument();
    expect(screen.getByText(/formula-version-1/)).toBeInTheDocument();
    expect(screen.getByText("payroll.calculation_trace.input_snapshot")).toBeInTheDocument();
    expect(screen.getByText("payroll.calculation_trace.output_snapshot")).toBeInTheDocument();
  });

  it("warns when trace payload is partial or malformed", async () => {
    vi.mocked(fetchPayrollCalculationRuns).mockResolvedValueOnce([{
      id: "run-2", employment_id: "employment-1", period_year: 2026, period_month: 10,
      engine_version: "formula-v1", input_snapshot: {}, output_snapshot: {},
      calculation_trace: [{ step: "formula", result: "100" }, { unexpected: true }],
      created_at: "2026-10-31T00:00:00Z",
    }]);
    renderTrace();
    fireEvent.click(screen.getByRole("button"));
    expect(await screen.findByText("payroll.calculation_trace.partial")).toBeInTheDocument();
    expect(screen.getByText("100")).toBeInTheDocument();
  });

  it("warns when provenance metadata is incomplete", async () => {
    vi.mocked(fetchPayrollCalculationRuns).mockResolvedValueOnce([{
      id: "run-3", employment_id: "employment-1", period_year: 2026, period_month: 10,
      engine_version: "formula-v1", input_snapshot: null, output_snapshot: {},
      calculation_trace: [], created_at: "2026-10-31T00:00:00Z",
    }]);
    renderTrace();
    fireEvent.click(screen.getByRole("button"));
    expect(await screen.findByText("payroll.calculation_trace.provenance_incomplete")).toBeInTheDocument();
  });

  it("renders empty state", async () => {
    vi.mocked(fetchPayrollCalculationRuns).mockResolvedValueOnce([]);
    renderTrace();
    fireEvent.click(screen.getByRole("button"));
    expect(await screen.findByText("payroll.calculation_trace.empty")).toBeInTheDocument();
  });
});
