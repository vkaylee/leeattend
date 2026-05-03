import FloatingRecordBadge from "@/components/common/floating-record-badge";
import EmployeeDetailDrawer from "@/components/features/employees/employee-detail-drawer";

import { EmployeeFilterChips } from "./employees/components/employee-filter-chips";
import { EmployeeModals } from "./employees/components/employee-modals";
import { EmployeeTable } from "./employees/components/employee-table";
import { EmployeesPageHeader } from "./employees/components/employees-page-header";
import { useEmployeesPage } from "./employees/hooks/use-employees-page";

export default function EmployeesPage() {
  const {
    wsId,
    formatDate,
    isModalOpen,
    setIsModalOpen,
    isImportDialogOpen,
    setIsImportDialogOpen,
    editingEmployee,
    setEditingEmployee,
    selectedEmployee,
    setSelectedEmployee,
    searchTerm,
    setSearchTerm,
    filterDeptIds,
    setFilterDeptIds,
    prefillEnrollmentId,
    setPrefillEnrollmentId,
    ConfirmationDialog,
    isTransferModalOpen,
    setIsTransferModalOpen,
    isHistoryModalOpen,
    setIsHistoryModalOpen,
    isRehireModalOpen,
    setIsRehireModalOpen,
    selectedDeptId,
    setSelectedDeptId,
    sortKey,
    sortDir,
    toggleSort,
    employees,
    totalItems,
    departments,
    toggleDeptFilter,
    deptHistory,
    employmentHistory,
    isLoading,
    isFetching,
    isPlaceholderData,
    fetchNextPage,
    hasNextPage,
    isFetchingNextPage,
    searchInputRef,
    handleDrawerClose,
    handleTerminate,
    t,
  } = useEmployeesPage();

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center py-32">
        <div className="animate-spin rounded-full h-12 w-12 border-4 border-primary-600 border-t-transparent shadow-xl"></div>
      </div>
    );
  }

  return (
    <div className="space-y-10 pb-20 animate-in fade-in duration-700">
      <EmployeesPageHeader
        setEditingEmployee={setEditingEmployee}
        setIsImportDialogOpen={setIsImportDialogOpen}
        setIsModalOpen={setIsModalOpen}
      />

      <EmployeeFilterChips
        departments={departments}
        filterDeptIds={filterDeptIds}
        setFilterDeptIds={setFilterDeptIds}
        toggleDeptFilter={toggleDeptFilter}
      />

      <EmployeeTable
        departments={departments}
        employees={employees}
        fetchNextPage={fetchNextPage}
        filterDeptIds={filterDeptIds}
        hasNextPage={hasNextPage}
        isFetching={isFetching}
        isFetchingNextPage={isFetchingNextPage}
        isLoading={false}
        isPlaceholderData={isPlaceholderData}
        searchInputRef={searchInputRef}
        searchTerm={searchTerm}
        selectedEmployee={selectedEmployee}
        setFilterDeptIds={setFilterDeptIds}
        setSearchTerm={setSearchTerm}
        setSelectedEmployee={setSelectedEmployee}
        sortDir={sortDir}
        sortKey={sortKey}
        toggleDeptFilter={toggleDeptFilter}
        toggleSort={toggleSort}
        totalItems={totalItems}
      />

      <EmployeeDetailDrawer
        departments={departments}
        deptHistory={deptHistory}
        employee={selectedEmployee}
        employmentHistory={employmentHistory}
        formatDate={formatDate}
        isOpen={!!selectedEmployee}
        key={selectedEmployee?.id ?? "closed"}
        onClose={handleDrawerClose}
        onEdit={(emp) => {
          setEditingEmployee(emp);
          setIsModalOpen(true);
        }}
        onRehire={() => { setIsRehireModalOpen(true); }}
        onTerminate={handleTerminate}
        onTransfer={(deptId) => {
          setSelectedDeptId(deptId);
          setIsTransferModalOpen(true);
        }}
        wsId={wsId}
      />

      <EmployeeModals
        departments={departments}
        editingEmployee={editingEmployee}
        employmentHistory={employmentHistory}
        formatDate={formatDate}
        isHistoryModalOpen={isHistoryModalOpen}
        isImportDialogOpen={isImportDialogOpen}
        isModalOpen={isModalOpen}
        isRehireModalOpen={isRehireModalOpen}
        isTransferModalOpen={isTransferModalOpen}
        prefillEnrollmentId={prefillEnrollmentId}
        selectedDeptId={selectedDeptId}
        selectedEmployee={selectedEmployee}
        setIsHistoryModalOpen={setIsHistoryModalOpen}
        setIsImportDialogOpen={setIsImportDialogOpen}
        setIsModalOpen={setIsModalOpen}
        setIsRehireModalOpen={setIsRehireModalOpen}
        setIsTransferModalOpen={setIsTransferModalOpen}
        setPrefillEnrollmentId={setPrefillEnrollmentId}
        setSelectedDeptId={setSelectedDeptId}
        setSelectedEmployee={setSelectedEmployee}
        wsId={wsId}
      />

      <FloatingRecordBadge label={t("common.pagination.records")} loaded={employees.length} total={totalItems} />
      
      <ConfirmationDialog />
    </div>
  );
}
