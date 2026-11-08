import React from "react";

import { PageContainer } from "@/components/ui/page-container";

import { LeaveBalancesFilters } from "./leave-balances/components/leave-balances-filters";
import { LeaveBalancesHeader } from "./leave-balances/components/leave-balances-header";
import { LeaveBalancesModals } from "./leave-balances/components/leave-balances-modals";
import { LeaveBalancesPagination } from "./leave-balances/components/leave-balances-pagination";
import { LeaveBalancesTable } from "./leave-balances/components/leave-balances-table";
import { useLeaveBalances } from "./leave-balances/hooks/use-leave-balances";

export default function LeaveBalancesPage() {
  const {
    searchTerm,
    setSearchTerm,
    page,
    setPage,
    resetPagination,
    selectedYear,
    setSelectedYear,
    selectedEmployee,
    setSelectedEmployee,
    selectedBalance,
    setSelectedBalance,
    isLoading,
    balances,
    totalItems,
    totalPages,
    limit,
    accrualMutation,
    handleRunAccrual,
    ConfirmationDialog,
  } = useLeaveBalances();

  return (
    <PageContainer>
      <LeaveBalancesHeader
        isPending={accrualMutation.isPending}
        onRunAccrual={handleRunAccrual}
        selectedYear={selectedYear}
      />

      <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-3xl md:rounded-[3rem] overflow-hidden shadow-sm animate-in slide-in-from-bottom-8 duration-700">
        <LeaveBalancesFilters
          searchTerm={searchTerm}
          selectedYear={selectedYear}
          setSearchTerm={(term) => {
            setSearchTerm(term);
            resetPagination();
          }}
          setSelectedYear={(year) => {
            setSelectedYear(year);
            resetPagination();
          }}
          totalItems={totalItems}
        />

        <LeaveBalancesTable
          balances={balances}
          isLoading={isLoading}
          onViewDetails={setSelectedBalance}
          onViewHistory={setSelectedEmployee}
        />

        <LeaveBalancesPagination
          limit={limit}
          onPageChange={setPage}
          page={page}
          totalItems={totalItems}
          totalPages={totalPages}
        />
      </div>

      <LeaveBalancesModals selectedBalance={selectedBalance}
      selectedEmployee={selectedEmployee}
      setSelectedBalance={setSelectedBalance}
      setSelectedEmployee={setSelectedEmployee}  />

      <ConfirmationDialog />
    </PageContainer>
  );
}