import { useQuery } from "@tanstack/react-query";
import { useSearch, useNavigate } from "@tanstack/react-router";
import { useState } from "react";

import { format } from "date-fns";

import type { Shift, Employee } from "@/lib/api/types";

import { useDerivedStateFrom } from "@/hooks/use-derived-state-from";
import { fetchEmployee } from "@/lib/api";
import { createQueryKey } from "@/lib/query-keys";

import type { ShiftsSearch } from "../search-schema";
import type { TabType, ViewMode } from "./use-shifts-types";


export type ScheduleCoverageFilter = "all" | "missing" | "complete";

export function useShiftFormState() {
  // useSearch({ strict: false }) returns a permissive type; we cast to
  // ShiftsSearch to match the route's validateSearch contract (codebase idiom
  // — see shifts-rules-page.tsx).
  const rawSearch = useSearch({ strict: false });
  const search = rawSearch as ShiftsSearch;
  const navigate = useNavigate();

  // Derive single employeeId from comma-separated employeeIds (schema field).
  const employeeId = search.employeeIds ? search.employeeIds.split(",")[0] : undefined;

  // Runtime guards: the type says tab/view are always valid (Zod .catch()
  // in the schema), but at runtime — especially in tests — useSearch may
  // return {} before Zod validation applies. Widen to string so TS doesn't
  // flag the runtime guard as unnecessary.
  const rawTab: string | undefined = search.tab;
  const tab: TabType = rawTab === "weekly-roster" || rawTab === "assignments" || rawTab === "directory" ? rawTab : "weekly-roster";
  const initialTab = tab;
  const [activeTab, setActiveTabState] = useState<TabType>(initialTab);

  const rawView: string | undefined = search.view;
  const initialView: ViewMode = rawView === "week" || rawView === "month" ? rawView : "week";
  const [viewMode, setViewModeState] = useState<ViewMode>(initialView);

  // Selection states
  const initialIds = search.employeeIds ? search.employeeIds.split(",") : [];
  const [selectedEmployeeIds, setSelectedEmployeeIdsState] = useState<string[]>(initialIds);
  const [coverageFilter, setCoverageFilter] = useState<ScheduleCoverageFilter>("all");
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

  // Load initial selected employee from URL query parameter if present on mount.
  // Do not gate on selectedEmployee — that would create a circular TDZ with the
  // derived-state declaration below. The query only runs when URL has employeeIds.
  const { data: initialEmployee } = useQuery({
    queryKey: createQueryKey("employee", employeeId),
    queryFn: () => {
      if (!employeeId) throw new Error("Missing params");
      return fetchEmployee(employeeId);
    },
    enabled: !!employeeId,
  });

  // Sync initial employee from React Query result — useDerivedStateFrom
  // handles this without useEffect. Reference stabilizes via the query cache.
  const [selectedEmployee, setSelectedEmployeeState] = useDerivedStateFrom(
    initialEmployee ?? null,
  );

  // Custom setters that update URL query params AND cross-sync overlapping state.
  // Cross-sync happens at the point of state change, not reactively in useEffect.
  const setActiveTab = (val: TabType) => {
    setActiveTabState(val);
    void navigate({
      to: "/shifts",
      search: (prev: unknown) => ({ ...(prev as ShiftsSearch), tab: val }),
    });
  };

  const setViewMode = (val: ViewMode) => {
    setViewModeState(val);
    void navigate({
      to: "/shifts",
      search: (prev: unknown) => ({ ...(prev as ShiftsSearch), view: val }),
    });
  };

  // Week View: selectedEmployee is the source of truth → sync IDs
  // Month View: set selectedEmployee to null so IDs become the source
  const setSelectedEmployee = (emp: Employee | null) => {
    setSelectedEmployeeState(emp);
    if (viewMode === "week") {
      setSelectedEmployeeIdsState(emp ? [emp.id] : []);
      setSelectedEmployeesState(emp ? [emp] : []);
    }
    void navigate({
      to: "/shifts",
      search: (prev: unknown) => ({ ...(prev as ShiftsSearch), employeeIds: emp?.id }),
    });
  };

  // Month View: selectedEmployeeIds is the source of truth → sync employee
  const setSelectedEmployeeIds = (ids: string[]) => {
    setSelectedEmployeeIdsState(ids);
    if (viewMode === "month" && ids.length > 0) {
      const match = selectedEmployees.find((e: Employee) => e.id === ids[0]);
      if (match) setSelectedEmployeeState(match);
    } else if (viewMode === "month" && ids.length === 0) {
      setSelectedEmployeeState(null);
    }
    void navigate({
      to: "/shifts",
      search: (prev: unknown) => ({ ...(prev as ShiftsSearch), employeeIds: ids.length > 0 ? ids.join(",") : undefined }),
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
    selectedEmployees, setSelectedEmployees,
    coverageFilter, setCoverageFilter
  };
}
