import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock the dependencies BEFORE importing the file under test
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

import { apiFetch } from "../helpers";
import { fetchSystemTenantStats } from "../system";

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
        headers: new Headers({ "content-type": "application/json" }),
        json: () => Promise.resolve(mockStats),
        text: () => Promise.resolve(JSON.stringify(mockStats)),
      } as Response);

      const result = await fetchSystemTenantStats("tenant_123");

      expect(apiFetch).toHaveBeenCalledTimes(1);
      expect(apiFetch).toHaveBeenCalledWith(expect.stringContaining("/system/tenants/tenant_123/stats"), expect.any(Object));
      expect(result).toEqual(mockStats);
    });

    it("should throw error if fetch failed", async () => {
      const mockResponse = {
        ok: false,
        status: 500,
        statusText: "Internal Server Error",
        headers: new Headers({ "content-type": "application/json" }),
        text: () => Promise.resolve("Failed to fetch tenant stats"),
        json: () => Promise.resolve({ message: "Failed to fetch tenant stats" }),
        clone: function() { return this; }
      } as unknown as Response;

      vi.mocked(apiFetch).mockResolvedValueOnce(mockResponse);

      await expect(fetchSystemTenantStats("tenant_123")).rejects.toThrow();
    });
  });
});

