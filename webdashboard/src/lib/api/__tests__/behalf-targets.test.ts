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
    workspace_id: "ws-test-123",
    employee_id_number: "EMP001",
    full_name: "Alice Nguyen",
    is_active: true,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
  },
  {
    id: "emp-2",
    workspace_id: "ws-test-123",
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

    const result = await fetchMyBehalfTargets("ws-test-123", "leave");

    expect(apiFetch).toHaveBeenCalledWith(
      "/v1/workspaces/ws-test-123/leave/behalf-assignments/my-targets",
    );
    expect(result).toEqual(MOCK_EMPLOYEES);
  });

  it("calls correct URL for attendance_adjustment domain", async () => {
    const mockResponse = {
      ok: true,
      json: vi.fn().mockResolvedValue(MOCK_EMPLOYEES),
    };
    (apiFetch as Mock).mockResolvedValue(mockResponse);

    const result = await fetchMyBehalfTargets("ws-test-123", "attendance_adjustment");

    expect(apiFetch).toHaveBeenCalledWith(
      "/v1/workspaces/ws-test-123/attendance/adjustments/behalf-assignments/my-targets",
    );
    expect(result).toEqual(MOCK_EMPLOYEES);
  });

  it("calls correct URL for overtime_request domain", async () => {
    const mockResponse = {
      ok: true,
      json: vi.fn().mockResolvedValue(MOCK_EMPLOYEES),
    };
    (apiFetch as Mock).mockResolvedValue(mockResponse);

    const result = await fetchMyBehalfTargets("ws-test-123", "overtime_request");

    expect(apiFetch).toHaveBeenCalledWith(
      "/v1/workspaces/ws-test-123/overtime/requests/behalf-assignments/my-targets",
    );
    expect(result).toEqual(MOCK_EMPLOYEES);
  });

  it("throws on non-ok response", async () => {
    const mockResponse = {
      ok: false,
      status: 403,
      json: vi.fn().mockResolvedValue({ error: "Forbidden" }),
    };
    (apiFetch as Mock).mockResolvedValue(mockResponse);

    await expect(
      fetchMyBehalfTargets("ws-test-123", "leave"),
    ).rejects.toThrow("Failed to fetch behalf targets: 403");
  });

  it("returns empty array when API returns empty list", async () => {
    const mockResponse = {
      ok: true,
      json: vi.fn().mockResolvedValue([]),
    };
    (apiFetch as Mock).mockResolvedValue(mockResponse);

    const result = await fetchMyBehalfTargets("ws-test-123", "leave");

    expect(result).toEqual([]);
  });
});
