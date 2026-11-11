import { describe, it, expect, vi, beforeEach } from "vitest";

import * as api from "../api";
import {
  getAccessToken,
  setCurrentUserId,
  setAccessToken,
} from "../api/helpers";

describe("api.ts comprehensive tests", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", vi.fn());
    localStorage.clear();
    // Tokens now live in the in-memory store (see lib/api/token-store.ts).
    // Reset via the public API so this test file starts clean.
    setAccessToken(null);
    setCurrentUserId(null);
  });

 
  const mockSuccess = (data: unknown = {}) => {
    vi.mocked(fetch).mockResolvedValue({
      ok: true,
      headers: new Headers({ "content-type": "application/json" }),
      json: () => Promise.resolve(data),
      text: () => Promise.resolve(data ? JSON.stringify(data) : ""),
    } as unknown as Response);
  };


  describe("Employee & Department API", () => {
    it("fetchEmployees uses flat route", async () => {
      mockSuccess([]);
      await api.fetchEmployees();
      const [url] = vi.mocked(fetch).mock.calls[0];
      expect(url).toContain("/api/v1/employees");
      expect(url).not.toContain("/workspaces/");
    });

    it("fetchEmployee uses flat route", async () => {
      mockSuccess({});
      await api.fetchEmployee("e1");
      const [url] = vi.mocked(fetch).mock.calls[0];
      expect(url).toContain("/api/v1/employees/e1");
      expect(url).not.toContain("/workspaces/");
    });

    it("createEmployee uses flat route", async () => {
      mockSuccess({});
      await api.createEmployee({ full_name: "A", employee_id_number: "E001", employer_id: "emp-1", hire_date: "2024-01-01" });
      const [url, init] = vi.mocked(fetch).mock.calls[0];
      expect(url).toContain("/api/v1/employees");
      expect(url).not.toContain("/workspaces/");
      expect(init).toMatchObject({ method: "POST" });
    });

    it("updateEmployee uses flat route", async () => {
      mockSuccess({});
      await api.updateEmployee("e1", { full_name: "B" });
      const [url, init] = vi.mocked(fetch).mock.calls[0];
      expect(url).toContain("/api/v1/employees/e1");
      expect(url).not.toContain("/workspaces/");
      expect(init).toMatchObject({ method: "PUT" });
    });

    it("deleteEmployee uses flat route", async () => {
      mockSuccess({});
      await api.deleteEmployee("e1");
      const [url, init] = vi.mocked(fetch).mock.calls[0];
      expect(url).toContain("/api/v1/employees/e1");
      expect(url).not.toContain("/workspaces/");
      expect(init).toMatchObject({ method: "DELETE" });
    });

    it("fetchDepartments", async () => {
      mockSuccess([]);
      await api.fetchDepartments();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/departments"), expect.any(Object));
    });

    it("createDepartment", async () => {
      mockSuccess({});
      await api.createDepartment({ name: "IT" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/departments"), expect.objectContaining({ method: "POST" }));
    });

    it("transferEmployee uses flat route", async () => {
      mockSuccess({});
      await api.transferEmployee("emp-1", { department_id: "d1", effective_date: "..." });
      const [url] = vi.mocked(fetch).mock.calls[0];
      expect(url).toContain("/api/v1/employees/emp-1/departments");
      expect(url).not.toContain("/workspaces/");
    });

    it("fetchEmployeeDepartmentHistory uses flat route", async () => {
      mockSuccess([]);
      await api.fetchEmployeeDepartmentHistory("emp-1");
      const [url] = vi.mocked(fetch).mock.calls[0];
      expect(url).toContain("/api/v1/employees/emp-1/departments");
      expect(url).not.toContain("/workspaces/");
    });
  });

  describe("Time Tracking & Attendance API", () => {
    it("punchIn", async () => {
      mockSuccess({});
      await api.punchIn({ employment_id: "e1" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/time/punch-in"), expect.objectContaining({ method: "POST" }));
    });

    it("punchOut", async () => {
      mockSuccess({});
      await api.punchOut({ employee_id: "e1", note: "N" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/time/punch-out"), expect.objectContaining({ method: "POST" }));
    });

    it("submitAttendanceAdjustment", async () => {
      mockSuccess({});
      await api.submitAttendanceAdjustment({ employment_id: "e1", requested_punch_time: "..." });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/attendance/adjustments"), expect.objectContaining({ method: "POST" }));
    });

    it("uploadAttendance", async () => {
      mockSuccess({});
      const file = new File([""], "test.xlsx");
      await api.uploadAttendance(file, "/attendance/logs/upload");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/attendance/logs/upload"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchLatestImportJob", async () => {
      vi.mocked(fetch).mockResolvedValueOnce({ ok: true, headers: new Headers({ "content-type": "application/json" }), json: () => Promise.resolve({ id: "job-1" }), text: () => Promise.resolve(JSON.stringify({ id: "job-1" })) } as unknown as Response);
      await api.fetchLatestImportJob("adjustment");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/attendance/import-jobs/latest?target_type=adjustment"), expect.anything());
    });

    it("fetchImportJobStatus", async () => {
      mockSuccess({});
      await api.fetchImportJobStatus("j1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/attendance/import-jobs/j1"), expect.any(Object));
    });

    it("fetchAttendanceAdjustments", async () => {
      mockSuccess([]);
      await api.fetchAttendanceAdjustments({ status: "pending" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/attendance/adjustments?status=pending"), expect.any(Object));
    });

    it("approveAttendanceAdjustment", async () => {
      mockSuccess({});
      await api.approveAttendanceAdjustment("req-1", { status: "approved" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/attendance/adjustments/req-1/approve"), expect.objectContaining({ method: "PUT" }));
    });

    it("fetchAttendanceReport", async () => {
      mockSuccess({});
      await api.fetchAttendanceReport({ start: "2026-01-01", end: "2026-01-31", type: "summary" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/reports/attendance?start_date=2026-01-01&end_date=2026-01-31&type=summary"), expect.any(Object));
    });
  });

  describe("Leave API", () => {
    it("submitLeaveRequest", async () => {
      mockSuccess({});
      await api.submitLeaveRequest({ employment_id: "e1", leave_type_id: "lt-1", start_date: "...", end_date: "..." });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/leave/submit"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchLeaveRequests", async () => {
      mockSuccess([]);
      await api.fetchLeaveRequests({ status: "pending" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/leave/requests?status=pending"), expect.any(Object));
    });

    it("approveLeaveRequest", async () => {
      mockSuccess({});
      await api.approveLeaveRequest("req-1", { status: "approved" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/leave/requests/req-1/approve"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchEmployeeLeaveBalances", async () => {
      mockSuccess([]);
      await api.fetchEmployeeLeaveBalances("e1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/employees/e1/leave/balances"), expect.any(Object));
    });

    it("fetchLeavePolicies", async () => {
      mockSuccess([]);
      await api.fetchLeavePolicies();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/leave/policies"), expect.any(Object));
    });

    it("createLeavePolicy", async () => {
      mockSuccess({});
      await api.createLeavePolicy({ name: "P", leave_type_id: "lt-1", rule_config: {} as never });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/leave/policies"), expect.objectContaining({ method: "POST" }));
    });

    it("assignLeavePolicy", async () => {
      mockSuccess({});
      await api.assignLeavePolicy({
        assignee_type: "employee",
        assignee_id: "e1",
        policy_id: "p1",
        effective_from: "2026-01-01"
      });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/leave/policies/assign"), expect.objectContaining({ method: "POST" }));
    });

    it("triggerAccrualRun returns AccrualJobResponse", async () => {
      mockSuccess({ job_id: "j-123", status: "queued", message: "Accrual job enqueued" });
      const result = await api.triggerAccrualRun({ date: "2026-01-01" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/leave/accrual/run"), expect.objectContaining({ method: "POST" }));
      expect(result).toEqual({ job_id: "j-123", status: "queued", message: "Accrual job enqueued" });
    });

    it("triggerAccrualRun handles completed status", async () => {
      mockSuccess({ job_id: "", status: "completed", message: "Processed 10 entries synchronously" });
      const result = await api.triggerAccrualRun({ date: "2026-02-01" });
      expect(result.status).toBe("completed");
      expect(result.job_id).toBe("");
    });

    it("fetchJobStatus returns job status", async () => {
      mockSuccess({ job_id: "j-1", status: "processing", completed_at: null, error_message: null });
      const result = await api.fetchJobStatus("j-1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/jobs/j-1/status"), expect.any(Object));
      expect(result.status).toBe("processing");
    });

    it("fetchJobStatus returns completed with timestamp", async () => {
      mockSuccess({ job_id: "j-2", status: "completed", completed_at: "2026-02-28T12:00:00Z", error_message: null });
      const result = await api.fetchJobStatus("j-2");
      expect(result.status).toBe("completed");
      expect(result.completed_at).toBe("2026-02-28T12:00:00Z");
    });

    it("fetchJobStatus returns failed with error message", async () => {
      mockSuccess({ job_id: "j-3", status: "failed", completed_at: "2026-02-28T12:00:00Z", error_message: "DB error" });
      const result = await api.fetchJobStatus("j-3");
      expect(result.status).toBe("failed");
      expect(result.error_message).toBe("DB error");
    });
  });

  describe("Payroll & Shift API", () => {
    it("calculate_payroll", async () => {
      mockSuccess({});
      await api.calculate_payroll("e1", 2026, 1);
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/employees/e1/calculate-payroll?year=2026&month=1"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchEmployeePayslips", async () => {
      mockSuccess([]);
      await api.fetchEmployeePayslips("e1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/employees/e1/payslips"), expect.any(Object));
    });

    it("fetchShifts", async () => {
      mockSuccess([]);
      await api.fetchShifts();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/shifts"), expect.any(Object));
    });

    it("createShift", async () => {
      mockSuccess({});
      await api.createShift({ name: "S" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/shifts"), expect.objectContaining({ method: "POST" }));
    });

    it("updateShift", async () => {
      mockSuccess({});
      await api.updateShift("s1", { name: "U" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/shifts/s1"), expect.objectContaining({ method: "PUT" }));
    });

    it("assignShift", async () => {
      mockSuccess({});
      await api.assignShift({ employment_id: "e1", shift_id: "s1", start_date: "..." });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/shifts/assign"), expect.objectContaining({ method: "POST" }));
    });
  });

  describe("Roster API", () => {
    it("fetchSchedules", async () => {
      mockSuccess([]);
      await api.fetchSchedules("2026-01-01", "2026-01-07");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/schedules"), expect.any(Object));
    });

    it("fetchAssignmentHistory", async () => {
      mockSuccess([]);
      await api.fetchAssignmentHistory();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/assignment-history"), expect.any(Object));
    });

    it("fetchEmployeeRosters", async () => {
      mockSuccess([]);
      await api.fetchEmployeeRosters("e1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/employees/e1/rosters"), expect.any(Object));
    });

    it("upsertRoster", async () => {
      mockSuccess({});
      await api.upsertRoster("e1", { shift_id: "s1", date: "..." });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/employees/e1/rosters"), expect.objectContaining({ method: "POST" }));
    });

    it("bulkUpsertRoster", async () => {
      mockSuccess({});
      await api.bulkUpsertRoster("e1", []);
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/employees/e1/rosters/bulk"), expect.objectContaining({ method: "POST" }));
    });

    it("deleteRoster", async () => {
      mockSuccess({});
      await api.deleteRoster("e1", "2026-01-01");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/employees/e1/rosters/2026-01-01"), expect.objectContaining({ method: "DELETE" }));
    });
  });

  describe("Organization & Auth Management", () => {
    it("fetchOrganization", async () => {
      mockSuccess({});
      await api.fetchOrganization();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/organization/settings"), expect.any(Object));
    });

    it("updateOrganization", async () => {
      mockSuccess({});
      await api.updateOrganization({ name: "U" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/organization/settings"), expect.objectContaining({ method: "PATCH" }));
    });

    it("fetchOrganizationMembers", async () => {
      mockSuccess([]);
      await api.fetchOrganizationMembers();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/members"), expect.any(Object));
    });

    it("addOrganizationMember", async () => {
      mockSuccess({});
      await api.addOrganizationMember("e@e.com", "admin");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/members"), expect.objectContaining({ method: "POST" }));
    });

    it("removeOrganizationMember", async () => {
      mockSuccess({});
      await api.removeOrganizationMember("u1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/members/u1"), expect.objectContaining({ method: "DELETE" }));
    });

    it("updateOrganizationMemberRole", async () => {
      mockSuccess({});
      await api.updateOrganizationMemberRole("u1", "owner");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/members/u1"), expect.objectContaining({ method: "PATCH" }));
    });

    it("login", async () => {
      mockSuccess({ access_token: "at", refresh_token: "rt", user: { id: "u1" } });
      await api.login("user", "pass");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/auth/login"), expect.objectContaining({ method: "POST" }));
      // access_token is now held in the in-memory store, not localStorage.
      expect(getAccessToken()).toBe("at");
    });

    it("logout", async () => {
      mockSuccess({});
      setAccessToken("existing"); // pre-condition: a token exists
      await api.logout();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/auth/logout"), expect.objectContaining({ method: "POST" }));
      // access_token is wiped from the in-memory store on logout.
      expect(getAccessToken()).toBeNull();
    });

    it("register", async () => {
      mockSuccess({});
      await api.register({ email: "e", password: "p", full_name: "f" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/auth/register"), expect.objectContaining({ method: "POST" }));
    });

    it("changePassword", async () => {
      vi.mocked(fetch).mockResolvedValue({ ok: true, headers: new Headers({ "content-type": "application/json" }), text: () => Promise.resolve("{}"), json: () => Promise.resolve({}) } as unknown as Response);
      await api.changePassword({ old_password: "o", new_password: "n" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/auth/change-password"), expect.objectContaining({ method: "POST" }));
    });

    it("fetchMe", async () => {
      mockSuccess({});
      await api.fetchMe();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/me"), expect.any(Object));
    });

    it("updateMe", async () => {
      mockSuccess({});
      await api.updateMe({ full_name: "F" });
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/me"), expect.objectContaining({ method: "PATCH" }));
    });

    it("fetchUsers", async () => {
      mockSuccess([]);
      await api.fetchUsers();
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/users"), expect.any(Object));
    });

    it("fetchMemberRoles", async () => {
      mockSuccess({});
      await api.fetchMemberRoles("u1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/members/u1/roles"), expect.any(Object));
    });

    it("assignExtraRole", async () => {
      mockSuccess({});
      await api.assignExtraRole("u1", "r1", "...");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/members/u1/roles"), expect.objectContaining({ method: "POST" }));
    });

    it("revokeExtraRole", async () => {
      mockSuccess({});
      await api.revokeExtraRole("u1", "rid1");
      expect(fetch).toHaveBeenCalledWith(expect.stringContaining("/api/v1/members/u1/roles/rid1"), expect.objectContaining({ method: "DELETE" }));
    });

  });
});
