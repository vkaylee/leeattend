import { useQueryClient } from "@tanstack/react-query";
import { useState, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";

import {
  getStandaloneOrganizationId,
  Employee,
} from "@/lib/api";

import { usePayrollData } from "./use-payroll-data";
import { usePayrollLifecycle } from "./use-payroll-lifecycle";
import { usePayrollPeriod } from "./use-payroll-period";
import { usePayrollSelection } from "./use-payroll-selection";
export function usePayroll() {
  const { t } = useTranslation();
  const wsId = getStandaloneOrganizationId() ?? "";
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
    workspacePayslips,
    workspacePayslipsLoading,
  } = usePayrollData({
    calcMonth,
    calcYear,
    debouncedSearchTerm,
    isDetailDrawerOpen,
    selectedEmployee,
    wsId,
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
    hasCalculatedPayslips: workspacePayslips.length > 0,
    wsId,
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
    void queryClient.invalidateQueries({ queryKey: ["payslips", selectedEmployee?.id] });
    void queryClient.invalidateQueries({ queryKey: ["workspace-payslips"] });
    void queryClient.invalidateQueries({ queryKey: ["adjustments"] });
  };

  return {
    t,
    wsId,
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
    workspacePayslips,
    workspacePayslipsLoading,
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
