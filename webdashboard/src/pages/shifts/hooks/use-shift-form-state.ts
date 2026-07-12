import { useQuery } from "@tanstack/react-query";
import { useSearch, useNavigate } from "@tanstack/react-router";
import { useState, useEffect } from "react";

import { format } from "date-fns";

import type { Shift, Employee } from "@/lib/api/types";

import { fetchEmployee } from "@/lib/api";

import type { TabType, ViewMode } from "./use-shifts-types";

interface ShiftSearchParams {
  employeeId?: string;
  employeeIds?: string;
  tab?: string;
  view?: string;
}

// eslint-disable-next-line complexity
export function useShiftFormState(wsId?: string) {
  // useSearch({ strict: false }) returns a permissive type; we cast to
  // ShiftSearchParams to document fields used by this hook (codebase idiom
  // — see shifts-rules-page.tsx).
  const rawSearch = useSearch({ strict: false });
  const search = rawSearch as ShiftSearchParams;
  const navigate = useNavigate();

  const tab = typeof search.tab === "string" ? search.tab : "";
  const initialTab = tab === "weekly-roster" || tab === "assignments" || tab === "directory" ? tab : "weekly-roster";
  const [activeTab, setActiveTabState] = useState<TabType>(initialTab);
  
  const initialView = search.view === "week" || search.view === "month" ? search.view : "week";
  const [viewMode, setViewModeState] = useState<ViewMode>(initialView);

  // Selection states
  const [selectedEmployee, setSelectedEmployeeState] = useState<Employee | null>(null);
  
  const initialIds = typeof search.employeeIds === "string" && search.employeeIds ? search.employeeIds.split(",") : [];
  const [selectedEmployeeIds, setSelectedEmployeeIdsState] = useState<string[]>(initialIds);
  const [selectedEmployees, setSelectedEmployeesState] = useState<Employee[]>([]);
  
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

  // Load initial selected employee from URL query parameter if present on mount
  const { data: initialEmployee } = useQuery({
    queryKey: ["employee", wsId, search.employeeId],
    queryFn: () => {
      if (!wsId || !search.employeeId) throw new Error("Missing params");
      return fetchEmployee(wsId, search.employeeId);
    },
    enabled: !!wsId && !!search.employeeId && !selectedEmployee,
  });

  useEffect(() => {
    if (initialEmployee && !selectedEmployee) {
      setSelectedEmployeeState(initialEmployee);
    }
  }, [initialEmployee, selectedEmployee]);

  // Reactive state synchronization
  // 1. In Week View: sync selectedEmployee -> selectedEmployeeIds & selectedEmployees
  useEffect(() => {
    if (viewMode === "week") {
      if (selectedEmployee) {
        setSelectedEmployeeIdsState([selectedEmployee.id]);
        setSelectedEmployeesState([selectedEmployee]);
      } else {
        setSelectedEmployeeIdsState([]);
        setSelectedEmployeesState([]);
      }
    }
  }, [selectedEmployee, viewMode]);

  // 2. In Month View: sync selectedEmployeeIds -> selectedEmployee
  useEffect(() => {
    if (viewMode === "month") {
      if (selectedEmployeeIds.length > 0) {
        if (!selectedEmployee || !selectedEmployeeIds.includes(selectedEmployee.id)) {
          const match = selectedEmployees.find((e: Employee) => e.id === selectedEmployeeIds[0]);
          if (match) {
            setSelectedEmployeeState(match);
          }
        }
      } else {
        setSelectedEmployeeState(null);
      }
    }
  }, [selectedEmployeeIds, selectedEmployees, viewMode, selectedEmployee]);

  // Custom setters that update URL query params
  const setActiveTab = (val: TabType) => {
    setActiveTabState(val);
    void navigate({
      to: "/shifts",
      search: { tab: val, employeeId: search.employeeId, employeeIds: search.employeeIds },
    });
  };

  const setViewMode = (val: ViewMode) => {
    setViewModeState(val);
    void navigate({
      to: "/shifts",
      search: { view: val, employeeId: search.employeeId, employeeIds: search.employeeIds },
    });
  };

  const setSelectedEmployee = (emp: Employee | null) => {
    setSelectedEmployeeState(emp);
    void navigate({
      to: "/shifts",
      search: { tab: search.tab, employeeId: emp?.id, employeeIds: search.employeeIds },
    });
  };

  const setSelectedEmployeeIds = (ids: string[]) => {
    setSelectedEmployeeIdsState(ids);
    void navigate({
      to: "/shifts",
      search: { tab: search.tab, employeeId: search.employeeId, employeeIds: ids.length > 0 ? ids.join(",") : undefined },
    });
  };

  const setSelectedEmployees = (emps: Employee[]) => {
    setSelectedEmployeesState(emps);
  };

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
