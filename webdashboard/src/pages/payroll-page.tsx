import React from "react";

import PayrollSummaryTable from "@/components/payroll/payroll-summary-table";

import { PayrollHeader } from "./payroll/components/payroll-header";
import { PayrollModals } from "./payroll/components/payroll-modals";
import { usePayroll } from "./payroll/hooks/use-payroll";

export default function PayrollPage() {
  const {
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
  } = usePayroll();

  

  return (
    <div className="flex flex-col h-[calc(100dvh-8rem)] gap-8 overflow-hidden animate-in fade-in duration-500">
      <PayrollHeader
        calcMonth={calcMonth}
        calcYear={calcYear}
        isLocked={isLocked}
        lifecycleStatus={lifecycleStatus}
        lockedAt={lifecycleAudit.lockedAt}
        lockedBy={lifecycleAudit.lockedBy}
        onChangeMonth={changeMonth}
        onMonthYearChange={(m, y) => {
          setCalcMonth(m);
          setCalcYear(y);
        }}
        onOpenBatchCalc={() => { setIsBatchCalcModalOpen(true); }}
        onToggleLock={() => {
          if (isLocked) {
            unlockPeriod();
          } else {
            lockPeriod();
          }
        }}
        selectedCount={selectedIds.size}
      />

      <div className="flex-1 min-h-0">
        <PayrollSummaryTable 
          employees={employees}
          isLoading={empsLoading || organizationPayslipsLoading}
          isLocked={isLocked}
          lockedAt={lifecycleAudit.lockedAt}
          lockedBy={lifecycleAudit.lockedBy}
          onSearchChange={setSearchTerm}
          onToggleSelect={toggleSelect}
          onToggleSelectAll={toggleSelectAll}
          onViewDetail={handleViewDetail}
          payslips={organizationPayslips}
          searchTerm={searchTerm}
          selectedIds={selectedIds}
        />
      </div>

      <PayrollModals adjustments={employeeAdjustments}
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
      selectedIds={selectedIds}  />
    </div>
  );
}