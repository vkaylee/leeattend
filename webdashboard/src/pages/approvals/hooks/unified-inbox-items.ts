import type {
  AttendanceAdjustmentRequest,
  AttendanceDailySummary,
  Employee,
  LeaveRequest,
  ShiftSwapRequest,
} from "@/lib/api";

import type {
  TeamImpactInfo,
  UnifiedApprovalRequester,
} from "../types/unified-approvals";

export type EmployeeMap = Map<string, UnifiedApprovalRequester>;

export interface ApprovalSources {
  adjustments: AttendanceAdjustmentRequest[];
  leaves: LeaveRequest[];
  overtime: AttendanceDailySummary[];
  swaps: ShiftSwapRequest[];
}

export interface ItemBuilderContext extends ApprovalSources {
  employeeMap: EmployeeMap;
  getTeamImpact: (
    targetDate: string,
    departmentName?: string,
    itemId?: string,
  ) => TeamImpactInfo | undefined;
  translate: (key: string, defaultValue: string) => string;
}

export function createEmployeeMap(employees: Employee[]): EmployeeMap {
  const map = new Map<string, UnifiedApprovalRequester>();

  for (const employee of employees) {
    const requester: UnifiedApprovalRequester = {
      id: employee.id,
      name: employee.full_name,
      idNumber: employee.employee_id_number,
      departmentName: employee.department_name ?? undefined,
    };

    map.set(employee.id, requester);
    if (employee.active_employment_id) {
      map.set(employee.active_employment_id, requester);
    }
  }

  return map;
}

export function createTeamImpactResolver(
  approvedLeaves: LeaveRequest[],
  employeeMap: EmployeeMap,
) {
  return (
    targetDate: string,
    departmentName?: string,
    itemId?: string,
  ): TeamImpactInfo | undefined => {
    if (!departmentName || !targetDate) return undefined;

    const date = targetDate.split("T")[0];
    const colleaguesAwayNames = approvedLeaves
      .filter(
        (leave) =>
          leave.id !== itemId &&
          leave.start_date <= date &&
          leave.end_date >= date &&
          employeeMap.get(leave.employment_id)?.departmentName === departmentName,
      )
      .map((leave) => employeeMap.get(leave.employment_id)?.name ?? leave.employment_id);

    return {
      departmentName,
      colleaguesAwayCount: colleaguesAwayNames.length,
      colleaguesAwayNames,
      isHighImpact: colleaguesAwayNames.length >= 2,
    };
  };
}
