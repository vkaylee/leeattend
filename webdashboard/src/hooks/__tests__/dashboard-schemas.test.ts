import { describe, expect, it } from "vitest";

import {
    attendanceTrendSchema,
    departmentAttendanceSchema,
    organizationStatsSchema,
} from "../dashboard-schemas";

const rate = (value: string | null) => ({ value, numerator: 5, denominator: 10, unit: "employee_days" });

describe("dashboard operational schemas (rate contract)", () => {
    it("accepts organization stats with rate fields", () => {
        const payload = {
            total_employees: 10,
            present_today: 8,
            pending_leave_requests: 1,
            estimated_monthly_payroll: "50000.00",
            late_today: 1,
            absent_today: 1,
            on_leave_today: 1,
            early_departure_today: 0,
            avg_work_hours_today: 7.5,
            attendance_rate: rate("80"),
            absence_rate: rate("10"),
            late_rate: rate("10"),
        };
        expect(() => organizationStatsSchema.parse(payload)).not.toThrow();
    });

    it("rejects organization stats missing a rate field", () => {
        const { attendance_rate: _omit, ...rest } = {
            total_employees: 10,
            present_today: 8,
            pending_leave_requests: 1,
            estimated_monthly_payroll: "50000.00",
            late_today: 1,
            absent_today: 1,
            on_leave_today: 1,
            early_departure_today: 0,
            avg_work_hours_today: 7.5,
            attendance_rate: rate("80"),
            absence_rate: rate("10"),
            late_rate: rate("10"),
        };
        expect(() => organizationStatsSchema.parse(rest)).toThrow();
    });

    it("accepts attendance trend with eligible + rate fields", () => {
        const payload = [{
            date: "2026-02-01",
            present: 40,
            late: 3,
            absent: 5,
            on_leave: 2,
            eligible: 50,
            attendance_rate: rate("80"),
            absence_rate: rate("10"),
            late_rate: rate("6"),
        }];
        expect(() => attendanceTrendSchema.parse(payload)).not.toThrow();
    });

    it("accepts department attendance with total_employees + rate fields", () => {
        const payload = [{
            department_id: "dept-1",
            department_name: "Engineering",
            on_time: 15,
            late: 2,
            absent: 3,
            on_leave: 1,
            total_employees: 21,
            attendance_rate: rate("71.4"),
            absence_rate: rate("14.3"),
            late_rate: rate("9.5"),
        }];
        expect(() => departmentAttendanceSchema.parse(payload)).not.toThrow();
    });

    it("accepts null rate value (zero denominator) without fabricating 0", () => {
        const payload = [{
            date: "2026-02-01",
            present: 0,
            late: 0,
            absent: 0,
            on_leave: 0,
            eligible: 0,
            attendance_rate: rate(null),
            absence_rate: rate(null),
            late_rate: rate(null),
        }];
        expect(() => attendanceTrendSchema.parse(payload)).not.toThrow();
    });

    it("rejects unknown fields on rate quantities (strict)", () => {
        const payload = [{
            date: "2026-02-01",
            present: 0,
            late: 0,
            absent: 0,
            on_leave: 0,
            eligible: 0,
            attendance_rate: { ...rate(null), bogus: true },
            absence_rate: rate(null),
            late_rate: rate(null),
        }];
        expect(() => attendanceTrendSchema.parse(payload)).toThrow();
    });
});
