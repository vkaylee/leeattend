import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { useWorkspaceQuery } from "@/hooks/common/use-workspace-query";
import { employeeKeys } from "@/lib/query-keys";

import { MAX_EMPLOYEE_FETCH } from "@/hooks/use-employees";
import {
  fetchEmployees,
  searchEmployees,
  fetchEmployeePayslips,
  fetchWorkspacePayslips,
  fetchWorkspaceAttendanceAdjustments,
  Employee,
} from "@/lib/api";

interface UsePayrollDataOptions {
  calcMonth: number;
  calcYear: number;
  debouncedSearchTerm: string;
  isDetailDrawerOpen: boolean;
  selectedEmployee: Employee | null;
  wsId: string;
}

export function usePayrollData({
  calcMonth,
  calcYear,
  debouncedSearchTerm,
  isDetailDrawerOpen,
  selectedEmployee,
  wsId,
}: UsePayrollDataOptions) {
  const { data: employeesData, isLoading: empsLoading } = useWorkspaceQuery({
    workspaceId: wsId,
    queryKey: employeeKeys.search(wsId, { search: debouncedSearchTerm }),
    queryFn: async (workspaceId) => {
      const res = debouncedSearchTerm.trim()
        ? await searchEmployees({ wsId: workspaceId, query: debouncedSearchTerm, page: 1, limit: 100 })
        : await fetchEmployees(workspaceId, 1, MAX_EMPLOYEE_FETCH);
      const maybeRes = ((): typeof res | undefined => res)();
      return maybeRes ?? null;
    },
    placeholderData: "keepPreviousData",
  });

  const employees = useMemo(() => employeesData?.data ?? [], [employeesData?.data]);

  const { data: workspacePayslipsData = [], isLoading: workspacePayslipsLoading } = useQuery({
    queryKey: ["workspace-payslips", wsId, calcYear, calcMonth],
    queryFn: () => fetchWorkspacePayslips(wsId, calcYear, calcMonth),
    enabled: !!wsId,
    placeholderData: keepPreviousData,
  });

  const { data: employeePayslips = [], isLoading: employeePayslipsLoading } = useQuery({
    queryKey: ["payslips", selectedEmployee?.id],
    queryFn: () => {
      if (!selectedEmployee) throw new Error("Employee is required");
      return fetchEmployeePayslips(wsId, selectedEmployee.id);
    },
    enabled: !!selectedEmployee && isDetailDrawerOpen,
  });

  const { data: allAdjustmentsResponse } = useQuery({
    queryKey: ["adjustments", wsId, "pending"],
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId, { status: "pending" }),
    enabled: !!wsId,
    placeholderData: keepPreviousData,
  });

  const allAdjustments = useMemo(() => allAdjustmentsResponse?.data ?? [], [allAdjustmentsResponse]);

  const employeeAdjustments = useMemo(() => {
    if (!selectedEmployee) return [];
    return allAdjustments.filter(adj => adj.employment_id === selectedEmployee.active_employment_id);
  }, [selectedEmployee, allAdjustments]);

  return {
    allAdjustments,
    employeeAdjustments,
    employeePayslips,
    employeePayslipsLoading,
    employees,
    empsLoading,
    workspacePayslips: workspacePayslipsData,
    workspacePayslipsLoading,
  };
}
