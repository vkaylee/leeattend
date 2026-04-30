import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock the dependencies BEFORE importing the file under test
vi.mock("../helpers", () => ({
    apiFetch: vi.fn(),
    getCurrentWorkspaceId: vi.fn().mockReturnValue("test-ws-id"),
    TIMEOUT: { REPORT_OPERATION: 180000 },
}));

import { fetchAttendanceReport } from "../attendance";
import { API_BASE_URL } from "../config";
import { apiFetch } from "../helpers";

describe("fetchAttendanceReport", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("should append standard query parameters correctly", async () => {
 
        vi.mocked(apiFetch).mockResolvedValueOnce({
            ok: true,
            json: async () => ({ data: [], total: 0 }),
        } as Response);

        await fetchAttendanceReport("test-ws-id", "2026-03-01", "2026-03-31", "summary");

        expect(apiFetch).toHaveBeenCalledTimes(1);
 
        const calledUrl = vi.mocked(apiFetch).mock.calls[0][0];

        expect(calledUrl).toContain(`${API_BASE_URL}/workspaces/test-ws-id/reports/attendance`);
        expect(calledUrl).toContain("start_date=2026-03-01");
        expect(calledUrl).toContain("end_date=2026-03-31");
        expect(calledUrl).toContain("type=summary");
        expect(calledUrl).not.toContain("department_id="); // Should not be there if omitted
    });

    it("should append department_id query parameter when provided", async () => {
 
        vi.mocked(apiFetch).mockResolvedValueOnce({
            ok: true,
            json: async () => ({ data: [], total: 0 }),
        } as Response);

        const departmentId = "dept-uuid-123";

        // type signature: wsId, start, end, type, page, limit, sort, employeeIds, departmentId
        await fetchAttendanceReport(
            "test-ws-id",
            "2026-03-01",
            "2026-03-31",
            "summary",
            1,
            20,
            undefined,
            undefined,
            departmentId
        );

        expect(apiFetch).toHaveBeenCalledTimes(1);
 
        const calledUrl = vi.mocked(apiFetch).mock.calls[0][0];

        // Verify department_id is appended correctly
        expect(calledUrl).toContain(`department_id=${departmentId}`);

        // Check some other standard params were mapped correctly
        expect(calledUrl).toContain("page=1");
        expect(calledUrl).toContain("limit=20");
    });

    it("should format employee_ids as comma-separated string when provided", async () => {
 
        vi.mocked(apiFetch).mockResolvedValueOnce({
            ok: true,
            json: async () => ({ data: [], total: 0 }),
        } as Response);

        const employeeIds = ["emp-1", "emp-2"];

        await fetchAttendanceReport(
            "test-ws-id",
            "2026-03-01",
            "2026-03-31",
            "summary",
            1,
            20,
            undefined,
            employeeIds
        );

        expect(apiFetch).toHaveBeenCalledTimes(1);
 
        const calledUrl = vi.mocked(apiFetch).mock.calls[0][0];

        expect(calledUrl).toContain("employee_ids=emp-1%2Cemp-2"); // %2C is URL encoded comma
    });
});
