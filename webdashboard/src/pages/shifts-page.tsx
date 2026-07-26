import { Link } from "@tanstack/react-router";
import { Plus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { useMonthlyRoster } from "@/components/shifts/hooks/use-monthly-roster";
import { RosterToolbar } from "@/components/shifts/roster-toolbar";
import { Typography } from "@/components/ui/typography";
import { ROUTES } from "@/lib/routes";

import { ShiftsContent } from "./shifts/content";
import { EmptyEmployeeState } from "./shifts/employee-selector";
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
      <div>
        <Typography className="font-bold mb-0.5 normal-case"
          data-testid="shifts-title"
          variant="h3"
        >
          {t("shifts.title")}
        </Typography>
        <Typography className="text-xs text-slate-500 dark:text-slate-400 font-medium opacity-80" variant="small">
          {t("shifts.subtitle")}
        </Typography>
      </div>

      <div className="flex flex-wrap items-center justify-between gap-3">
        <ShiftsTabs
          activeTab={activeTab}
          setActiveTab={setActiveTab}
          setViewMode={setViewMode}
          viewMode={viewMode}
        />
        <Link className="bg-primary-600 hover:bg-primary-700 text-white font-bold px-5 py-2 rounded-xl flex items-center gap-2 shadow-md transition-all text-center"
          to={ROUTES.SHIFTS_NEW}
        >
          <Plus className="w-5 h-5" />
          <span className="text-xs uppercase tracking-widest">
            {t("shifts.create_btn")}
          </span>
        </Link>
      </div>

      {activeTab === "weekly-roster" && (
        <RosterToolbar 
          hook={hook} 
          isCopying={monthlyRoster.isCopying}
          isMutating={viewMode === "month" ? monthlyRoster.isMutating : hook.bulkRosterMutation.isPending}
          onApplyShift={viewMode === "month" ? monthlyRoster.handleApplyShift : hook.handleApplyShift}
          onAssignSelected={() => { monthlyRoster.setIsShiftPickerOpen(true); }}
          onCancelSelection={() => { monthlyRoster.setSelectedCells(new Set()); }}
          onClearOverrides={monthlyRoster.handleClearSelectedOverrides}
          onClearWeekSelection={() => { hook.setSelectedDays([]); }}
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

