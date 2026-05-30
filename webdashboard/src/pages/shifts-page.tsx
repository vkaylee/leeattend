import React from "react";
import { useTranslation } from "react-i18next";

import { ShiftsContent } from "./shifts/content";
import { EmployeeSelector, EmptyEmployeeState } from "./shifts/employee-selector";
import { ShiftsHeader } from "./shifts/header";
import { useShifts } from "./shifts/hooks/use-shifts";
import { ShiftsModals } from "./shifts/modals";
import { ShiftsTabs } from "./shifts/tabs";

export default function ShiftsPage() {
  const { t } = useTranslation();
  const hook = useShifts();
  const {
    activeTab,
    selectedEmployee,
    setActiveTab,
    setIsCreateModalOpen,
    setSelectedEmployee,
    wsId,
    viewMode,
    setViewMode,
  } = hook;

  if (!wsId) {
    return (
      <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">
        {t("common.select_workspace")}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <ShiftsHeader />

      <ShiftsTabs 
        activeTab={activeTab} 
        setActiveTab={setActiveTab} 
        setViewMode={setViewMode}
        viewMode={viewMode}
      />

      <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
        {activeTab !== "directory" && viewMode === "week" ? (
          <EmployeeSelector
            onSelect={setSelectedEmployee}
            selectedEmployeeId={selectedEmployee?.id}
            wsId={wsId}
          />
        ) : null}

        {!selectedEmployee && activeTab !== "directory" && viewMode === "week" ? (
          <EmptyEmployeeState />
        ) : (
          <ShiftsContent hook={hook} />
        )}
      </div>

      <ShiftsModals hook={hook} />
    </div>
  );
}
