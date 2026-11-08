import { describe, it, expect, vi, beforeEach, type Mock } from "vitest";

import { fetchMyBehalfTargets } from "../behalf-targets";
import { apiFetch } from "../helpers";

// Mock apiFetch
vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

const MOCK_EMPLOYEES = [
  {
    id: "emp-1",
    employee_id_number: "EMP001",
    full_name: "Alice Nguyen",
    is_active: true,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "emp-2",
    employee_id_number: "EMP002",
    full_name: "Bob Tran",
    is_active: true,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
];

describe("fetchMyBehalfTargets", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("calls correct URL for leave domain", async () => {
    const mockResponse = {
      ok: true,
      json: vi.fn().mockResolvedValue(MOCK_EMPLOYEES),
    };
    (apiFetch as Mock).mockResolvedValue(mockResponse);

    const result = await fetchMyBehalfTargets("leave");

    expect(apiFetch).toHaveBeenCalledWith(
      "/v1/leave/behalf-assignments/my-targets",
    );
    expect(result).toEqual(MOCK_EMPLOYEES);
  });

  it("calls correct URL for attendance_adjustment domain", async () => {
    const mockResponse = {
      ok: true,
      json: vi.fn().mockResolvedValue(MOCK_EMPLOYEES),
    };
    (apiFetch as Mock).mockResolvedValue(mockResponse);

    const result = await fetchMyBehalfTargets("attendance_adjustment");

    expect(apiFetch).toHaveBeenCalledWith(
      "/v1/attendance/adjustments/behalf-assignments/my-targets",
    );
    expect(result).toEqual(MOCK_EMPLOYEES);
  });

  it("calls correct URL for overtime_request domain", async () => {
    const mockResponse = {
      ok: true,
      json: vi.fn().mockResolvedValue(MOCK_EMPLOYEES),
    };
    (apiFetch as Mock).mockResolvedValue(mockResponse);

    const result = await fetchMyBehalfTargets("overtime_request");

    expect(apiFetch).toHaveBeenCalledWith(
      "/v1/overtime/requests/behalf-assignments/my-targets",
    );
    expect(result).toEqual(MOCK_EMPLOYEES);
  });

  it("throws on non-ok response", async () => {
    const mockResponse = {
      ok: false,
      status: 403,
      statusText: "Forbidden",
      headers: new Headers({ "content-type": "application/json" }),
      text: vi.fn().mockResolvedValue(JSON.stringify({ error: "Forbidden" })),
      json: vi.fn().mockResolvedValue({ error: "Forbidden" }),
    };
    (apiFetch as Mock).mockResolvedValue(mockResponse);

    await expect(
      fetchMyBehalfTargets("leave"),
    ).rejects.toThrow();
  });

  it("returns empty array when API returns empty list", async () => {
    const mockResponse = {
      ok: true,
      json: vi.fn().mockResolvedValue([]),
    };
    (apiFetch as Mock).mockResolvedValue(mockResponse);

    const result = await fetchMyBehalfTargets("leave");

    expect(result).toEqual([]);
  });
});
