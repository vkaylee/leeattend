import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import React from "react";

import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { ToastProvider } from "../../components/ui/toast";
import { useConfirm } from "../../hooks/use-confirm";
import * as systemApi from "../../lib/api/system";
import TenantsPage from "../system-admin/tenants-page";

// ── Mocks ────────────────────────────────────────────────────────────────────

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

vi.mock("../../hooks/use-confirm", () => ({
  useConfirm: vi.fn(),
}));

vi.mock("../../lib/api/system", () => ({
  fetchSystemTenants: vi.fn(),
  createSystemTenant: vi.fn(),
  deleteSystemTenant: vi.fn(),
  fetchSystemTenantStats: vi.fn(),
  updateTenantDatabase: vi.fn(),
}));

// ── Helpers ──────────────────────────────────────────────────────────────────

const MOCK_TENANTS: systemApi.SystemTenant[] = [
  {
    id: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
    name: "Workspace Alpha",
    is_byod: false,
    timezone: "Asia/Ho_Chi_Minh",
    created_at: "2024-01-15T10:00:00Z",
  },
  {
    id: "ffffffff-eeee-dddd-cccc-bbbbbbbbbbbb",
    name: "BYOD Corp",
    is_byod: true,
    timezone: "UTC",
    created_at: "2024-02-01T00:00:00Z",
  },
];

describe("TenantsPage", () => {
  let queryClient: QueryClient;
  const mockConfirm = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    vi.mocked(systemApi.fetchSystemTenants).mockResolvedValue(MOCK_TENANTS);
    vi.mocked(systemApi.createSystemTenant).mockResolvedValue(undefined);
    vi.mocked(systemApi.deleteSystemTenant).mockResolvedValue(undefined);
    vi.mocked(systemApi.updateTenantDatabase).mockResolvedValue(undefined);
    vi.mocked(systemApi.fetchSystemTenantStats).mockResolvedValue({
      id: "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee",
      name: "Workspace Alpha",
      is_byod: false,
      timezone: "Asia/Ho_Chi_Minh",
      created_at: "2024-01-15T10:00:00Z",
      masked_db_url: "postgres://user:***@localhost:5432/db",
      connection: { host: "localhost", port: "5432" },
    });

    (useConfirm as Mock).mockReturnValue({
      confirm: mockConfirm,
      ConfirmationDialog: () => <div data-testid="confirmation-dialog"></div>,
    });
  });

  const renderPage = () =>
    render(
      <QueryClientProvider client={queryClient}>
        <ToastProvider>
          <TenantsPage />
        </ToastProvider>
      </QueryClientProvider>
    );

  it("renders the tenant table with correct columns", async () => {
    renderPage();

    expect(await screen.findByText("Workspace Alpha")).toBeInTheDocument();
    expect(screen.getByText("BYOD Corp")).toBeInTheDocument();
    expect(screen.getByText("Standard")).toBeInTheDocument();
    expect(screen.getByText("BYOD")).toBeInTheDocument();
  });

  it("opens create modal and submits", async () => {
    const user = userEvent.setup();
    renderPage();

    const addBtn = await screen.findByText("system_admin.tenants.add_btn");
    await user.click(addBtn);

    const nameInput = await screen.findByPlaceholderText("My Workspace");
    await user.type(nameInput, "New Workspace");

    const submitBtn = screen.getByText("system_admin.tenants.modal.submit_btn");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(systemApi.createSystemTenant).toHaveBeenCalledWith({
        name: "New Workspace",
        database_url: undefined,
      });
    });
  });

  it("opens stats modal", async () => {
    const user = userEvent.setup();
    renderPage();

    const statsBtns = await screen.findAllByTitle("View Connection & Stats");
    await user.click(statsBtns[0]);

    await waitFor(() => {
      expect(systemApi.fetchSystemTenantStats).toHaveBeenCalledWith(MOCK_TENANTS[0].id);
      expect(screen.getByText("postgres://user:***@localhost:5432/db")).toBeInTheDocument();
    });
  });

  it("opens update db modal and submits", async () => {
    const user = userEvent.setup();
    mockConfirm.mockResolvedValueOnce(true);
    renderPage();

    const manageDbBtns = await screen.findAllByTitle("Manage Database");
    await user.click(manageDbBtns[0]);

    const urlInput = await screen.findByPlaceholderText("postgres://user:password@host:5432/db");
    await user.type(urlInput, "postgres://new");

    const updateBtn = screen.getByText("Update Database");
    await user.click(updateBtn);

    await waitFor(() => {
      expect(systemApi.updateTenantDatabase).toHaveBeenCalledWith(MOCK_TENANTS[0].id, "postgres://new");
    });
  });

  it("deletes a tenant after confirmation", async () => {
    const user = userEvent.setup();
    mockConfirm.mockResolvedValueOnce(true);
    renderPage();

    await screen.findByText("Workspace Alpha");
    const deleteBtns = screen.getAllByRole("button", { name: "" }).filter(
      (btn) => btn.querySelector("svg.text-red-500")
    );
    await user.click(deleteBtns[0]);

    expect(mockConfirm).toHaveBeenCalledWith(
      expect.objectContaining({ variant: "danger" })
    );

    await waitFor(() => {
      expect(systemApi.deleteSystemTenant).toHaveBeenCalledWith(MOCK_TENANTS[0].id);
    });
  });
});
