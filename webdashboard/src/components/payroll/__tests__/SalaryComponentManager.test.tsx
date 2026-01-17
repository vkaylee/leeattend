import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import SalaryComponentManager from "../SalaryComponentManager";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import * as api from "@/lib/api/payroll";

vi.mock("@/lib/api/payroll", () => ({
  fetchSalaryComponents: vi.fn(),
  createSalaryComponent: vi.fn(),
}));

vi.mock("@/components/Toast", () => ({
  useToast: () => ({
    success: vi.fn(),
    error: vi.fn(),
  }),
}));

const queryClient = new QueryClient({
  defaultOptions: { queries: { retry: false } },
});

const wrapper = ({ children }: { children: React.ReactNode }) => (
  <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
);

describe("SalaryComponentManager", () => {
  const mockWsId = "ws-123";
  const mockComponents = [
    { id: "1", name: "Lương cơ bản", code: "BASE", is_base_for_ot: true, is_taxable: true },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    (api.fetchSalaryComponents as any).mockResolvedValue(mockComponents);
  });

  it("renders list of components", async () => {
    render(<SalaryComponentManager wsId={mockWsId} />, { wrapper });

    await waitFor(() => {
      expect(screen.getByText("Lương cơ bản")).toBeInTheDocument();
      expect(screen.getByText("BASE")).toBeInTheDocument();
    });
  });

  it("opens modal and submits new component", async () => {
    (api.createSalaryComponent as any).mockResolvedValue({ id: "2" });
    render(<SalaryComponentManager wsId={mockWsId} />, { wrapper });

    const addBtn = screen.getByText("Thêm");
    fireEvent.click(addBtn);

    expect(screen.getByPlaceholderText(/Phụ cấp ăn trưa/i)).toBeInTheDocument();

    fireEvent.change(screen.getByPlaceholderText(/Phụ cấp ăn trưa/i), { target: { value: "Gas" } });
    fireEvent.change(screen.getByPlaceholderText("MEAL"), { target: { value: "GAS" } });

    const saveBtn = screen.getByText("Lưu");
    fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(api.createSalaryComponent).toHaveBeenCalledWith(mockWsId, expect.objectContaining({
        name: "Gas",
        code: "GAS"
      }));
    });
  });
});
