import { useQuery, keepPreviousData } from "@tanstack/react-query";
import { useMemo } from "react";

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
  const { data: employeesData, isLoading: empsLoading } = useQuery({
    queryKey: ["employees", wsId, debouncedSearchTerm],
    queryFn: () => {
      if (debouncedSearchTerm.trim()) {
        return searchEmployees(wsId, debouncedSearchTerm, 1, 100);
      }
      return fetchEmployees(wsId, 1, 3000);
    },
    enabled: !!wsId,
    placeholderData: keepPreviousData,
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

  const { data: allAdjustments = [] } = useQuery({
    queryKey: ["adjustments", wsId, "pending"],
    queryFn: () => fetchWorkspaceAttendanceAdjustments(wsId, "pending"),
    enabled: !!wsId,
    placeholderData: keepPreviousData,
  });

  const employeeAdjustments = useMemo(() => {
    if (!selectedEmployee) return [];
    return allAdjustments.filter(adj => adj.employee_id === selectedEmployee.id);
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
