import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import type { ReactNode } from "react";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tanstack/react-router", () => ({
  Link: ({ children }: { children: ReactNode }) => <>{children}</>,
  useNavigate: () => vi.fn(),
  useSearch: () => ({}),
}));
vi.mock("@/lib/api/salary-management", () => ({ fetchSalaryManagement: vi.fn() }));
vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (_key: string, options?: { defaultValue?: string }) => options?.defaultValue ?? "translation" }) }));

import { fetchSalaryManagement } from "@/lib/api/salary-management";
import SalaryManagementPage from "@/pages/salary-management-page";

function renderPage() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(<QueryClientProvider client={queryClient}><SalaryManagementPage /></QueryClientProvider>);
}

describe("salary-management page", () => {
  it("renders accessible filters and loading state", () => {
    vi.mocked(fetchSalaryManagement).mockReturnValue(new Promise(() => undefined));
    renderPage();
    expect(screen.getByRole("textbox", { name: "Search employee code or name" })).toBeDefined();
    expect(screen.getByLabelText("Effective on")).toBeDefined();
    expect(screen.getByRole("status", { name: "Loading salary records" })).toBeDefined();
  });

  it("renders configured component amount and effective date", async () => {
    vi.mocked(fetchSalaryManagement).mockResolvedValue({
      as_of: "2026-11-12",
      data: [{
        amount: "5000.00",
        component_code: "BASE",
        component_definition_id: "definition-1",
        component_id: "component-1",
        component_name: "Base salary",
        department_id: null,
        department_name: "Engineering",
        effective_date: "2026-10-01",
        employee_id: "employee-1",
        employee_id_number: "EMP-001",
        employee_name: "Employee A",
        employer_id: "employer-1",
        employer_name: "Legal Entity",
        employment_end_date: null,
        employment_id: "employment-1",
        employment_start_date: "2026-01-01",
        employment_status: "active",
        employment_type: "full_time",
        job_title: "Engineer",
        salary_status: "current",
      }],
      limit: 25,
      page: 1,
      total: 1,
    });
    renderPage();
    expect(await screen.findByText("Base salary")).toBeDefined();
    expect(screen.getByText("5000.00")).toBeDefined();
    expect(screen.getByText("2026-10-01")).toBeDefined();
  });

  it("renders empty state when API returns no records", async () => {
    vi.mocked(fetchSalaryManagement).mockResolvedValue({ as_of: "2026-11-12", data: [], limit: 25, page: 1, total: 0 });
    renderPage();
    expect(await screen.findByText("No salary records")).toBeDefined();
  });

  it("defaults status filter to active while API defaults to active", async () => {
    vi.mocked(fetchSalaryManagement).mockReturnValue(new Promise(() => undefined));
    renderPage();

    expect(screen.getByRole("combobox", { name: "Status" })).toHaveValue("active");
  });

  it("renders API failure with retry action", async () => {
    vi.mocked(fetchSalaryManagement).mockRejectedValue(new Error("unavailable"));
    renderPage();
    expect(await screen.findByText("Salary records could not be loaded.")).toBeDefined();
    expect(screen.getByRole("button", { name: "Retry" })).toBeDefined();
  });
});
