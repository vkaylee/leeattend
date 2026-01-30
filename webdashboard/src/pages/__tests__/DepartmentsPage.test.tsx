import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import DepartmentsPage from "../DepartmentsPage";
import { BrowserRouter } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ToastProvider } from "../../components/ui/Toast";
import * as api from "../../lib/api";
import React from "react";

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
    t: (key: string) => key,
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
    (api.getCurrentWorkspaceId as any).mockReturnValue("test-ws-id");
    (api.fetchDepartments as any).mockResolvedValue([
      { id: "dept-1", name: "Engineering", description: "Tech stuff", manager_id: "user-1" }
    ]);
    (api.fetchUsers as any).mockResolvedValue([
      { id: "user-1", full_name: "Manager One", email: "manager1@example.com" }
    ]);
    (api.fetchWorkingPatterns as any).mockResolvedValue([
      { id: "pat-1", name: "Standard Pattern" }
    ]);
  });

  const renderWithProviders = (ui: React.ReactElement) => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <BrowserRouter>
            {ui}
          </BrowserRouter>
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders without workspace selected", () => {
    (api.getCurrentWorkspaceId as any).mockReturnValue(null);
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
    (api.createDepartment as any).mockResolvedValue({ id: "new-dept", name: "HR" });

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
    (api.updateDepartment as any).mockResolvedValue({ id: "dept-1", name: "Engineering Updated" });

    renderWithProviders(<DepartmentsPage />);
    
    // Wait for data to load
    await screen.findByText("Engineering");

    // Use title which we added during modernization
    const editBtn = await screen.findByTitle(/common.edit/i);
    await user.click(editBtn);

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
    (api.deleteDepartment as any).mockResolvedValue({});
    vi.spyOn(window, "confirm").mockReturnValue(true);

    renderWithProviders(<DepartmentsPage />);

    await screen.findByText("Engineering");
    
    const deleteBtn = await screen.findByTitle(/common.delete/i);
    await user.click(deleteBtn);

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => {
      expect(api.deleteDepartment).toHaveBeenCalledWith("test-ws-id", "dept-1");
    });
  });
});
