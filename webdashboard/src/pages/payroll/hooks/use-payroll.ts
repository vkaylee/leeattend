import { useQueryClient } from "@tanstack/react-query";
import { useState, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";

import {
  Employee,
} from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";

import { usePayrollData } from "./use-payroll-data";
import { usePayrollLifecycle } from "./use-payroll-lifecycle";
import { usePayrollPeriod } from "./use-payroll-period";
import { usePayrollSelection } from "./use-payroll-selection";

export function usePayroll() {
  const { t } = useTranslation();

  const queryClient = useQueryClient();

  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [isBatchCalcModalOpen, setIsBatchCalcModalOpen] = useState(false);
  const [isAdjustmentDrawerOpen, setIsAdjustmentDrawerOpen] = useState(false);
  const [isDetailDrawerOpen, setIsDetailDrawerOpen] = useState(false);

  const [debouncedSearchTerm, setDebouncedSearchTerm] = useState("");
  useEffect(() => {
    const timer = setTimeout(() => { setDebouncedSearchTerm(searchTerm); }, 300);
    return () => { clearTimeout(timer); };
  }, [searchTerm]);

  const { calcMonth, calcYear, changeMonth, setCalcMonth, setCalcYear } = usePayrollPeriod();

  const {
    employeeAdjustments,
    employeePayslips,
    employeePayslipsLoading,
    employees,
    empsLoading,
    organizationPayslips,
    organizationPayslipsLoading,
  } = usePayrollData({
    calcMonth,
    calcYear,
    debouncedSearchTerm,
    isDetailDrawerOpen,
    selectedEmployee,
  });

  const { selectedIds, setSelectedIds, toggleSelect, toggleSelectAll } = usePayrollSelection(employees);

  const {
    audit: lifecycleAudit,
    isLocked,
    lockPeriod,
    status: lifecycleStatus,
    unlockPeriod,
  } = usePayrollLifecycle({
    calcMonth,
    calcYear,
    hasCalculatedPayslips: organizationPayslips.length > 0,
  });
  const handleViewDetail = (employee: Employee) => {
    setSelectedEmployee(employee);
    setIsDetailDrawerOpen(true);
  };

  const employeeNames = useMemo(() => {
    const map: Record<string, string> = {};
    employees.forEach(e => { map[e.id] = e.full_name; });
    return map;
  }, [employees]);

  const handleAdjustmentResolved = () => {
    void queryClient.invalidateQueries({ queryKey: createQueryKey("payslips", selectedEmployee?.id) });
    void queryClient.invalidateQueries({ queryKey: createQueryKey("organization-payslips") });
    void queryClient.invalidateQueries({ queryKey: createQueryKey("adjustments") });
  };

  return {
    t,
    selectedEmployee,
    setSelectedEmployee,
    selectedIds,
    setSelectedIds,
    searchTerm,
    setSearchTerm,
    calcYear,
    setCalcYear,
    calcMonth,
    setCalcMonth,
    isBatchCalcModalOpen,
    setIsBatchCalcModalOpen,
    isAdjustmentDrawerOpen,
    setIsAdjustmentDrawerOpen,
    isDetailDrawerOpen,
    setIsDetailDrawerOpen,
    employees,
    empsLoading,
    organizationPayslips,
    organizationPayslipsLoading,
    employeePayslips,
    employeePayslipsLoading,
    employeeAdjustments,
    toggleSelectAll,
    toggleSelect,
    handleViewDetail,
    employeeNames,
    handleAdjustmentResolved,
    changeMonth,
    isLocked,
    lifecycleAudit,
    lifecycleStatus,
    lockPeriod,
    unlockPeriod,
  };
}
