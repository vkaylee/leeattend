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
  fetchStaleTenants: vi.fn(),
  createSystemTenant: vi.fn(),
  deleteSystemTenant: vi.fn(),
  fetchSystemTenantStats: vi.fn(),
  updateTenantDatabase: vi.fn(),
  updateTenantMode: vi.fn(),
}));

// ── Helpers ──────────────────────────────────────────────────────────────────

const MOCK_TENANTS: systemApi.SystemTenant[] = [
  {
    id: "tenant_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa1",
    pool_size: 10,
    created_at: "2024-01-15T10:00:00Z",
    updated_at: "2024-03-20T12:00:00Z",
  },
  {
    id: "leeattend",
    pool_size: 20,
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
  },
];

const MOCK_STALE_TENANTS: systemApi.SystemTenant[] = [
  {
    id: "tenant_orphan111111111111111111111111",
    pool_size: 5,
    created_at: "2024-02-01T00:00:00Z",
    updated_at: "2024-02-01T00:00:00Z",
  },
  {
    id: "tenant_orphan222222222222222222222222",
    pool_size: 3,
    created_at: "2024-03-01T00:00:00Z",
    updated_at: "2024-03-01T00:00:00Z",
  },
];

describe("TenantsPage — Stale Tenant Detection", () => {
  let queryClient: QueryClient;
  const mockConfirm = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    vi.mocked(systemApi.fetchSystemTenants).mockResolvedValue(MOCK_TENANTS);
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue([]);
    vi.mocked(systemApi.deleteSystemTenant).mockResolvedValue(undefined);

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

  // ── Button Presence ──────────────────────────────────────────────────────

  it("renders the Check Stale Tenants button", async () => {
    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    expect(btn).toBeInTheDocument();
  });

  it("renders the Add Tenant button alongside the stale check button", async () => {
    renderPage();
    expect(
      await screen.findByText("system_admin.tenants.add_btn")
    ).toBeInTheDocument();
    expect(
      screen.getByText("system_admin.tenants.check_stale_btn")
    ).toBeInTheDocument();
  });

  // ── Stale Modal — Empty State ────────────────────────────────────────────

  it("shows empty state when no stale tenants found", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue([]);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(
        screen.getByText("system_admin.tenants.stale_modal.no_stale")
      ).toBeInTheDocument();
    });
  });

  it("shows modal title and subtitle when opened", async () => {
    const user = userEvent.setup();
    renderPage();

    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(
        screen.getByText("system_admin.tenants.stale_modal.title")
      ).toBeInTheDocument();
    });
  });

  // ── Stale Modal — With Stale Tenants ─────────────────────────────────────

  it("displays stale tenants in the modal", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue(MOCK_STALE_TENANTS);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(screen.getByText("tenant_orphan111111111111111111111111")).toBeInTheDocument();
      expect(screen.getByText("tenant_orphan222222222222222222222222")).toBeInTheDocument();
    });
  });

  it("shows stale count warning banner", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue(MOCK_STALE_TENANTS);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(screen.getByText(/2 stale configurations detected/)).toBeInTheDocument();
    });
  });

  it("shows singular 'configuration' for single stale tenant", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue([MOCK_STALE_TENANTS[0]]);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(screen.getByText(/1 stale configuration detected/)).toBeInTheDocument();
    });
  });

  // ── Individual Cleanup ───────────────────────────────────────────────────

  it("triggers confirmation and deletes individual stale tenant", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue([MOCK_STALE_TENANTS[0]]);
    mockConfirm.mockResolvedValueOnce(true);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(screen.getByText("tenant_orphan111111111111111111111111")).toBeInTheDocument();
    });

    // Find the Delete button next to the stale tenant
    const deleteButtons = screen.getAllByText("Delete");
    await user.click(deleteButtons[0]);

    expect(mockConfirm).toHaveBeenCalledWith(
      expect.objectContaining({ variant: "danger" })
    );

    await waitFor(() => {
      expect(systemApi.deleteSystemTenant).toHaveBeenCalledWith(
        "tenant_orphan111111111111111111111111"
      );
    });
  });

  it("does not delete if confirmation is cancelled", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue([MOCK_STALE_TENANTS[0]]);
    mockConfirm.mockResolvedValueOnce(false);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(screen.getByText("tenant_orphan111111111111111111111111")).toBeInTheDocument();
    });

    const deleteButtons = screen.getAllByText("Delete");
    await user.click(deleteButtons[0]);

    expect(mockConfirm).toHaveBeenCalled();
    expect(systemApi.deleteSystemTenant).not.toHaveBeenCalled();
  });

  // ── Bulk Cleanup ─────────────────────────────────────────────────────────

  it("shows Clean Up All button when multiple stale tenants exist", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue(MOCK_STALE_TENANTS);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(
        screen.getByText("system_admin.tenants.stale_modal.clean_all")
      ).toBeInTheDocument();
    });
  });

  it("does not show Clean Up All button for single stale tenant", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue([MOCK_STALE_TENANTS[0]]);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(screen.getByText("tenant_orphan111111111111111111111111")).toBeInTheDocument();
    });

    expect(
      screen.queryByText("system_admin.tenants.stale_modal.clean_all")
    ).not.toBeInTheDocument();
  });

  it("triggers bulk cleanup with confirmation for all stale tenants", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue(MOCK_STALE_TENANTS);
    mockConfirm.mockResolvedValueOnce(true);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(
        screen.getByText("system_admin.tenants.stale_modal.clean_all")
      ).toBeInTheDocument();
    });

    const cleanAllBtn = screen.getByText("system_admin.tenants.stale_modal.clean_all");
    await user.click(cleanAllBtn);

    expect(mockConfirm).toHaveBeenCalledWith(
      expect.objectContaining({ variant: "danger" })
    );

    await waitFor(() => {
      expect(systemApi.deleteSystemTenant).toHaveBeenCalledTimes(2);
      expect(systemApi.deleteSystemTenant).toHaveBeenCalledWith(
        "tenant_orphan111111111111111111111111"
      );
      expect(systemApi.deleteSystemTenant).toHaveBeenCalledWith(
        "tenant_orphan222222222222222222222222"
      );
    });
  });

  // ── Modal Close ──────────────────────────────────────────────────────────

  it("can close the stale tenants modal via Close button", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue([]);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(
        screen.getByText("system_admin.tenants.stale_modal.no_stale")
      ).toBeInTheDocument();
    });

    const closeBtn = screen.getByText("Close");
    await user.click(closeBtn);

    await waitFor(() => {
      expect(
        screen.queryByText("system_admin.tenants.stale_modal.no_stale")
      ).not.toBeInTheDocument();
    });
  });

  // ── API interaction ────────────────────────────────────────────────────

  it("calls fetchStaleTenants when button is clicked", async () => {
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(systemApi.fetchStaleTenants).toHaveBeenCalled();
    });
  });

  // ── Pool size & date display ───────────────────────────────────────────

  it("displays pool size and creation date for stale tenants", async () => {
    vi.mocked(systemApi.fetchStaleTenants).mockResolvedValue([MOCK_STALE_TENANTS[0]]);
    const user = userEvent.setup();

    renderPage();
    const btn = await screen.findByText("system_admin.tenants.check_stale_btn");
    await user.click(btn);

    await waitFor(() => {
      expect(screen.getByText(/Pool: 5/)).toBeInTheDocument();
    });
  });
});

