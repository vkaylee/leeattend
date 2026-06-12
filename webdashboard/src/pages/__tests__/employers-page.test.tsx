import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import i18n from "../../i18n/config";
import * as api from "../../lib/api";
import EmployersPage from "../settings/employers-page";

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchEmployers: vi.fn(),
  fetchEmployees: vi.fn(),
  createEmployer: vi.fn(),
  updateEmployer: vi.fn(),
  deleteEmployer: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
}));

describe("EmployersPage", () => {
  let queryClient: QueryClient;

  const mockEmployers = [
    {
      id: "employer-1",
      workspace_id: "test-ws-id",
      name: "Global Trade LLC",
      tax_code: "TX-998877",
      legal_address: "123 Business Rd, SF",
      business_type: "llc",
      registration_number: "REG-111",
      phone: "+15551234",
      email: "trade@example.com",
      website: "https://trade.example.com",
      incorporation_date: "2020-05-15",
      country_code: "USA",
      currency_code: "USD",
      created_at: "",
      updated_at: "",
    },
    {
      id: "employer-2",
      workspace_id: "test-ws-id",
      name: "Local Shop Corp",
      tax_code: "TX-112233",
      legal_address: "456 Market St, SF",
      business_type: "corporation",
      registration_number: "REG-222",
      phone: "+15555678",
      email: "shop@example.com",
      website: "https://shop.example.com",
      incorporation_date: "2018-10-20",
      country_code: "USA",
      currency_code: "USD",
      created_at: "",
      updated_at: "",
    }
  ];

  beforeEach(() => {
    vi.clearAllMocks();
    void i18n.changeLanguage("en");
    queryClient = new QueryClient({
      defaultOptions: {
        queries: {
          retry: false,
          gcTime: 0,
          staleTime: 0,
        },
      },
    });

    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue("test-ws-id");
    vi.mocked(api.fetchEmployers).mockResolvedValue(mockEmployers);
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [],
      total: 0,
      page: 1,
      limit: 1000,
    });
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders employers list and details in cards", async () => {
    renderWithProviders(<EmployersPage />);

    expect(await screen.findByText("Global Trade LLC")).toBeDefined();
    expect(screen.getByText("Local Shop Corp")).toBeDefined();

    // Check tax codes are rendered
    expect(screen.getByText(/TX-998877/)).toBeDefined();
    expect(screen.getByText(/TX-112233/)).toBeDefined();

    // Check new legal fields are rendered in card details
    expect(screen.getByText("REG-111")).toBeDefined();
    expect(screen.getByText("REG-222")).toBeDefined();
    expect(screen.getByText("2020-05-15")).toBeDefined();
    expect(screen.getAllByText(/USA \(USD\)/).length).toBe(2);
    expect(screen.getByText(/trade@example.com/)).toBeDefined();
    expect(screen.getByText(/shop@example.com/)).toBeDefined();
  });

  it("renders empty state when no employers exist", async () => {
    vi.mocked(api.fetchEmployers).mockResolvedValue([]);
    renderWithProviders(<EmployersPage />);

    expect(await screen.findByText(/No employers added yet|Chưa có đơn vị nào được thêm/i)).toBeDefined();
  });

  it("handles creating a new employer with all legal entity fields", async () => {
    const user = userEvent.setup();
    vi.mocked(api.createEmployer).mockResolvedValue({
      ...mockEmployers[0],
      id: "employer-new",
      name: "Newly Created Corp",
    });

    renderWithProviders(<EmployersPage />);

    const addButton = await screen.findByTestId("add-employer-btn");
    await user.click(addButton);

    // Verify modal is open
    expect(await screen.findByRole("dialog")).toBeDefined();

    // Fill out general section
    const nameInput = screen.getByLabelText(/Tên đơn vị|Tên pháp nhân|Name/i);
    await user.type(nameInput, "Newly Created Corp");

    const addressInput = screen.getByLabelText(/Địa chỉ pháp lý|Legal Address/i);
    await user.type(addressInput, "789 Enterprise Blvd");

    // Fill out legal section
    const taxInput = screen.getByLabelText(/Mã số thuế|Tax Code/i);
    await user.type(taxInput, "TX-555666");

    const typeSelect = screen.getByLabelText(/Loại hình doanh nghiệp|Business Type/i);
    await user.selectOptions(typeSelect, "corporation");

    const regInput = screen.getByLabelText(/Số đăng ký kinh doanh|Registration Number/i);
    await user.type(regInput, "REG-999");

    const incInput = screen.getByLabelText(/Ngày thành lập|Incorporation Date/i);
    await user.type(incInput, "2026-06-01");

    // Fill out contact & financial section
    const phoneInput = screen.getByLabelText(/Số điện thoại|Phone Number/i);
    await user.type(phoneInput, "+15559999");

    const emailInput = screen.getByLabelText(/Địa chỉ email|Email Address/i);
    await user.type(emailInput, "newcorp@example.com");

    const webInput = screen.getByLabelText(/Trang web|Website/i);
    await user.type(webInput, "https://newcorp.example.com");

    const countryInput = screen.getByLabelText(/Mã quốc gia/i);
    await user.type(countryInput, "VNM");

    const currencyInput = screen.getByLabelText(/Mã tiền tệ/i);
    await user.clear(currencyInput);
    await user.type(currencyInput, "VND");

    const submitBtn = screen.getByTestId("save-employer-btn");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(api.createEmployer).toHaveBeenCalledWith("test-ws-id", {
        name: "Newly Created Corp",
        tax_code: "TX-555666",
        legal_address: "789 Enterprise Blvd",
        business_type: "corporation",
        registration_number: "REG-999",
        incorporation_date: "2026-06-01",
        phone: "+15559999",
        email: "newcorp@example.com",
        website: "https://newcorp.example.com",
        country_code: "VNM",
        currency_code: "VND",
      });
    });
  });

  it("handles editing an existing employer", async () => {
    const user = userEvent.setup();
    vi.mocked(api.updateEmployer).mockResolvedValue({
      ...mockEmployers[0],
      name: "Updated Legal Trade LLC",
    });

    renderWithProviders(<EmployersPage />);

    // Find the first card (Global Trade LLC) and click edit
    const card = await screen.findByText("Global Trade LLC");
    const cardContainer = card.closest("div")!;
    const editButton = within(cardContainer).getByTitle(/Chỉnh sửa|Edit/i);
    await user.click(editButton);

    // Verify modal is open and values pre-populated
    expect(await screen.findByRole("dialog")).toBeDefined();
    
    const nameInput = screen.getByLabelText(/Tên đơn vị|Tên pháp nhân|Name/i);
    expect((nameInput as HTMLInputElement).value).toBe("Global Trade LLC");

    const regInput = screen.getByLabelText(/Số đăng ký kinh doanh|Registration Number/i);
    expect((regInput as HTMLInputElement).value).toBe("REG-111");

    // Modify values
    await user.clear(nameInput);
    await user.type(nameInput, "Updated Legal Trade LLC");

    await user.clear(regInput);
    await user.type(regInput, "REG-111-UPDATED");

    const submitBtn = screen.getByTestId("save-employer-btn");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(api.updateEmployer).toHaveBeenCalledWith("test-ws-id", "employer-1", {
        name: "Updated Legal Trade LLC",
        tax_code: "TX-998877",
        legal_address: "123 Business Rd, SF",
        business_type: "llc",
        registration_number: "REG-111-UPDATED",
        incorporation_date: "2020-05-15",
        phone: "+15551234",
        email: "trade@example.com",
        website: "https://trade.example.com",
        country_code: "USA",
        currency_code: "USD",
      });
    });
  });

  it("handles deleting an employer", async () => {
    const user = userEvent.setup();
    vi.mocked(api.deleteEmployer).mockResolvedValue(undefined);

    renderWithProviders(<EmployersPage />);

    const card = await screen.findByText("Global Trade LLC");
    const cardContainer = card.closest("div")!;
    const deleteButton = within(cardContainer).getByTitle(/Xóa|Delete/i);
    await user.click(deleteButton);

    // Confirm dialog
    const confirmButton = await screen.findByText(/Confirm|Xác nhận|common\.confirm/i);
    await user.click(confirmButton);

    await waitFor(() => {
      expect(api.deleteEmployer).toHaveBeenCalledWith("test-ws-id", "employer-1");
    });
  });
});
