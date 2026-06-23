import React from "react";
import { useTranslation } from "react-i18next";

import { useMonthlyRoster } from "@/components/shifts/hooks/use-monthly-roster";
import { RosterToolbar } from "@/components/shifts/roster-toolbar";

import { ShiftsContent } from "./shifts/content";
import { EmptyEmployeeState } from "./shifts/employee-selector";
import { ShiftsHeader } from "./shifts/header";
import { useShifts } from "./shifts/hooks/use-shifts";
import { ShiftsModals } from "./shifts/modals";
import { ShiftsTabs } from "./shifts/tabs";

export default function ShiftsPage() {
  const { t } = useTranslation();
  const hook = useShifts();
  const monthlyRoster = useMonthlyRoster({ hook });
  
  const {
    activeTab,
    selectedEmployee,
    setActiveTab,
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

      {activeTab === "weekly-roster" && (
        <RosterToolbar 
          hook={hook} 
          isCopying={monthlyRoster.isCopying}
          isMutating={monthlyRoster.isMutating}
          onApplyShift={monthlyRoster.handleApplyShift}
          onAssignSelected={() => monthlyRoster.setIsShiftPickerOpen(true)}
          onCancelSelection={() => monthlyRoster.setSelectedCells(new Set())}
          onClearOverrides={monthlyRoster.handleClearSelectedOverrides}
          onClearWeekSelection={() => hook.setSelectedDays([])}
          onCopyMonth={monthlyRoster.handleCopyMonth}
          onOpenWeekShiftPicker={(dates: string[]) => {
            hook.setTargetDates(dates);
            hook.setIsShiftPickerOpen(true);
          }}
          selectedCells={monthlyRoster.selectedCells}
          selectedDays={hook.selectedDays}
          selectedEmployeesMeta={monthlyRoster.selectedEmployeesMeta}
          viewMode={viewMode}
        />
      )}

      <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
        {!selectedEmployee && activeTab !== "directory" && viewMode === "week" ? (
          <EmptyEmployeeState />
        ) : (
          <ShiftsContent hook={hook} monthlyRoster={monthlyRoster} />
        )}
      </div>

      <ShiftsModals hook={hook} />
    </div>
  );
}

