import { keepPreviousData, useQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { useOrganizationQuery } from "@/hooks/common/use-organization-query";
import { MAX_EMPLOYEE_FETCH } from "@/hooks/use-employees";
import {
  fetchEmployees,
  searchEmployees,
  fetchEmployeePayslips,
  fetchPayslips,
  fetchAttendanceAdjustments,
  Employee,
} from "@/lib/api";
import { employeeKeys, createQueryKey  } from "@/lib/query-keys";


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

  const { data: organizationPayslipsData = [], isLoading: organizationPayslipsLoading } = useQuery({
    queryKey: createQueryKey("organization-payslips", calcYear, calcMonth),
    queryFn: () => fetchPayslips(calcYear, calcMonth),
    placeholderData: keepPreviousData,
  });

  const { data: employeePayslips = [], isLoading: employeePayslipsLoading } = useQuery({
    queryKey: createQueryKey("payslips", selectedEmployee?.id),
    queryFn: () => {
      if (!selectedEmployee) throw new Error("Employee is required");
      return fetchEmployeePayslips(selectedEmployee.id);
    },
    enabled: !!selectedEmployee && isDetailDrawerOpen,
  });

  const { data: allAdjustmentsResponse } = useQuery({
    queryKey: createQueryKey("adjustments", "pending"),
    queryFn: () => fetchAttendanceAdjustments({ status: "pending" }),
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
    organizationPayslips: organizationPayslipsData,
    organizationPayslipsLoading,
  };
}
