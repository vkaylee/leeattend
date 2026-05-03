import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import * as api from "../../lib/api";
import { Department, WorkingPattern } from "../../lib/api/types";
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
  fetchUsers: vi.fn(),
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
 
    vi.mocked(api.fetchUsers).mockResolvedValue([
      { id: "user-1", full_name: "Manager One", email: "manager1@example.com" }
    ]);
 
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
    expect(await screen.findByText("Engineering")).toBeDefined();
    expect(screen.getByText("Tech stuff")).toBeDefined();
    expect(screen.getByText("Manager One")).toBeDefined();
  });

  it("handles creating a new department with working pattern", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.createDepartment).mockResolvedValue({ id: "new-dept", name: "HR" } as unknown as Department);

    renderWithProviders(<DepartmentsPage />);

    await user.click(screen.getByText(/departments.add_btn/i));

    // Wait for modal to be visible
    expect(await screen.findByText(/departments.modal.add_title/i)).toBeDefined();

    const nameInput = await screen.findByLabelText(/departments.form.name_label/i);
    await user.type(nameInput, "Human Resources");
    
    // Select pattern via mock select (which is what Combobox is in tests)
    const mockSelect = await screen.findByTestId("dept-pattern");
    await user.selectOptions(mockSelect, "pat-1");

    await user.click(screen.getByText(/departments.modal.submit_btn/i));

    await waitFor(() => {
      expect(api.createDepartment).toHaveBeenCalledWith("test-ws-id", expect.objectContaining({
        name: "Human Resources",
        default_working_pattern_id: "pat-1"
      }));
    });
  });

  it("handles updating a department", async () => {
    const user = userEvent.setup();
 
    vi.mocked(api.updateDepartment).mockResolvedValue({ id: "dept-1", name: "Engineering Updated" } as unknown as Department);

    renderWithProviders(<DepartmentsPage />);
    
    // Wait for data to load
    await screen.findByText("Engineering");

    // Use title which we added during modernization
    const editButton = await screen.findByTitle(/common.edit/i);
    await user.click(editButton);

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
 
    vi.mocked(api.deleteDepartment).mockResolvedValue({} as unknown as void);

    renderWithProviders(<DepartmentsPage />);

    await screen.findByText("Engineering");
    
    const deleteButton = await screen.findByTitle(/common.delete/i);
    await user.click(deleteButton);

    // Confirm the useConfirm modal
    const confirmModalButton = await screen.findByRole("button", { name: "Confirm" });
    fireEvent.click(confirmModalButton);

    await waitFor(() => {
      expect(api.deleteDepartment).toHaveBeenCalledWith("test-ws-id", "dept-1");
    });
  });
});
