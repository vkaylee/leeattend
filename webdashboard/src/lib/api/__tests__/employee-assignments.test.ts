import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("../helpers", () => ({
  apiFetch: vi.fn(),
}));

vi.mock("../client", () => ({
  client: {
    GET: vi.fn(),
    POST: vi.fn(),
    PATCH: vi.fn(),
    DELETE: vi.fn(),
  },
}));

import {
  fetchWorkspaceAssignments,
  fetchEmployeeAssignments,
  createEmployeeAssignment,
  updateEmployeeAssignment,
  deleteEmployeeAssignment,
} from "../employee-assignments";
import { apiFetch } from "../helpers";
import { client } from "../client";

function okRes(payload: unknown) {
  return {
    ok: true,
    json: () => Promise.resolve(payload),
    text: () => Promise.resolve(JSON.stringify(payload)),
  };
}

describe("employee-assignments api", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("fetchWorkspaceAssignments", () => {
    it("fetches workspace assignments via apiFetch", async () => {
      const mockData = [
        {
          id: "assign-1",
          workspace_id: "ws-1",
          employee_id: "emp-1",
          employee_name: "Nguyen Van A",
          employee_code: "EMP-001",
          home_location_id: "loc-1",
          host_location_id: "loc-2",
          home_location_name: "HQ",
          host_location_name: "Branch 1",
          start_date: "2026-01-01",
          end_date: "2026-06-30",
          status: "active",
        },
      ];
      vi.mocked(apiFetch).mockResolvedValueOnce(okRes(mockData) as unknown as Response);

      const result = await fetchWorkspaceAssignments("ws-1");
      expect(result).toEqual(mockData);
      expect(apiFetch).toHaveBeenCalledWith("/v1/workspaces/ws-1/assignments");
    });

    it("throws error when apiFetch response is not ok", async () => {
      vi.mocked(apiFetch).mockResolvedValueOnce({
        ok: false,
        status: 500,
        text: () => Promise.resolve(JSON.stringify({ message: "Internal server error" })),
      } as unknown as Response);

      await expect(fetchWorkspaceAssignments("ws-1")).rejects.toThrow();
    });
  });

  describe("fetchEmployeeAssignments", () => {
    it("calls client.GET with workspace and employee id", async () => {
      const mockList = [{ id: "assign-1", status: "active" }];
      vi.mocked(client.GET).mockResolvedValueOnce({ data: mockList, error: null } as unknown as never);

      const result = await fetchEmployeeAssignments("ws-1", "emp-1");
      expect(result).toEqual(mockList);
      expect(client.GET).toHaveBeenCalledWith(
        "/v1/workspaces/{ws_id}/employees/{employee_id}/assignments",
        {
          params: { path: { ws_id: "ws-1", employee_id: "emp-1" } },
        }
      );
    });
  });

  describe("createEmployeeAssignment", () => {
    it("calls client.POST with data", async () => {
      const mockCreated = { id: "assign-new", status: "scheduled" };
      vi.mocked(client.POST).mockResolvedValueOnce({ data: mockCreated, error: null } as unknown as never);

      const payload = {
        host_location_id: "loc-2",
        start_date: "2026-06-01",
      };
      const result = await createEmployeeAssignment("ws-1", "emp-1", payload);
      expect(result).toEqual(mockCreated);
      expect(client.POST).toHaveBeenCalledWith(
        "/v1/workspaces/{ws_id}/employees/{employee_id}/assignments",
        {
          params: { path: { ws_id: "ws-1", employee_id: "emp-1" } },
          body: payload,
        }
      );
    });
  });

  describe("updateEmployeeAssignment", () => {
    it("calls client.PATCH with payload", async () => {
      const mockUpdated = { id: "assign-1", status: "completed" };
      vi.mocked(client.PATCH).mockResolvedValueOnce({ data: mockUpdated, error: null } as unknown as never);

      const payload = { status: "completed" as const };
      const result = await updateEmployeeAssignment("ws-1", "emp-1", "assign-1", payload);
      expect(result).toEqual(mockUpdated);
      expect(client.PATCH).toHaveBeenCalledWith(
        "/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}",
        {
          params: { path: { ws_id: "ws-1", employee_id: "emp-1", assignment_id: "assign-1" } },
          body: payload,
        }
      );
    });
  });

  describe("deleteEmployeeAssignment", () => {
    it("calls client.DELETE", async () => {
      vi.mocked(client.DELETE).mockResolvedValueOnce({ data: undefined, error: null } as unknown as never);

      await deleteEmployeeAssignment("ws-1", "emp-1", "assign-1");
      expect(client.DELETE).toHaveBeenCalledWith(
        "/v1/workspaces/{ws_id}/employees/{employee_id}/assignments/{assignment_id}",
        {
          params: { path: { ws_id: "ws-1", employee_id: "emp-1", assignment_id: "assign-1" } },
        }
      );
    });
  });
});
