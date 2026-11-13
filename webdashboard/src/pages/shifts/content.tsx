import React from "react";

import { AssignmentHistory } from "@/components/shifts/assignment-history";
import { useMonthlyRoster } from "@/components/shifts/hooks/use-monthly-roster";
import { MonthlyRosterGrid } from "@/components/shifts/monthly-roster-grid";
import { PriorityLegend } from "@/components/shifts/priority-legend";
import { ShiftList } from "@/components/shifts/shift-list";
import { WeeklyRosterGrid } from "@/components/shifts/weekly-roster-grid";
import { ScheduleLifecyclePanel } from "@/components/shifts/schedule-lifecycle-panel";

import { useShifts } from "./hooks/use-shifts";

interface ShiftsContentProps {
  hook: ReturnType<typeof useShifts>;
  monthlyRoster: ReturnType<typeof useMonthlyRoster>;
}

 
export function ShiftsContent({ hook, monthlyRoster }: ShiftsContentProps) {
  const {
    activeTab,
    viewMode,
    currentDate,
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
    return <MonthlyRosterGrid hook={hook} monthlyRoster={monthlyRoster} />;
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
