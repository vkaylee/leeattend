import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

import SalarySetupPage from "@/pages/salary-management/salary-setup-page";

const fetchEmployee = vi.fn();
const fetchEmployeeEmployments = vi.fn();
let currentSearch: Record<string, string> = {};

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children }: { children: ReactNode }) => <>{children}</>,
  useNavigate: () => vi.fn(),
  useParams: () => ({}),
  useSearch: () => currentSearch,
}));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (_key: string, fallback?: string | { defaultValue?: string }) => typeof fallback === "string" ? fallback : fallback?.defaultValue ?? "translation" }),
}));
vi.mock("@/lib/api/employees", () => ({ fetchEmployee: (...args: unknown[]) => fetchEmployee(...args), fetchEmployeeEmployments: (...args: unknown[]) => fetchEmployeeEmployments(...args) }));
vi.mock("@/components/common/employee-search", () => ({ EmployeeSearch: () => <div aria-label="Employee search" /> }));
vi.mock("@/components/payroll/employee-salary-config", () => ({ default: ({ employmentId }: { employmentId: string }) => <div data-testid="employee-salary-config">{employmentId}</div> }));

function renderPage() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(<QueryClientProvider client={queryClient}><SalarySetupPage /></QueryClientProvider>);
}

describe("salary setup page", () => {
  it("asks HR to select an employee before loading salary data", () => {
    currentSearch = {};
    renderPage();
    expect(screen.getByRole("heading", { name: "Thiết lập lương" })).toBeDefined();
    expect(screen.getByLabelText("Employee search")).toBeDefined();
    expect(fetchEmployee).not.toHaveBeenCalled();
    expect(fetchEmployeeEmployments).not.toHaveBeenCalled();
  });

  it("renders explicit employment context before mounting salary configuration", async () => {
    currentSearch = { employee_id: "employee-1", employment_id: "employment-2" };
    fetchEmployee.mockResolvedValue({ id: "employee-1", employee_id_number: "EMP-001", full_name: "Employee A" });
    fetchEmployeeEmployments.mockResolvedValue([
      { id: "employment-1", status: "active", start_date: "2025-01-01", job_title: "Old role" },
      { id: "employment-2", status: "ended", start_date: "2026-01-01", end_date: "2026-06-30", job_title: "Selected role" },
    ]);

    renderPage();

    expect(await screen.findByText("Employee A")).toBeDefined();
    expect(await screen.findByTestId("employee-salary-config")).toHaveTextContent("employment-2");
  });
});
