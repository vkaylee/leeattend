import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tanstack/react-router", () => ({
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
