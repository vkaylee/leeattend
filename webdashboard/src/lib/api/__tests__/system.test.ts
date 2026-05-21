import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock the dependencies BEFORE importing the file under test
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

import { API_BASE_URL } from "../config";
import { apiFetch } from "../helpers";
import { fetchSystemTenantStats, updateTenantMode } from "../system";

describe("System API methods", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("fetchSystemTenantStats", () => {
    it("should fetch tenant connection stats correctly", async () => {
      const mockStats = {
        config: {
          data: {
            db_database: "tenant_db",
            default_mode: "transaction",
            users: [{ db_user: "admin", mode_type: "transaction" }],
          },
        },
        connection: {
          host: "localhost",
          port: "6543",
          masked_db_url: "postgres://admin:***@localhost:6543/tenant_db",
        },
      };

      vi.mocked(apiFetch).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockStats),
      } as Response);

      const result = await fetchSystemTenantStats("tenant_123");

      expect(apiFetch).toHaveBeenCalledTimes(1);
      expect(apiFetch).toHaveBeenCalledWith(`${API_BASE_URL}/system/tenants/tenant_123/stats`);
      expect(result).toEqual(mockStats);
    });

    it("should throw error if fetch failed", async () => {
      vi.mocked(apiFetch).mockResolvedValueOnce({
        ok: false,
      } as Response);

      await expect(fetchSystemTenantStats("tenant_123")).rejects.toThrow("Failed to fetch tenant stats");
    });
  });

  describe("updateTenantMode", () => {
    it("should patch tenant mode correctly", async () => {
      vi.mocked(apiFetch).mockResolvedValueOnce({
        ok: true,
      } as Response);

      await updateTenantMode("tenant_123", "transaction");

      expect(apiFetch).toHaveBeenCalledTimes(1);
      expect(apiFetch).toHaveBeenCalledWith(
        `${API_BASE_URL}/system/tenants/tenant_123/mode`,
        {
          method: "PATCH",
          body: JSON.stringify({ mode_type: "transaction" }),
        }
      );
    });

    it("should throw error if update failed", async () => {
      vi.mocked(apiFetch).mockResolvedValueOnce({
        ok: false,
        text: () => Promise.resolve("Supavisor error"),
      } as Response);

      await expect(updateTenantMode("tenant_123", "transaction")).rejects.toThrow("Supavisor error");
    });
  });
});
