import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import * as api from "../../lib/api";
import { Department, WorkingPattern, Employee, PaginatedResponse } from "../../lib/api/types";
import DepartmentsPage from "../departments-page";


// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
 
  useLocation: () => ({ pathname: "/", search: "", hash: "" }),
  Link: ({ to, children, ...props }: { to: string; children: React.ReactNode }) => <a href={to} {...props}>{children}</a>,
}));

// Mock the API library
vi.mock("../../lib/api", () => ({
  fetchDepartments: vi.fn(),
  createDepartment: vi.fn(),
  updateDepartment: vi.fn(),
  deleteDepartment: vi.fn(),
  getCurrentWorkspaceId: vi.fn(),
  fetchEmployees: vi.fn(),
  fetchWorkingPatterns: vi.fn(),
  fetchDepartmentPayrollConfigs: vi.fn(),
  setDepartmentPayrollConfig: vi.fn(),
}));

// Override i18n for this test to return keys
vi.mock('react-i18next', () => ({
 
  useTranslation: () => ({
    t: (key: string, options?: { defaultValue?: string }) => {
      if (options?.defaultValue) return options.defaultValue;
      return key;
    },
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'en',
    },
  }),
}));

describe("DepartmentsPage", () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
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
 
    vi.mocked(api.fetchDepartments).mockResolvedValue([
      { id: "dept-1", name: "Engineering", description: "Tech stuff", manager_id: "user-1", active_employee_count: 5 }
    ] as unknown as Department[]);
 
    vi.mocked(api.fetchEmployees).mockResolvedValue({
      data: [
        { id: "user-2", full_name: "Manager Two", employee_id_number: "EMP002", department_name: "HR" },
        { id: "user-1", full_name: "Manager One", employee_id_number: "EMP001", department_name: "Engineering" }
      ],
      total: 2,
      page: 1,
      limit: 1000
    } as unknown as PaginatedResponse<Employee>);
 
    vi.mocked(api.fetchWorkingPatterns).mockResolvedValue([
      { id: "pat-1", name: "Standard Pattern" }
    ] as unknown as WorkingPattern[]);
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

  it("renders without workspace selected", () => {
 
    vi.mocked(api.getCurrentWorkspaceId).mockReturnValue(null);
    renderWithProviders(<DepartmentsPage />);
    expect(screen.getByText("common.select_workspace")).toBeDefined();
  });

  it("renders departments list", async () => {
    renderWithProviders(<DepartmentsPage />);
    expect((await screen.findAllByText("Engineering"))[0]).toBeDefined();
    expect(screen.getAllByText("Tech stuff")[0]).toBeDefined();
    expect(screen.getAllByText("Manager One")[0]).toBeDefined();
  });

  it("handles creating a new department", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.createDepartment).mockResolvedValue({ id: "new-dept", name: "HR" } as unknown as Department);

    renderWithProviders(<DepartmentsPage />);

    await user.click(screen.getByText(/departments.add_btn/i));

    // Wait for modal to be visible
    expect(await screen.findByText(/departments.modal.add_title/i)).toBeDefined();

    const nameInput = await screen.findByLabelText(/departments.form.name_label/i);
    await user.type(nameInput, "Human Resources");

    await user.click(screen.getByText(/departments.modal.submit_btn/i));

    await waitFor(() => {
      expect(api.createDepartment).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
        name: "Human Resources"
      }));
    });
  });

  it("handles updating a department", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.updateDepartment).mockResolvedValue({ id: "dept-1", name: "Engineering Updated" } as unknown as Department);

    renderWithProviders(<DepartmentsPage />);
    
    // Wait for data to load
    await screen.findAllByText("Engineering");

    // Open actions dropdown menu and click Edit
    const actionsButtons = await screen.findAllByRole("button", { name: /departments.columns.actions/i });
    await user.click(actionsButtons[0]);
    const editButtons = await screen.findAllByText(/common.edit/i);
    await user.click(editButtons[0]);

    const nameInput = await screen.findByLabelText(/departments.form.name_label/i);
    await user.clear(nameInput);
    await user.type(nameInput, "Software Engineering");

    await user.click(screen.getByText(/common.update/i));

    await waitFor(() => {
      expect(api.updateDepartment).toHaveBeenCalledWith("test-ws-id", "dept-1", expect.objectContaining({
        name: "Software Engineering"
      }));
    });
  });

  it("handles deleting a department", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.deleteDepartment).mockResolvedValue(undefined);

    renderWithProviders(<DepartmentsPage />);

    await screen.findAllByText("Engineering");
    
    // Open actions dropdown menu and click Delete
    const actionsButtons = await screen.findAllByRole("button", { name: /departments.columns.actions/i });
    await user.click(actionsButtons[0]);
    const deleteButtons = await screen.findAllByText(/common.delete/i);
    await user.click(deleteButtons[0]);

    // Confirm the useConfirm modal
    const confirmModalButton = await screen.findByRole("button", { name: "Confirm" });
    fireEvent.click(confirmModalButton);

    await waitFor(() => {
      expect(api.deleteDepartment).toHaveBeenCalledWith("test-ws-id", "dept-1");
    });
  });

  it("sorts employees prioritizing current department members in edit modal", async () => {
    const user = userEvent.setup();
    renderWithProviders(<DepartmentsPage />);
    
    // Wait for departments to load
    await screen.findAllByText("Engineering");
    
    // Open edit modal via actions dropdown
    const actionsButtons = await screen.findAllByRole("button", { name: /departments.columns.actions/i });
    await user.click(actionsButtons[0]);
    const editButtons = await screen.findAllByText(/common.edit/i);
    await user.click(editButtons[0]);
    
    // Find the hidden select element used for the combobox
    // The Combobox is mocked in setup.tsx to render a select with the exact data-testid
    const managerSelect = await screen.findByTestId("dept-manager");
    
    // Get all options except the placeholder
    const options = Array.from(managerSelect.querySelectorAll("option")).filter(opt => opt.value !== "");
    
    // Original array has "Manager Two" (HR) then "Manager One" (Engineering).
    // Because we are editing "Engineering", "Manager One" should be sorted first.
    expect(options[0].value).toBe("user-1");
    expect(options[0].text).toBe("MANAGER ONE");
    expect(options[1].value).toBe("user-2");
    expect(options[1].text).toBe("MANAGER TWO");
  });
});
