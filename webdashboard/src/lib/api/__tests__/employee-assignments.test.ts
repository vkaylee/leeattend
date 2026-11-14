import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("../client", () => ({
  client: {
    GET: vi.fn(),
    POST: vi.fn(),
    PATCH: vi.fn(),
    DELETE: vi.fn(),
  },
}));

import { client } from "../client";
import {
  fetchAssignments,
  fetchEmployeeAssignments,
  createEmployeeAssignment,
  updateEmployeeAssignment,
  deleteEmployeeAssignment,
} from "../employee-assignments";

describe("employee-assignments api", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("fetchAssignments", () => {
    it("fetches assignments from the flat endpoint", async () => {
      const mockData = [{ id: "assign-1", employee_id: "emp-1", employee_name: "Nguyen Van A", status: "active" }];
      vi.mocked(client.GET).mockResolvedValueOnce({ data: mockData } as unknown as never);

      const result = await fetchAssignments();
      expect(result).toEqual(mockData);
      expect(client.GET).toHaveBeenCalledWith("/api/v1/assignments");
    });
  });

  describe("fetchEmployeeAssignments", () => {
    it("calls client.GET with employee id and flat endpoint", async () => {
      const mockList = [{ id: "assign-1", status: "active" }];
      vi.mocked(client.GET).mockResolvedValueOnce({ data: mockList } as unknown as never);

      const result = await fetchEmployeeAssignments("emp-1");
      expect(result).toEqual(mockList);
      expect(client.GET).toHaveBeenCalledWith("/api/v1/employees/{employee_id}/assignments", {
        params: { path: { employee_id: "emp-1" } },
      });
    });
  });

  describe("createEmployeeAssignment", () => {
    it("calls client.POST with data", async () => {
      const mockCreated = { id: "assign-new", status: "scheduled" };
      vi.mocked(client.POST).mockResolvedValueOnce({ data: mockCreated } as unknown as never);

      const payload = {
        host_location_id: "loc-2",
        start_date: "2026-06-01",
      };
      const result = await createEmployeeAssignment("emp-1", payload);
      expect(result).toEqual(mockCreated);
      expect(client.POST).toHaveBeenCalledWith(
        "/api/v1/employees/{employee_id}/assignments",
        {
          params: { path: { employee_id: "emp-1" } },
          body: payload,
        }
      );
    });
  });

  describe("updateEmployeeAssignment", () => {
    it("calls client.PATCH with payload", async () => {
      const mockUpdated = { id: "assign-1", status: "completed" };
      vi.mocked(client.PATCH).mockResolvedValueOnce({ data: mockUpdated } as unknown as never);

      const payload = { status: "completed" as const };
      const result = await updateEmployeeAssignment("emp-1", "assign-1", payload);
      expect(result).toEqual(mockUpdated);
      expect(client.PATCH).toHaveBeenCalledWith(
        "/api/v1/employees/{employee_id}/assignments/{assignment_id}",
        {
          params: { path: { employee_id: "emp-1", assignment_id: "assign-1" } },
          body: payload,
        }
      );
    });
  });

  describe("deleteEmployeeAssignment", () => {
    it("calls client.DELETE", async () => {
      vi.mocked(client.DELETE).mockResolvedValueOnce({ data: undefined } as unknown as never);

      await deleteEmployeeAssignment("emp-1", "assign-1");
      expect(client.DELETE).toHaveBeenCalledWith(
        "/api/v1/employees/{employee_id}/assignments/{assignment_id}",
        {
          params: { path: { employee_id: "emp-1", assignment_id: "assign-1" } },
        }
      );
    });
  });
});
