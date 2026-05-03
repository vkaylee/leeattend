import React from "react";

import PayrollSummaryTable from "@/components/payroll/payroll-summary-table";

import { PayrollHeader } from "./payroll/components/payroll-header";
import { PayrollModals } from "./payroll/components/payroll-modals";
import { usePayroll } from "./payroll/hooks/use-payroll";

export default function PayrollPage() {
  const {
    t,
    wsId,
    selectedEmployee,
    selectedIds,
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
  } = usePayroll();

  if (!wsId) return <div className="p-8 text-center">{t("common.select_workspace")}</div>;

  return (
    <div className="flex flex-col h-[calc(100vh-8rem)] gap-8 overflow-hidden animate-in fade-in duration-500">
      <PayrollHeader
        calcMonth={calcMonth}
        calcYear={calcYear}
        onChangeMonth={changeMonth}
        onMonthYearChange={(m, y) => {
          setCalcMonth(m);
          setCalcYear(y);
        }}
        onOpenBatchCalc={() => { setIsBatchCalcModalOpen(true); }}
        selectedCount={selectedIds.size}
      />

      <div className="flex-1 min-h-0">
        <PayrollSummaryTable 
          employees={employees}
          isLoading={empsLoading || workspacePayslipsLoading}
          onSearchChange={setSearchTerm}
          onToggleSelect={toggleSelect}
          onToggleSelectAll={toggleSelectAll}
          onViewDetail={handleViewDetail}
          payslips={workspacePayslips}
          searchTerm={searchTerm}
          selectedIds={selectedIds}
        />
      </div>

      <PayrollModals
        adjustments={employeeAdjustments}
        employee={selectedEmployee}
        employeeNames={employeeNames}
        isAdjustmentDrawerOpen={isAdjustmentDrawerOpen}
        isBatchCalcModalOpen={isBatchCalcModalOpen}
        isDetailDrawerOpen={isDetailDrawerOpen}
        isLoadingPayslips={employeePayslipsLoading}
        onAdjustmentResolved={handleAdjustmentResolved}
        onCloseAdjustmentDrawer={() => { setIsAdjustmentDrawerOpen(false); }}
        onCloseBatchCalcModal={() => { setIsBatchCalcModalOpen(false); }}
        onCloseDetailDrawer={() => { setIsDetailDrawerOpen(false); }}
        onOpenAdjustmentDrawer={() => { setIsAdjustmentDrawerOpen(true); }}
        payslips={employeePayslips}
        selectedIds={selectedIds}
        workspaceId={wsId}
      />
    </div>
  );
}