import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import React from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { Employee } from "@/lib/api";

import { ToastProvider } from "@/components/ui/toast";
import * as api from "@/lib/api";

import EmployeeFormModal from "../employee-form-modal";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { changeLanguage: () => Promise.resolve(), language: "en" },
  }),
}));

vi.mock("@/lib/api", () => ({
  fetchEmployers: vi.fn(() => Promise.resolve([])),
  fetchEmployeeEmployments: vi.fn(() => Promise.resolve([{
    id: "employment-1",
    employee_id: "emp-1",
    employer_id: "employer-1",
    primary_location_id: "location-1",
    start_date: "2024-06-15",
    status: "active",
  }])),
  updateEmployee: vi.fn(),
  updateEmployeeEmployment: vi.fn(() => Promise.resolve({})),
}));

const employee: Employee = {
  id: "emp-1",
  full_name: "Alice Nguyen",
  employee_id_number: "EMP-001",
  workspace_id: "ws-1",
  user_id: "user-abc",
  created_at: "2024-06-15T00:00:00Z",
  updated_at: "2024-06-15T00:00:00Z",
  tags: [],
};

describe("EmployeeFormModal", () => {
  const onClose = vi.fn();
  let queryClient: QueryClient;

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  });

  function renderModal() {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <EmployeeFormModal editingEmployee={employee} isOpen onClose={onClose} />
        </ToastProvider>
      </QueryClientProvider>,
    );
  }

  it("renders edit fields without account-link controls", async () => {
    renderModal();

    expect(screen.getByTestId("employees-form-name-input")).toHaveValue("Alice Nguyen");
    expect(screen.getByTestId("employees-form-id-input")).toHaveValue("EMP-001");
    await waitFor(() => expect(screen.queryByTestId("employees-form-user-combobox")).toBeNull());
  });

  it("updates profile without sending user_id", async () => {
    vi.mocked(api.updateEmployee).mockResolvedValue(employee);
    renderModal();

    await userEvent.setup().click(screen.getByTestId("save-employee-btn"));

    await waitFor(() => {
      expect(api.updateEmployee).toHaveBeenCalledWith("emp-1", {
        full_name: "Alice Nguyen",
        employee_id_number: "EMP-001",
      });
    });
    expect(vi.mocked(api.updateEmployee).mock.calls[0]?.[1]).not.toHaveProperty("user_id");
  });

  it("closes when cancel is clicked", async () => {
    renderModal();
    await userEvent.setup().click(screen.getByText(/employees.form.cancel/i));
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
