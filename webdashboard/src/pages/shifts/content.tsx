import React from "react";

import { format } from "date-fns";

import { AssignmentHistory } from "@/components/shifts/assignment-history";
import { MonthlyRosterGrid } from "@/components/shifts/monthly-roster-grid";
import { PatternAssignmentForm } from "@/components/shifts/pattern-assignment-form";
import { PriorityLegend } from "@/components/shifts/priority-legend";
import { ShiftList } from "@/components/shifts/shift-list";
import { WeeklyRosterGrid } from "@/components/shifts/weekly-roster-grid";

import { useShifts } from "./hooks/use-shifts";

interface ShiftsContentProps {
  hook: ReturnType<typeof useShifts>;
}

 
export function ShiftsContent({ hook }: ShiftsContentProps) {
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
    return <MonthlyRosterGrid hook={hook} />;
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
          onAssignAllWeek={() => {
            setTargetDates(weekDays.map((d) => format(d, "yyyy-MM-dd")));
            setIsShiftPickerOpen(true);
          }}
          onClearSelection={() => {
            setSelectedDays([]);
          }}
          onClearWeek={() => {
            handleAssignAllWeek(null);
          }}
          onCopyPreviousWeek={() => {
            void handleCopyPreviousWeek();
          }}
          onDeleteRoster={(date) => {
            deleteRosterMutation.mutate(date);
          }}
          onNavigateWeek={handleNavigateWeek}
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
        />
        <PriorityLegend />
        <AssignmentHistory
          history={history}
          selectedEmployee={selectedEmployee}
        />
      </div>
    );
  }

  if (activeTab === "assignments" && selectedEmployee) {
    return (
      <PatternAssignmentForm
        endDate={endDate}
        isForever={isForever}
        isLoading={assignPatternMutation.isPending}
        onAssign={handleAssign}
        patterns={patterns}
        selectedEmployee={selectedEmployee}
        selectedPatternId={selectedPatternId}
        setEndDate={setEndDate}
        setIsForever={setIsForever}
        setSelectedPatternId={setSelectedPatternId}
        setStartDate={setStartDate}
        startDate={startDate}
      />
    );
  }

  return <ShiftList onEdit={openEditModal} shifts={shifts} />;
}
