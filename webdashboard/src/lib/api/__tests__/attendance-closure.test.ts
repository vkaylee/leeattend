import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("../client", () => ({
  client: {
    GET: vi.fn(),
    POST: vi.fn(),
  },
}));

import { closeAttendancePeriod, fetchAttendancePeriodStatus, reopenAttendancePeriod } from "../attendance-closure";
import { client } from "../client";

describe("attendance closure API", () => {
  beforeEach(() => vi.clearAllMocks());

  it("reads status with employer query and period path", async () => {
    vi.mocked(client.GET).mockResolvedValue({
      data: { status: "open", version: 1, payroll_period_found: true },
      response: new Response(null, { status: 200 }),
    });
    await fetchAttendancePeriodStatus(2026, 11, "employer-1");
    expect(client.GET).toHaveBeenCalledWith("/api/v1/attendance/periods/{year}/{month}", {
      params: { path: { year: 2026, month: 11 }, query: { employer_id: "employer-1" } },
    });
  });

  it("posts close and reopen with expected version and reason", async () => {
    vi.mocked(client.POST).mockResolvedValue({
      data: { status: "closed", version: 2, period_id: "period-1" },
      response: new Response(null, { status: 201 }),
    });
    await closeAttendancePeriod(2026, 11, "employer-1", { version: 1, reason: "Monthly close" });
    await reopenAttendancePeriod(2026, 11, "employer-1", { version: 2, reason: "Correction" });
    expect(client.POST).toHaveBeenNthCalledWith(1, "/api/v1/attendance/periods/{year}/{month}/close", expect.objectContaining({
      params: expect.objectContaining({ query: { employer_id: "employer-1" } }),
      body: { version: 1, reason: "Monthly close" },
    }));
    expect(client.POST).toHaveBeenNthCalledWith(2, "/api/v1/attendance/periods/{year}/{month}/reopen", expect.objectContaining({
      body: { version: 2, reason: "Correction" },
    }));
  });
});
