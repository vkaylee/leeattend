import { Link } from "@tanstack/react-router";
import { Plus } from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { useMonthlyRoster } from "@/components/shifts/hooks/use-monthly-roster";
import { RosterToolbar } from "@/components/shifts/roster-toolbar";
import { PageHeader } from "@/components/ui/page-header";
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
    viewMode,
    setViewMode,
  } = hook;

  

  return (
    <div className="space-y-6">
      <PageHeader
        data-testid="shifts-title"
        subtitle={t("shifts.subtitle")}
        title={t("shifts.title")}
      />
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

