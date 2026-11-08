import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { useOrganizationQuery } from "@/hooks/common/use-organization-query";
import { employeeKeys } from "@/lib/query-keys";

import { MAX_EMPLOYEE_FETCH } from "@/hooks/use-employees";
import {
  fetchEmployees,
  searchEmployees,
  fetchEmployeePayslips,
  fetchPayslips,
  fetchWorkspaceAttendanceAdjustments,
  Employee,
} from "@/lib/api";

interface UsePayrollDataOptions { calcMonth: number;
calcYear: number;
debouncedSearchTerm: string;
isDetailDrawerOpen: boolean;
selectedEmployee: Employee | null;  }

export function usePayrollData({
  calcMonth,
  calcYear,
  debouncedSearchTerm,
  isDetailDrawerOpen,
  selectedEmployee,
}: UsePayrollDataOptions) {
  const { data: employeesData, isLoading: empsLoading } = useOrganizationQuery({ queryKey: employeeKeys.search({ search: debouncedSearchTerm }),
  queryFn: async () => {
    const res = debouncedSearchTerm.trim()
      ? await searchEmployees({ query: debouncedSearchTerm, page: 1, limit: 100 })
      : await fetchEmployees(1, MAX_EMPLOYEE_FETCH);
    const maybeRes = ((): typeof res | undefined => res)();
    return maybeRes ?? null;
  },
  placeholderData: "keepPreviousData", });

  const employees = useMemo(() => employeesData?.data ?? [], [employeesData?.data]);

  const { data: workspacePayslipsData = [], isLoading: workspacePayslipsLoading } = useQuery({
    queryKey: ["workspace-payslips", calcYear, calcMonth],
    queryFn: () => fetchPayslips(calcYear, calcMonth),
    placeholderData: keepPreviousData,
  });

  const { data: employeePayslips = [], isLoading: employeePayslipsLoading } = useQuery({
    queryKey: ["payslips", selectedEmployee?.id],
    queryFn: () => {
      if (!selectedEmployee) throw new Error("Employee is required");
      return fetchEmployeePayslips(selectedEmployee.id);
    },
    enabled: !!selectedEmployee && isDetailDrawerOpen,
  });

  const { data: allAdjustmentsResponse } = useQuery({
    queryKey: ["adjustments", "pending"],
    queryFn: () => fetchWorkspaceAttendanceAdjustments({ status: "pending" }),
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
