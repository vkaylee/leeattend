import React from "react";

import { format } from "date-fns";

import { useMonthlyRoster } from "@/components/shifts/hooks/use-monthly-roster";
import { AssignmentHistory } from "@/components/shifts/assignment-history";
import { MonthlyRosterGrid } from "@/components/shifts/monthly-roster-grid";
import { PriorityLegend } from "@/components/shifts/priority-legend";
import { ShiftList } from "@/components/shifts/shift-list";
import { WeeklyRosterGrid } from "@/components/shifts/weekly-roster-grid";

import { useShifts } from "./hooks/use-shifts";

interface ShiftsContentProps {
  hook: ReturnType<typeof useShifts>;
  monthlyRoster: ReturnType<typeof useMonthlyRoster>;
}

 
export function ShiftsContent({ hook, monthlyRoster }: ShiftsContentProps) {
  const {
    activeTab,
    viewMode,
    assignPatternMutation,
    currentDate,
    deleteRosterMutation,
    employeeSchedule,
    endDate,
    handleAssign,
    handleAssignAllWeek,
    handleCopyPreviousWeek,
    handleNavigateWeek,
    history,
    isForever,
    openEditModal,
    patterns,
    selectedDays,
    selectedEmployee,
    selectedPatternId,
    setIsForever,
    setIsShiftPickerOpen,
    setSelectedDays,
    setSelectedPatternId,
    setStartDate,
    setTargetDates,
    setEndDate,
    shifts,
    startDate,
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
          weekDays={weekDays}
          shifts={shifts}
        />
        <PriorityLegend />
        <AssignmentHistory
          history={history}
          selectedEmployee={selectedEmployee}
        />
      </div>
    );
  }

  return <ShiftList onEdit={openEditModal} shifts={shifts} />;
}
