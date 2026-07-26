import { useMemo } from "react";

import { orderSchedulesBySelection } from "@/components/shifts/roster/roster-utils";

import type { Employee, EmployeeSchedule } from "@/lib/api/types";

interface UseMonthlySchedulesProps {
  /** Full employee list from the workspace (may be undefined while loading). */
  employees: Employee[] | undefined;
  /** Monthly schedule rows keyed by employee/employment. */
  monthSchedules: EmployeeSchedule[] | undefined;
  /** Ordered IDs driving selection-first row ordering. */
  selectedEmployeeIds: string[];
  /** Explicitly selected employee objects (may differ from employees list). */
  selectedEmployees: Employee[];
}

interface ScheduleRow {
  department_name?: string;
  employee_id: string;
  employee_id_number: string;
  employment_id: string;
  full_name: string;
  schedules: EmployeeSchedule["schedules"];
}

interface UseMonthlySchedulesReturn {
  /** Merged list of selected employees with resolved metadata. */
  selectedEmployeesMeta: Employee[];
  /** All employees + selected extras, enriched with month schedules. */
  combinedSchedules: ScheduleRow[];
  /** `combinedSchedules` reordered to put selected employees first. */
  filteredSchedules: ScheduleRow[];
}

/**
 * Derives the three schedule views needed by the monthly roster grid:
 * selected employee metadata, the full combined schedule list, and a
 * selection-first filtered view for grid rendering.
 */
export function useMonthlySchedules({
  employees,
  monthSchedules,
  selectedEmployeeIds,
  selectedEmployees,
}: UseMonthlySchedulesProps): UseMonthlySchedulesReturn {
  const selectedEmployeesMeta = useMemo(() => {
    const list = [...selectedEmployees];
    if (employees) {
      const empMap = new Map(employees.map((e) => [e.id, e]));
      selectedEmployeeIds.forEach((id) => {
        if (!list.some((e) => e.id === id)) {
          const emp = empMap.get(id);
          if (emp) list.push(emp);
        }
      });
    }
    return list;
  }, [employees, selectedEmployeeIds, selectedEmployees]);

  const combinedSchedules = useMemo(() => {
    if (!employees) return [];
    const list = [...employees];
    selectedEmployees.forEach((selectedEmp) => {
      if (!list.some((e) => e.id === selectedEmp.id)) {
        list.push(selectedEmp);
      }
    });
    return list.map((emp) => {
      const existing = monthSchedules?.find(
        (s) => s.employee_id === emp.id || s.employment_id === emp.active_employment_id,
      );
      if (existing) {
        return {
          ...existing,
          employee_id: emp.id,
          employee_id_number: emp.employee_id_number,
          department_name: emp.department_name ?? undefined,
        };
      }
      return {
        employee_id: emp.id,
        employee_id_number: emp.employee_id_number,
        department_name: emp.department_name ?? undefined,
        employment_id: emp.active_employment_id ?? "",
        full_name: emp.full_name,
        schedules: [],
      };
    });
  }, [employees, monthSchedules, selectedEmployees]);

  const filteredSchedules = useMemo(
    () => orderSchedulesBySelection(combinedSchedules, selectedEmployeeIds),
    [combinedSchedules, selectedEmployeeIds],
  );

  return { selectedEmployeesMeta, combinedSchedules, filteredSchedules };
}
