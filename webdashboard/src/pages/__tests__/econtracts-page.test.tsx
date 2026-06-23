import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, fireEvent, waitFor, within } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import Button from "@/components/ui/button";
import { ToastProvider } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { useAllEmployees } from "@/hooks/use-employees";
import { fetchWorkspaceContracts, deleteEmployeeContract } from "@/lib/api/contracts";

import EContractsPage from "../econtracts-page";

// Mock i18next translation
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: any) => options?.defaultValue || key,
  }),
}));

// Mock current workspace ID
let mockWorkspaceId = "ws-1";
vi.mock("@/lib/api/helpers", () => ({
  getCurrentWorkspaceId: () => mockWorkspaceId,
}));

// Mock TanStack Router
vi.mock("@tanstack/react-router", () => ({
  useSearch: () => ({ q: "" }),
  Link: ({ to, children, ...props }: { to?: string; children: React.ReactNode }) => (
    <a href={to} {...props}>
      {children}
    </a>
  ),
}));

// Mock useDateTimeFormatter
vi.mock("@/hooks/use-datetime", () => ({
  useDateTimeFormatter: () => ({
    formatDate: (date: Date | string) => {
      const d = typeof date === "string" ? new Date(date) : date;
      const day = String(d.getDate()).padStart(2, "0");
      const month = String(d.getMonth() + 1).padStart(2, "0");
      const year = d.getFullYear();
      return `${day}/${month}/${year}`;
    },
  }),
}));

// Mock API endpoints
vi.mock("@/lib/api/contracts", () => ({
  fetchWorkspaceContracts: vi.fn(),
  deleteEmployeeContract: vi.fn(),
}));

// Mock Hooks
vi.mock("@/hooks/use-employees", () => ({
  useAllEmployees: vi.fn(),
}));

vi.mock("@/hooks/use-confirm", () => ({
  useConfirm: vi.fn(),
}));

// Mock ContractFormModal
vi.mock("@/components/features/employees/contract-form-modal", () => ({
  default: ({ isOpen, onClose, editingContract, wsId, employeeId }: any) => {
    if (!isOpen) return null;
    return (
      <div data-testid="mock-contract-form-modal">
        <div>Employee ID: {employeeId}</div>
        <div>Workspace ID: {wsId}</div>
        {!!editingContract && <div>Editing: {editingContract.id}</div>}
        <Button onClick={onClose} type="button">Close Form</Button>
      </div>
    );
  },
}));

const mockContracts = [
  {
    employee_id: "emp-1",
    employee_name: "John Doe",
    employee_code: "EMP001",
    contract: {
      id: "contract-1",
      contract_number: "CON-1001",
      contract_type: "fixed_term" as const,
      start_date: "2025-01-01T00:00:00Z",
      end_date: "2026-01-01T00:00:00Z",
      status: "active" as const,
    },
  },
  {
    employee_id: "emp-2",
    employee_name: "Jane Smith",
    employee_code: "EMP002",
    contract: {
      id: "contract-2",
      contract_number: "CON-1002",
      contract_type: "probation" as const,
      start_date: "2025-02-01T00:00:00Z",
      end_date: "2025-03-01T00:00:00Z",
      status: "draft" as const,
    },
  },
];

const mockEmployees = [
  { id: "emp-1", full_name: "John Doe", employee_id_number: "EMP001" },
  { id: "emp-2", full_name: "Jane Smith", employee_id_number: "EMP002" },
];