// ── API Client Tests ─────────────────────────────────────────────────────────

describe("fetchStaleTenants API client", () => {
  it("is exported and callable", () => {
    expect(typeof systemApi.fetchStaleTenants).toBe("function");
  });
});

describe("TenantsPage — Tenant Management", () => {
  let queryClient: QueryClient;
  const mockConfirm = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false } },
    });

    vi.mocked(systemApi.fetchSystemTenants).mockResolvedValue(MOCK_TENANTS);
    vi.mocked(systemApi.createSystemTenant).mockResolvedValue(undefined);
    vi.mocked(systemApi.updateTenantDatabase).mockResolvedValue(undefined);
    vi.mocked(systemApi.fetchSystemTenantStats).mockResolvedValue({
      connection: { host: "localhost", port: 5432, masked_db_url: "postgres://masked" },
      config: { users: [{ db_user: "user", pool_size: 10 }] }
    } as any);

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

  it("opens create modal and submits", async () => {
    const user = userEvent.setup();
    renderPage();

    const addBtn = await screen.findByText("system_admin.tenants.add_btn");
    await user.click(addBtn);

    const dbNameInput = await screen.findByPlaceholderText("system_admin.tenants.modal.db_name_placeholder");
    await user.type(dbNameInput, "new_db");

    const submitBtn = screen.getByText("system_admin.tenants.modal.submit_btn");
    await user.click(submitBtn);

    await waitFor(() => {
      expect(systemApi.createSystemTenant).toHaveBeenCalledWith({
        db_name: "new_db",
        create_db: true,
        pool_size: 15
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
      expect(screen.getByText("postgres://masked")).toBeInTheDocument();
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
});

