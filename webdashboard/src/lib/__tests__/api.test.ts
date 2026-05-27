import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../api";

describe("api.ts comprehensive tests", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
    localStorage.clear();
  });

 
  const mockSuccess = (data: unknown = {}) => {
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      headers: new Headers({ "content-type": "application/json" }),
      json: () => Promise.resolve(data),
      text: () => Promise.resolve(data ? JSON.stringify(data) : ""),
    } as unknown as Response);
  };

  describe("Device & Agent API", () => {
    it("fetchAgents", async () => {
      mockSuccess([]);
      await api.fetchAgents();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/agents"), expect.any(Object));
    });

    it("sendAgentCommand", async () => {
      mockSuccess({});
      await api.sendAgentCommand("ag-1", { cmd: "test" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/agent/ag-1/command"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchDevices", async () => {
      mockSuccess([]);
      await api.fetchDevices();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/devices"), expect.any(Object));
    });

    it("createDevice", async () => {
      mockSuccess({ id: "d1" });
      await api.createDevice({ name: "D" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/devices"), expect.objectContaining({ method: "POST" }));
    });

    it("updateDevice", async () => {
      mockSuccess({});
      await api.updateDevice("dev-1", { name: "New Name" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/devices/dev-1"), expect.objectContaining({ method: "PUT" }));
    });

    it("deleteDevice", async () => {
      mockSuccess({});
      await api.deleteDevice("dev-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/devices/dev-1"), expect.objectContaining({ method: "DELETE" }));
    });

    it("fetchAuthorizedAgents", async () => {
      mockSuccess([]);
      await api.fetchAuthorizedAgents(true);
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/authorized-agents?include_inactive=true"), expect.any(Object));
    });

    it("fetchAuthorizedAgentById", async () => {
      mockSuccess({});
      await api.fetchAuthorizedAgentById("ag-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/authorized-agents/ag-1"), expect.any(Object));
    });

    it("updateAgentTrust", async () => {
      mockSuccess({});
      await api.updateAgentTrust("ag-1", true);
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/authorized-agents/ag-1/trust"), expect.objectContaining({ method: "PUT" }));
    });

    it("updateAuthorizedAgent", async () => {
      mockSuccess({});
      await api.updateAuthorizedAgent("ag-1", { name: "New" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/authorized-agents/ag-1"), expect.objectContaining({ method: "PATCH" }));
    });

    it("deleteAuthorizedAgent", async () => {
      mockSuccess({});
      await api.deleteAuthorizedAgent("ag-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/authorized-agents/ag-1"), expect.objectContaining({ method: "DELETE" }));
    });
  });

  describe("Employee & Department API", () => {
    it("fetchEmployees", async () => {
      mockSuccess([]);
      await api.fetchEmployees("ws-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees"), expect.any(Object));
    });

    it("fetchEmployee", async () => {
      mockSuccess({});
      await api.fetchEmployee("ws-1", "e1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/e1"), expect.any(Object));
    });

    it("createEmployee", async () => {
      mockSuccess({});
      await api.createEmployee("ws-1", { full_name: "A" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees"), expect.objectContaining({ method: "POST" }));
    });

    it("updateEmployee", async () => {
      mockSuccess({});
      await api.updateEmployee("ws-1", "e1", { full_name: "B" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/e1"), expect.objectContaining({ method: "PUT" }));
    });

    it("deleteEmployee", async () => {
      mockSuccess({});
      await api.deleteEmployee("ws-1", "e1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/e1"), expect.objectContaining({ method: "DELETE" }));
    });

    it("fetchDepartments", async () => {
      mockSuccess([]);
      await api.fetchDepartments("ws-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/departments"), expect.any(Object));
    });

    it("createDepartment", async () => {
      mockSuccess({});
      await api.createDepartment("ws-1", { name: "IT" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/departments"), expect.objectContaining({ method: "POST" }));
    });

    it("transferEmployee", async () => {
      mockSuccess({});
      await api.transferEmployee("ws-1", "emp-1", { department_id: "d1", effective_date: "..." });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/emp-1/departments"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchEmployeeDepartmentHistory", async () => {
      mockSuccess([]);
      await api.fetchEmployeeDepartmentHistory("ws-1", "emp-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/emp-1/departments"), expect.any(Object));
    });
  });

  describe("Time Tracking & Attendance API", () => {
    it("punchIn", async () => {
      mockSuccess({});
      await api.punchIn("ws-1", { employee_id: "e1" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/time/punch-in"), expect.objectContaining({ method: "POST" }));
    });

    it("punchOut", async () => {
      mockSuccess({});
      await api.punchOut("ws-1", "e1", { note: "N" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/time/punch-out/e1"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchTimeHistory", async () => {
      mockSuccess([]);
      await api.fetchTimeHistory("ws-1", "e1", "2026-01-01", "2026-01-31");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/time/history/e1?start=2026-01-01&end=2026-01-31"), expect.any(Object));
    });

    it("updateTimeEntry", async () => {
      mockSuccess({});
      await api.updateTimeEntry("ws-1", "t1", { note: "U" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/time/entries/t1"), expect.objectContaining({ method: "PUT" }));
    });

    it("deleteTimeEntry", async () => {
      mockSuccess({});
      await api.deleteTimeEntry("ws-1", "t1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/time/entries/t1"), expect.objectContaining({ method: "DELETE" }));
    });

    it("submitAttendanceAdjustment", async () => {
      mockSuccess({});
      await api.submitAttendanceAdjustment("ws-1", { employee_id: "e1", requested_punch_in_time: "..." });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/attendance/adjustments"), expect.objectContaining({ method: "POST" }));
    });

    it("uploadAttendanceExcel", async () => {
      mockSuccess({ job_id: "j1" });
      const file = new File([""], "test.csv");
      await api.uploadAttendanceExcel("ws-1", file);
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/attendance/upload"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchLatestImportJob", async () => {
      vi.mocked(fetch).mockResolvedValueOnce({ ok: true, headers: new Headers({ "content-type": "application/json" }), json: () => Promise.resolve({ id: "job-1" }), text: () => Promise.resolve(JSON.stringify({ id: "job-1" })) } as unknown as Response);
      await api.fetchLatestImportJob("ws-1", "adjustment");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/attendance/import-jobs/latest?target_type=adjustment"), expect.anything());
    });

    it("fetchImportJobStatus", async () => {
      mockSuccess({});
      await api.fetchImportJobStatus("ws-1", "j1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/attendance/import-jobs/j1"), expect.any(Object));
    });

    it("fetchWorkspaceAttendanceAdjustments", async () => {
      mockSuccess([]);
      await api.fetchWorkspaceAttendanceAdjustments("ws-1", "pending");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/attendance/adjustments?status=pending"), expect.any(Object));
    });

    it("approveAttendanceAdjustment", async () => {
      mockSuccess({});
      await api.approveAttendanceAdjustment("ws-1", "req-1", { status: "approved" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/attendance/adjustments/req-1/approve"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchAttendanceLogs", async () => {
      localStorage.setItem("active_workspace_id", "ws-1");
      mockSuccess({ data: [], total: 0 });
      await api.fetchAttendanceLogs({ page: 1 });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/attendance/logs"), expect.any(Object));
    });

    it("fetchAttendanceReport", async () => {
      mockSuccess({});
      await api.fetchAttendanceReport({ wsId: "ws-1", start: "2026-01-01", end: "2026-01-31" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/reports/attendance?start_date=2026-01-01&end_date=2026-01-31&type=summary"), expect.any(Object));
    });
  });

  describe("Leave API", () => {
    it("submitLeaveRequest", async () => {
      mockSuccess({});
      await api.submitLeaveRequest("ws-1", { employee_id: "e1", leave_type_id: "lt-1", start_date: "...", end_date: "..." });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/leave/submit"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchWorkspaceLeaveRequests", async () => {
      mockSuccess([]);
      await api.fetchWorkspaceLeaveRequests("ws-1", "pending");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/leave/requests?status=pending"), expect.any(Object));
    });

    it("approveLeaveRequest", async () => {
      mockSuccess({});
      await api.approveLeaveRequest("ws-1", "req-1", { status: "approved" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/leave/requests/req-1/approve"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchEmployeeLeaveBalances", async () => {
      mockSuccess([]);
      await api.fetchEmployeeLeaveBalances("ws-1", "e1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/e1/leave/balances"), expect.any(Object));
    });

    it("fetchLeavePolicies", async () => {
      mockSuccess([]);
      await api.fetchLeavePolicies("ws-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/leave/policies"), expect.any(Object));
    });

    it("createLeavePolicy", async () => {
      mockSuccess({});
      await api.createLeavePolicy("ws-1", { name: "P" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/leave/policies"), expect.objectContaining({ method: "POST" }));
    });

    it("assignLeavePolicy", async () => {
      mockSuccess({});
      await api.assignLeavePolicy("ws-1", {
        assignee_type: "employee",
        assignee_id: "e1",
        policy_id: "p1",
        effective_from: "2026-01-01"
      });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/leave/policies/assign"), expect.objectContaining({ method: "POST" }));
    });

    it("triggerAccrualRun returns AccrualJobResponse", async () => {
      mockSuccess({ job_id: "j-123", status: "queued", message: "Accrual job enqueued" });
      const result = await api.triggerAccrualRun("ws-1", { date: "2026-01-01" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/leave/accrual/run"), expect.objectContaining({ method: "POST" }));
      expect(result).toEqual({ job_id: "j-123", status: "queued", message: "Accrual job enqueued" });
    });

    it("triggerAccrualRun handles completed status", async () => {
      mockSuccess({ job_id: "", status: "completed", message: "Processed 10 entries synchronously" });
      const result = await api.triggerAccrualRun("ws-1", { date: "2026-02-01" });
      expect(result.status).toBe("completed");
      expect(result.job_id).toBe("");
    });

    it("fetchJobStatus returns job status", async () => {
      mockSuccess({ job_id: "j-1", status: "processing", completed_at: null, error_message: null });
      const result = await api.fetchJobStatus("ws-1", "j-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/jobs/j-1/status"), expect.any(Object));
      expect(result.status).toBe("processing");
    });

    it("fetchJobStatus returns completed with timestamp", async () => {
      mockSuccess({ job_id: "j-2", status: "completed", completed_at: "2026-02-28T12:00:00Z", error_message: null });
      const result = await api.fetchJobStatus("ws-1", "j-2");
      expect(result.status).toBe("completed");
      expect(result.completed_at).toBe("2026-02-28T12:00:00Z");
    });

    it("fetchJobStatus returns failed with error message", async () => {
      mockSuccess({ job_id: "j-3", status: "failed", completed_at: "2026-02-28T12:00:00Z", error_message: "DB error" });
      const result = await api.fetchJobStatus("ws-1", "j-3");
      expect(result.status).toBe("failed");
      expect(result.error_message).toBe("DB error");
    });
  });

  describe("Payroll & Shift API", () => {
    it("calculate_payroll", async () => {
      mockSuccess({});
      await api.calculate_payroll("ws-1", "e1", 2026, 1);
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/e1/calculate-payroll?year=2026&month=1"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchEmployeePayslips", async () => {
      mockSuccess([]);
      await api.fetchEmployeePayslips("ws-1", "e1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/e1/payslips"), expect.any(Object));
    });

    it("fetchShifts", async () => {
      mockSuccess([]);
      await api.fetchShifts("ws-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/shifts"), expect.any(Object));
    });

    it("createShift", async () => {
      mockSuccess({});
      await api.createShift("ws-1", { name: "S" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/shifts"), expect.objectContaining({ method: "POST" }));
    });

    it("updateShift", async () => {
      mockSuccess({});
      await api.updateShift("ws-1", "s1", { name: "U" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/shifts/s1"), expect.objectContaining({ method: "PUT" }));
    });

    it("assignShift", async () => {
      mockSuccess({});
      await api.assignShift("ws-1", { employee_id: "e1", shift_id: "s1", start_date: "..." });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/shifts/assign"), expect.objectContaining({ method: "POST" }));
    });
  });

  describe("Roster API", () => {
    it("fetchWorkspaceSchedules", async () => {
      mockSuccess([]);
      await api.fetchWorkspaceSchedules("ws-1", "2026-01-01", "2026-01-07");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/schedules"), expect.any(Object));
    });

    it("fetchAssignmentHistory", async () => {
      mockSuccess([]);
      await api.fetchAssignmentHistory("ws-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/assignment-history"), expect.any(Object));
    });

    it("fetchEmployeeRosters", async () => {
      mockSuccess([]);
      await api.fetchEmployeeRosters("ws-1", "e1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/e1/rosters"), expect.any(Object));
    });

    it("upsertRoster", async () => {
      mockSuccess({});
      await api.upsertRoster("ws-1", "e1", { shift_id: "s1", date: "..." });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/e1/rosters"), expect.objectContaining({ method: "POST" }));
    });

    it("bulkUpsertRoster", async () => {
      mockSuccess({});
      await api.bulkUpsertRoster("ws-1", "e1", []);
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/e1/rosters/bulk"), expect.objectContaining({ method: "POST" }));
    });

    it("deleteRoster", async () => {
      mockSuccess({});
      await api.deleteRoster("ws-1", "e1", "2026-01-01");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/employees/e1/rosters/2026-01-01"), expect.objectContaining({ method: "DELETE" }));
    });
  });

  describe("Workspace & Auth Management", () => {
    it("fetchWorkspaces", async () => {
      mockSuccess([]);
      await api.fetchWorkspaces();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces"), expect.any(Object));
    });

    it("createWorkspace", async () => {
      mockSuccess({});
      await api.createWorkspace({ name: "N", database_url: "postgres://u:p@h:5432/db" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchWorkspace", async () => {
      mockSuccess({});
      await api.fetchWorkspace("ws-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1"), expect.any(Object));
    });

    it("updateWorkspace", async () => {
      mockSuccess({});
      await api.updateWorkspace("ws-1", { name: "U" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1"), expect.objectContaining({ method: "PATCH" }));
    });

    it("fetchWorkspaceMembers", async () => {
      mockSuccess([]);
      await api.fetchWorkspaceMembers("ws-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/members"), expect.any(Object));
    });

    it("addWorkspaceMember", async () => {
      mockSuccess({});
      await api.addWorkspaceMember("ws-1", "e@e.com", "admin");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/members"), expect.objectContaining({ method: "POST" }));
    });

    it("removeWorkspaceMember", async () => {
      mockSuccess({});
      await api.removeWorkspaceMember("ws-1", "u1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/members/u1"), expect.objectContaining({ method: "DELETE" }));
    });

    it("updateWorkspaceMemberRole", async () => {
      mockSuccess({});
      await api.updateWorkspaceMemberRole("ws-1", "u1", "owner");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/members/u1"), expect.objectContaining({ method: "PATCH" }));
    });

    it("login", async () => {
      mockSuccess({ access_token: "at", refresh_token: "rt", user: { id: "u1" } });
      await api.login("user", "pass");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/auth/login"), expect.objectContaining({ method: "POST" }));
      expect(localStorage.getItem("access_token")).toBe("at");
    });

    it("logout", async () => {
      mockSuccess({});
      await api.logout();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/auth/logout"), expect.objectContaining({ method: "POST" }));
      expect(localStorage.getItem("access_token")).toBeNull();
    });

    it("register", async () => {
      mockSuccess({});
      await api.register({ email: "e", password: "p", full_name: "f" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/auth/register"), expect.objectContaining({ method: "POST" }));
    });

    it("changePassword", async () => {
      vi.mocked(fetch).mockResolvedValue({ ok: true, headers: new Headers({ "content-type": "application/json" }), text: () => Promise.resolve("{}"), json: () => Promise.resolve({}) } as unknown as Response);
      await api.changePassword({ old_password: "o", new_password: "n" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/auth/change-password"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchMe", async () => {
      mockSuccess({});
      await api.fetchMe();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/me"), expect.any(Object));
    });

    it("updateMe", async () => {
      mockSuccess({});
      await api.updateMe({ full_name: "F" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/me"), expect.objectContaining({ method: "PATCH" }));
    });

    it("fetchUsers", async () => {
      mockSuccess([]);
      await api.fetchUsers();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/users"), expect.any(Object));
    });

    it("fetchMemberRoles", async () => {
      mockSuccess({});
      await api.fetchMemberRoles("ws-1", "u1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/members/u1/roles"), expect.any(Object));
    });

    it("assignExtraRole", async () => {
      mockSuccess({});
      await api.assignExtraRole("ws-1", "u1", "r1", "...");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/members/u1/roles"), expect.objectContaining({ method: "POST" }));
    });

    it("revokeExtraRole", async () => {
      mockSuccess({});
      await api.revokeExtraRole("ws-1", "u1", "rid1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/workspaces/ws-1/members/u1/roles/rid1"), expect.objectContaining({ method: "DELETE" }));
    });

    it("fetchDevSuperUser", async () => {
      mockSuccess({});
      await api.fetchDevSuperUser();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/dev/super-user"), expect.any(Object));
    });
  });
});