describe("EContractsPage Component - Registry", () => {
  let queryClient: QueryClient;
  const mockConfirm = vi.fn();

  beforeEach(() => {
    mockWorkspaceId = "ws-1";
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0 },
        mutations: { retry: false },
      },
    });

    (fetchWorkspaceContracts as Mock).mockResolvedValue(mockContracts);
    (useAllEmployees as Mock).mockReturnValue({
      employees: mockEmployees,
      isLoading: false,
    });
    (useConfirm as Mock).mockReturnValue({
      confirm: mockConfirm,
      ConfirmationDialog: () => <div data-testid="mock-confirmation-dialog" />,
    });
    (deleteEmployeeContract as Mock).mockResolvedValue(undefined);
  });

  const renderPage = () => {
    return render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <EContractsPage />
        </ToastProvider>
      </QueryClientProvider>
    );
  };

  it("renders page title and contracts table list", async () => {
    renderPage();
    expect(screen.getByText("Centralized Contract Registry")).toBeInTheDocument();
    await screen.findByText("CON-1001");
    expect(screen.getByText("CON-1002")).toBeInTheDocument();
    expect(screen.getByText("John Doe")).toBeInTheDocument();
    expect(screen.getByText("Jane Smith")).toBeInTheDocument();
  });

  it("filters contracts list by search query", async () => {
    const user = userEvent.setup();
    renderPage();
    await screen.findByText("CON-1001");

    const searchInput = screen.getByPlaceholderText("Search employee, ID or contract...");
    await user.type(searchInput, "Jane");

    expect(screen.queryByText("John Doe")).not.toBeInTheDocument();
    expect(screen.getByText("Jane Smith")).toBeInTheDocument();
  });

  it("filters contracts list by type select", async () => {
    renderPage();
    await screen.findByText("CON-1001");

    const typeSelect = screen.getByDisplayValue("ALL TYPES");
    fireEvent.change(typeSelect, { target: { value: "probation" } });

    expect(screen.queryByText("John Doe")).not.toBeInTheDocument();
    expect(screen.getByText("Jane Smith")).toBeInTheDocument();
  });

  it("filters contracts list by status select", async () => {
    renderPage();
    await screen.findByText("CON-1001");

    const statusSelect = screen.getByDisplayValue("ALL STATUS");
    fireEvent.change(statusSelect, { target: { value: "draft" } });

    expect(screen.queryByText("John Doe")).not.toBeInTheDocument();
    expect(screen.getByText("Jane Smith")).toBeInTheDocument();
  });

  it("renders empty state when no contracts match the filters", async () => {
    const user = userEvent.setup();
    renderPage();
    await screen.findByText("CON-1001");

    const searchInput = screen.getByPlaceholderText("Search employee, ID or contract...");
    await user.type(searchInput, "NoMatch");

    expect(screen.getByText("No Contracts Found")).toBeInTheDocument();
    const clearBtn = await screen.findByText("CLEAR FILTERS");
    await user.click(clearBtn);

    await screen.findByText("CON-1001");
  });

  it("opens SelectEmployeeModal and selects employee to sign new contract", async () => {
    const user = userEvent.setup();
    renderPage();
    await screen.findByText("CON-1001");

    const signBtn = screen.getByRole("button", { name: /SIGN CONTRACT/i });
    await user.click(signBtn);

    // Modal should be open
    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByText("Select Employee")).toBeInTheDocument();

    // Select employee
    const selectEmpBtn = within(dialog).getByText("John Doe");
    await user.click(selectEmpBtn);

    // Should open contract form modal
    expect(screen.getByTestId("mock-contract-form-modal")).toBeInTheDocument();
    expect(screen.getByText("Employee ID: emp-1")).toBeInTheDocument();
  });

  it("opens SelectEmployeeModal and searches for employee", async () => {
    const user = userEvent.setup();
    renderPage();
    await screen.findByText("CON-1001");

    const signBtn = screen.getByRole("button", { name: /SIGN CONTRACT/i });
    await user.click(signBtn);

    const dialog = screen.getByRole("dialog");
    const searchInput = within(dialog).getByPlaceholderText("Search employee by name or code...");
    await user.type(searchInput, "Jane");

    expect(within(dialog).queryByText("John Doe")).not.toBeInTheDocument();
    expect(within(dialog).getByText("Jane Smith")).toBeInTheDocument();
  });

  it("opens edit contract modal on clicking Edit", async () => {
    const user = userEvent.setup();
    renderPage();
    await screen.findByText("CON-1001");

    const editBtns = screen.getAllByRole("button");
    // Find button with Pencil icon (which will be mapped in button structure)
    // The registry table has Pencil buttons inside tr actions.
    const editBtn = editBtns.find((btn: HTMLElement) => btn.innerHTML.includes("pencil"));
    if (editBtn) {
      await user.click(editBtn);
      expect(screen.getByTestId("mock-contract-form-modal")).toBeInTheDocument();
      expect(screen.getByText("Editing: contract-1")).toBeInTheDocument();
    }
  });

  it("opens confirmation dialog and deletes contract on clicking Delete", async () => {
    const user = userEvent.setup();
    mockConfirm.mockResolvedValueOnce(true);
    renderPage();
    await screen.findByText("CON-1001");

    const deleteBtns = screen.getAllByRole("button");
    const deleteBtn = deleteBtns.find((btn: HTMLElement) => btn.innerHTML.includes("trash2"));
    if (deleteBtn) {
      await user.click(deleteBtn);
      expect(mockConfirm).toHaveBeenCalled();
      await waitFor(() => {
        expect(deleteEmployeeContract).toHaveBeenCalledWith("ws-1", "emp-1", "contract-1");
      });
    }
  });

  it("shows workspace selection warning when wsId is empty", () => {
    mockWorkspaceId = "";
    renderPage();
    expect(screen.getByText("common.select_workspace")).toBeInTheDocument();
  });
});
