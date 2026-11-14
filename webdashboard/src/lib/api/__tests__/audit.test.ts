import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../client", () => ({
  fetch: vi.fn(),
}));

import { fetchAuditLogs } from "../audit";
import { fetch as apiFetch } from "../client";

describe("fetchAuditLogs", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("requests the flat audit-log endpoint with pagination and filters", async () => {
    const response = { data: [], total: 0, page: 2, limit: 20 };
    vi.mocked(apiFetch).mockResolvedValueOnce({ data: response });

    await expect(fetchAuditLogs({
      page: 2,
      limit: 20,
      action: "member.invite",
      startDate: "2026-01-01T00:00:00.000Z",
      endDate: "2026-01-31T23:59:59.999Z",
    })).resolves.toEqual(response);

    expect(apiFetch).toHaveBeenCalledWith("GET", "/api/v1/audit-logs", {
      params: {
        query: {
          page: 2,
          limit: 20,
          action: "member.invite",
          start_date: "2026-01-01T00:00:00.000Z",
          end_date: "2026-01-31T23:59:59.999Z",
        },
      },
    });
  });
});
