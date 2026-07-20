import { describe, it, expect, vi, beforeEach } from "vitest";

// system.ts mixes typed client.GET/POST (for /system/roles,/system/health,
// /system/db-clusters) with the untyped fetch/fire wrappers (for tenants and
// tenant stats). Both paths traverse apiFetch, so a single mock intercepts all.
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

import { apiFetch } from "../helpers";
import {
  fetchSystemRoles,
  fetchUserSystemRoles,
  assignSystemRole,
  revokeSystemRole,
  fetchSystemHealth,
  fetchSystemTenants,
  createSystemTenant,
  deleteSystemTenant,
  fetchSystemTenantStats,
  updateTenantDatabase,
  fetchSystemDbClusters,
  createSystemDbCluster,
  deleteSystemDbCluster,
  updateSystemDbClusterStatus,
} from "../system";

function okRes(payload: unknown) {
  return {
    ok: true,
    status: 200,
    statusText: "OK",
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(payload),
    text: () => Promise.resolve(JSON.stringify(payload)),
    clone() {
      return this;
    },
  } as unknown as Response;
}

function errRes(status: number, statusText: string, body: unknown) {
  return {
    ok: false,
    status,
    statusText,
    headers: new Headers({ "content-type": "application/json" }),
    json: () => Promise.resolve(body),
    text: () => Promise.resolve(JSON.stringify(body)),
    clone() {
      return this;
    },
  } as unknown as Response;
}

describe("system API (extended)", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("fetchSystemRoles: GET list", async () => {
    const mock = [{ name: "admin", description: "Administrator" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchSystemRoles();
    expect(result).toEqual(mock);
  });

  it("fetchSystemRoles: ApiError on 403", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(403, "Forbidden", {}));
    await expect(fetchSystemRoles()).rejects.toThrow();
  });

  it("fetchUserSystemRoles: GET list for user", async () => {
    const mock = [{ user_id: "u1", role_name: "admin" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchUserSystemRoles("u1");
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/system/users/u1/roles");
  });

  it("assignSystemRole: POST forwards request body", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await assignSystemRole("u1", { role_name: "manager" } as never);
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/system/users/u1/roles");
    expect(JSON.parse((call[1] as RequestInit).body as string).role_name).toBe("manager");
  });

  it("revokeSystemRole: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await revokeSystemRole("u1", "manager");
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/system/users/u1/roles/manager");
    expect((call[1] as RequestInit).method).toBe("DELETE");
  });

  it("fetchSystemHealth: GET returns record", async () => {
    const mock = { status: "ok", uptime: 12345 };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchSystemHealth();
    expect(result).toEqual(mock);
  });

  it("fetchSystemTenants: GET list (via fetch wrapper)", async () => {
    const mock = [{ id: "t1", name: "Test", is_byod: false, timezone: "UTC", created_at: "" }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchSystemTenants();
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/system/tenants");
  });

  it("fetchSystemTenants: ApiError on 403", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(403, "Forbidden", {}));
    await expect(fetchSystemTenants()).rejects.toThrow();
  });

  it("createSystemTenant: POST (via fire wrapper)", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await createSystemTenant({ name: "Test Tenant" });
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect((call[1] as RequestInit).method).toBe("POST");
    expect(call[0]).toContain("/system/tenants");
    expect(JSON.parse((call[1] as RequestInit).body as string).name).toBe("Test Tenant");
  });

  it("deleteSystemTenant: DELETE (via fire wrapper)", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteSystemTenant("t1");
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect((call[1] as RequestInit).method).toBe("DELETE");
    expect(call[0]).toContain("/system/tenants/t1");
  });

  it("fetchSystemTenantStats: GET stats (via fetch wrapper)", async () => {
    const mock = {
      id: "t1",
      name: "Test",
      is_byod: false,
      timezone: "UTC",
      created_at: "2024-01-01T00:00:00Z",
      masked_db_url: null,
      connection: { host: "h", port: "1" },
    };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchSystemTenantStats("t1");
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/system/tenants/t1/stats");
  });

  it("fetchSystemTenantStats: ApiError on 500", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(errRes(500, "Error", {}));
    await expect(fetchSystemTenantStats("t1")).rejects.toThrow();
  });

  it("updateTenantDatabase: PUT (via fire wrapper)", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await updateTenantDatabase("t1", "postgres://x");
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect((call[1] as RequestInit).method).toBe("PUT");
    expect(JSON.parse((call[1] as RequestInit).body as string).database_url).toBe("postgres://x");
  });

  it("fetchSystemDbClusters: GET list", async () => {
    const mock = [{ id: "c1", host: "h", is_active: true }];
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await fetchSystemDbClusters();
    expect(result).toEqual(mock);
  });

  it("createSystemDbCluster: POST returns cluster", async () => {
    const mock = { id: "c-new", host: "h2", is_active: false };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await createSystemDbCluster({ host: "h2" } as never);
    expect(result).toEqual(mock);
    expect(vi.mocked(apiFetch).mock.calls[0][0]).toContain("/system/db-clusters");
  });

  it("deleteSystemDbCluster: DELETE", async () => {
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(undefined));
    await deleteSystemDbCluster("c1");
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/system/db-clusters/c1");
    expect((call[1] as RequestInit).method).toBe("DELETE");
  });

  it("updateSystemDbClusterStatus: PATCH returns cluster", async () => {
    const mock = { id: "c1", is_active: false };
    vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mock));
    const result = await updateSystemDbClusterStatus("c1", false);
    expect(result).toEqual(mock);
    const call = vi.mocked(apiFetch).mock.calls[0];
    expect(call[0]).toContain("/system/db-clusters/c1/status");
    expect((call[1] as RequestInit).method).toBe("PATCH");
    expect(JSON.parse((call[1] as RequestInit).body as string).is_active).toBe(false);
  });
});
