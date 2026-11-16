import React from "react";
import { useTranslation } from "react-i18next";

import { Combobox } from "@/components/ui/combobox";

import { AssignmentHistory } from "@/components/shifts/assignment-history";
import { useMonthlyRoster } from "@/components/shifts/hooks/use-monthly-roster";
import { MonthlyRosterGrid } from "@/components/shifts/monthly-roster-grid";
import { PriorityLegend } from "@/components/shifts/priority-legend";
import { ScheduleLifecyclePanel } from "@/components/shifts/schedule-lifecycle-panel";
import { ShiftList } from "@/components/shifts/shift-list";
import { WeeklyRosterGrid } from "@/components/shifts/weekly-roster-grid";

import { useShifts } from "./hooks/use-shifts";

interface ShiftsContentProps {
  hook: ReturnType<typeof useShifts>;
  monthlyRoster: ReturnType<typeof useMonthlyRoster>;
}

 
export function ShiftsContent({ hook, monthlyRoster }: ShiftsContentProps) {
  const { t } = useTranslation();
  const {
    activeTab,
    viewMode,
    currentDate,
    coverageFilter,
    setCoverageFilter,
    deleteRosterMutation,
    employeeSchedule,
    history,
    openEditModal,
    selectedDays,
    selectedEmployee,
    setIsShiftPickerOpen,
    setSelectedDays,
    setTargetDates,
    shifts,
    weekDays,
  } = hook;

  if (activeTab === "weekly-roster" && viewMode === "month") {
    return <div className="space-y-4"><div className="flex items-center justify-end"><Combobox aria-label={t("scheduleLifecycle.coverageLabel")} onChange={(value) => { setCoverageFilter(value as "all" | "missing" | "complete"); }} options={[{ label: t("scheduleLifecycle.coverageAll"), value: "all" }, { label: t("scheduleLifecycle.coverageMissing"), value: "missing" }, { label: t("scheduleLifecycle.coverageComplete"), value: "complete" }]} placeholder={t("scheduleLifecycle.coverageLabel")} searchPlaceholder={t("common.combobox.search")} value={coverageFilter} /></div><MonthlyRosterGrid hook={hook} monthlyRoster={monthlyRoster} /></div>;
  }

  if (!selectedEmployee && activeTab !== "directory" && viewMode === "week") {
    return null;
  }

  if (activeTab === "weekly-roster" && selectedEmployee) {
    return (
      <div className="space-y-6">
        <WeeklyRosterGrid
          currentDate={currentDate}
          employeeSchedule={employeeSchedule}
          onClearSelection={() => {
            setSelectedDays([]);
          }}
          onDeleteRoster={(date) => {
            deleteRosterMutation.mutate(date);
          }}
          onOpenShiftPicker={(dates) => {
            setTargetDates(dates);
            setIsShiftPickerOpen(true);
          }}
          onToggleDaySelection={(dateString) => {
            if (selectedDays.includes(dateString)) {
              setSelectedDays(selectedDays.filter((d) => d !== dateString));
            } else {
              setSelectedDays([...selectedDays, dateString]);
            }
          }}
          selectedDays={selectedDays}
          selectedEmployee={selectedEmployee}
          shifts={shifts}
          weekDays={weekDays}
        />
        <PriorityLegend />
        {employeeSchedule ? <ScheduleLifecyclePanel employeeSchedule={employeeSchedule} weekDays={weekDays} /> : null}
        <AssignmentHistory
          history={history}
          selectedEmployee={selectedEmployee}
        />
      </div>
    );
  }

  return <ShiftList onEdit={openEditModal} shifts={shifts} />;
}
