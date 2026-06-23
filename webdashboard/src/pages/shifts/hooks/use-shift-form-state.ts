import { useSearch } from "@tanstack/react-router";
import { useState } from "react";

import { format } from "date-fns";

import type { Shift, Employee } from "@/lib/api/types";

import type { TabType, ViewMode } from "./use-shifts-types";

export function useShiftFormState() {
  const search = useSearch({ strict: false }) as unknown as { tab?: string };
  const tab = typeof search.tab === "string" ? search.tab : "";
  const initialTab = tab === "weekly-roster" || tab === "assignments" || tab === "directory" ? tab : "weekly-roster";
  const [activeTab, setActiveTab] = useState<TabType>(initialTab);
  const [viewMode, setViewMode] = useState<ViewMode>("week");

  
  // Selection states
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [selectedEmployees, setSelectedEmployees] = useState<Employee[]>([]);
  const [selectedDays, setSelectedDays] = useState<string[]>([]);
  const [targetDates, setTargetDates] = useState<string[]>([]);
  const [isShiftPickerOpen, setIsShiftPickerOpen] = useState(false);
  const [isPatternModalOpen, setIsPatternModalOpen] = useState(false);

  // Fixed Assignment Form State
  const [selectedPatternId, setSelectedPatternId] = useState("");
  const [startDate, setStartDate] = useState(format(new Date(), 'yyyy-MM-01'));
  const [endDate, setEndDate] = useState<string>("");
  const [isForever, setIsForever] = useState(true);

  // Create Shift Modal State
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [newShiftName, setNewShiftName] = useState("");
  const [newShiftSymbol, setNewShiftSymbol] = useState("");
  const [newShiftStart, setNewShiftStart] = useState("08:00");
  const [newShiftEnd, setNewShiftEnd] = useState("17:00");
  const [newShiftBreak, setNewShiftBreak] = useState(60);
  const [newShiftGrace, setNewShiftGrace] = useState(15);
  const [newBreakStart, setNewBreakStart] = useState("");
  const [newBreakEnd, setNewBreakEnd] = useState("");
  const [isFixedBreak, setIsFixedBreakState] = useState(false);
  const [isAutoDeduct, setIsAutoDeduct] = useState(true);
  const [isBreakOvertime, setIsBreakOvertime] = useState(false);
  const [allowEarlyOt, setAllowEarlyOt] = useState(false);
  const [maxEarlyOtMinutes, setMaxEarlyOtMinutes] = useState(60);

  const setIsFixedBreak = (val: boolean) => {
    setIsFixedBreakState(val);
    if (!val && isBreakOvertime) {
      setIsBreakOvertime(false);
      setIsAutoDeduct(true);
    }
  };

  // Edit Shift State
  const [editingShift, setEditingShift] = useState<Shift | null>(null);
  const [isCorrection, setIsCorrection] = useState(false);
  const [correctionEffectiveDate, setCorrectionEffectiveDate] = useState(format(new Date(), 'yyyy-MM-dd'));

  return {
    activeTab, setActiveTab,
    viewMode, setViewMode,
    selectedEmployee, setSelectedEmployee,
    selectedDays, setSelectedDays,
    targetDates, setTargetDates,
    isShiftPickerOpen, setIsShiftPickerOpen,
    isPatternModalOpen, setIsPatternModalOpen,
    selectedPatternId, setSelectedPatternId,
    startDate, setStartDate,
    endDate, setEndDate,
    isForever, setIsForever,
    isCreateModalOpen, setIsCreateModalOpen,
    newShiftName, setNewShiftName,
    newShiftSymbol, setNewShiftSymbol,
    newShiftStart, setNewShiftStart,
    newShiftEnd, setNewShiftEnd,
    newShiftBreak, setNewShiftBreak,
    newShiftGrace, setNewShiftGrace,
    newBreakStart, setNewBreakStart,
    newBreakEnd, setNewBreakEnd,
    isFixedBreak, setIsFixedBreak,
    isAutoDeduct, setIsAutoDeduct,
    isBreakOvertime, setIsBreakOvertime,
    allowEarlyOt, setAllowEarlyOt,
    maxEarlyOtMinutes, setMaxEarlyOtMinutes,
    editingShift, setEditingShift,
    isCorrection, setIsCorrection,
    correctionEffectiveDate, setCorrectionEffectiveDate,
    selectedEmployeeIds, setSelectedEmployeeIds,
    selectedEmployees, setSelectedEmployees
  };
}
