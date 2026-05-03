import { useMutation, useQuery, useQueryClient, UseMutationResult } from "@tanstack/react-query";
import { useState } from "react";
import { useTranslation } from "react-i18next";

import { addDays, format } from "date-fns";

import { useToast } from "@/components/ui/toast";
import {
  assignPattern,
  bulkUpsertRoster,
  createShift,
  deleteRoster,
  fetchAssignmentHistory,

  fetchShifts,
  fetchWorkingPatterns,
  fetchWorkspaceSchedules,
  getCurrentWorkspaceId,
  updateShift,
  upsertRoster,
} from "@/lib/api";
import {
  AssignPatternRequest,
  AssignmentHistory,
  Employee,
  EmployeeSchedule,
  EmployeeWorkingPattern,

  Shift,
  ShiftRoster,
  WorkingPattern,
} from "@/lib/api/types";

export type TabType = "weekly-roster" | "assignments" | "directory";

export interface UseShiftsReturn {
  activeTab: TabType;
  assignPatternMutation: UseMutationResult<EmployeeWorkingPattern, Error, AssignPatternRequest>;
  bulkRosterMutation: UseMutationResult<unknown, Error, { date: string; shift_id: string | null }[]>;
  correctionEffectiveDate: string;
  createShiftMutation: UseMutationResult<Shift, Error, Partial<Shift>>;
  currentDate: Date;
  deleteRosterMutation: UseMutationResult<unknown, Error, string>;
  editingShift: Shift | null;
  employeeSchedule: EmployeeSchedule | undefined;
  endDate: string;
  handleAssign: () => void;
  handleAssignAllWeek: (shiftId: string | null) => void;
  handleAssignBulk: (shiftId: string | null) => void;
  handleCopyPreviousWeek: () => Promise<unknown>;
  handleCreateShift: () => void;
  handleNavigateWeek: (direction: "prev" | "next") => void;
  handleUpdateShift: () => void;
  history: AssignmentHistory[] | undefined;
  isAutoDeduct: boolean;
  isCorrection: boolean;
  isCreateModalOpen: boolean;
  isFixedBreak: boolean;
  isForever: boolean;
  isShiftPickerOpen: boolean;
  newBreakEnd: string;
  newBreakStart: string;
  newShiftBreak: number;
  newShiftEnd: string;
  newShiftGrace: number;
  newShiftName: string;
  newShiftStart: string;
  newShiftSymbol: string;
  openEditModal: (shift: Shift) => void;
  patterns: WorkingPattern[] | undefined;
  selectedDays: string[];
  selectedEmployee: Employee | null;
  selectedPatternId: string;
  setActiveTab: (v: TabType) => void;
  setCorrectionEffectiveDate: (v: string) => void;
  setCurrentDate: (v: Date) => void;
  setEditingShift: (v: Shift | null) => void;
  setEndDate: (v: string) => void;
  setIsAutoDeduct: (v: boolean) => void;
  setIsCorrection: (v: boolean) => void;
  setIsCreateModalOpen: (v: boolean) => void;
  setIsFixedBreak: (v: boolean) => void;
  setIsForever: (v: boolean) => void;
  setIsShiftPickerOpen: (v: boolean) => void;
  setNewBreakEnd: (v: string) => void;
  setNewBreakStart: (v: string) => void;
  setNewShiftBreak: (v: number) => void;
  setNewShiftEnd: (v: string) => void;
  setNewShiftGrace: (v: number) => void;
  setNewShiftName: (v: string) => void;
  setNewShiftStart: (v: string) => void;
  setNewShiftSymbol: (v: string) => void;
  setSelectedDays: (v: string[]) => void;
  setSelectedEmployee: (v: Employee | null) => void;
  setSelectedPatternId: (v: string) => void;
  setStartDate: (v: string) => void;
  setTargetDates: (v: string[]) => void;
  shifts: Shift[] | undefined;
  startDate: string;
  targetDates: string[];
  updateShiftMutation: UseMutationResult<Shift, Error, { id: string; data: Partial<Shift> }>;
  weekDays: Date[];
  wsId: string;
}

export function useShifts(): UseShiftsReturn {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
  const queryClient = useQueryClient();
  const toast = useToast();
  
  const [activeTab, setActiveTab] = useState<TabType>("weekly-roster");

  // Selection states
  const [selectedEmployee, setSelectedEmployee] = useState<Employee | null>(null);
  const [selectedDays, setSelectedDays] = useState<string[]>([]);

  // Shift Picker Modal State
  const [isShiftPickerOpen, setIsShiftPickerOpen] = useState(false);
  const [targetDates, setTargetDates] = useState<string[]>([]);

  // Weekly Roster Navigation
  const [currentDate, setCurrentDate] = useState(new Date());

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
  const [isFixedBreak, setIsFixedBreak] = useState(false);
  const [isAutoDeduct, setIsAutoDeduct] = useState(true);

  // Edit Shift State
  const [editingShift, setEditingShift] = useState<Shift | null>(null);
  const [isCorrection, setIsCorrection] = useState(false);
  const [correctionEffectiveDate, setCorrectionEffectiveDate] = useState(format(new Date(), 'yyyy-MM-dd'));



  const { data: shifts } = useQuery<Shift[]>({
    queryKey: ["shifts", wsId],
    queryFn: () => fetchShifts(wsId),
    enabled: !!wsId,
  });

  const { data: patterns } = useQuery<WorkingPattern[]>({
    queryKey: ["working-patterns", wsId],
    queryFn: () => fetchWorkingPatterns(wsId),
    enabled: !!wsId,
  });

  // Weekly Dates Logic
  const startOfWeek = new Date(currentDate);
  startOfWeek.setDate(currentDate.getDate() - currentDate.getDay() + 1); // Monday
  const weekDays = Array.from({ length: 7 }, (_, index) => {
    const d = new Date(startOfWeek);
    d.setDate(startOfWeek.getDate() + index);
    return d;
  });

  // Fetch Effective Schedules (Bulk)
  const { data: allSchedules } = useQuery<EmployeeSchedule[]>({
    queryKey: ["schedules", wsId, format(startOfWeek, 'yyyy-MM-dd')],
    queryFn: () => fetchWorkspaceSchedules(
      wsId,
      format(weekDays[0], 'yyyy-MM-dd'),
      format(weekDays[6], 'yyyy-MM-dd')
    ),
    enabled: !!wsId,
  });

  const { data: history } = useQuery<AssignmentHistory[]>({
    queryKey: ["assignment_history", wsId],
    queryFn: () => fetchAssignmentHistory(wsId),
    enabled: !!wsId,
  });

  // Find schedule for selected employee
  const employeeSchedule = allSchedules?.find(s => s.employee_id === selectedEmployee?.id);

  // Mutations
  const assignPatternMutation = useMutation<EmployeeWorkingPattern, Error, AssignPatternRequest>({
    mutationFn: (data: AssignPatternRequest) => {
      if (!selectedEmployee) return Promise.reject(new Error("No employee selected"));
      return assignPattern(wsId, selectedEmployee.id, data);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.assign_success"));
      setSelectedPatternId("");
      setEndDate("");
      setIsForever(true);
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.assign_error")); }
  });

  const _rosterMutation = useMutation<ShiftRoster, Error, { date: string; shift_id: string | null; notes?: string }>({
    mutationFn: (data: { date: string; shift_id: string | null; notes?: string }) => {
      if (!selectedEmployee) return Promise.reject(new Error("No employee selected"));
      return upsertRoster(wsId, selectedEmployee.id, data);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.update_roster_success"));
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.update_roster_error")); }
  });

  const bulkRosterMutation = useMutation<unknown, Error, { date: string; shift_id: string | null }[]>({
    mutationFn: (items: { date: string; shift_id: string | null }[]) => {
      if (!selectedEmployee) return Promise.reject(new Error("No employee selected"));
      return bulkUpsertRoster(wsId, selectedEmployee.id, items);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.bulk_update_success"));
      setSelectedDays([]);
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.bulk_update_error")); }
  });

  const deleteRosterMutation = useMutation<unknown, Error, string>({
    mutationFn: (date: string) => {
      if (!selectedEmployee) return Promise.reject(new Error("No employee selected"));
      return deleteRoster(wsId, selectedEmployee.id, date);
    },
    onSuccess: () => {
      toast.success(t("shifts.messages.remove_override_success"));
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.remove_override_error")); }
  });

  const createShiftMutation = useMutation<Shift, Error, Partial<Shift>>({
    mutationFn: (data: Partial<Shift>) => createShift(wsId, data),
    onSuccess: (data) => {
      toast.success(t("shifts.messages.create_shift_success", { name: data.name }));
      setNewShiftName("");
      setNewShiftSymbol("");
      setNewShiftStart("08:00");
      setNewShiftEnd("17:00");
      setNewShiftBreak(60);
      setNewShiftGrace(15);
      setNewBreakStart("");
      setNewBreakEnd("");
      setIsFixedBreak(false);
      setIsCreateModalOpen(false);
      setActiveTab("directory");
      void queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.create_shift_error")); }
  });

  const updateShiftMutation = useMutation<Shift, Error, { id: string, data: Partial<Shift> }>({
    mutationFn: ({ id, data }: { id: string, data: Partial<Shift> }) => updateShift(wsId, id, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.update_shift_success"));
      setEditingShift(null);
      void queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.update_shift_error")); }
  });

  // Handlers
  const handleAssign = () => {
    if (!selectedEmployee || !selectedPatternId) {
      toast.warning(t("shifts.messages.validation_missing_fields"));
      return;
    }
    assignPatternMutation.mutate({
      pattern_id: selectedPatternId,
      start_date: startDate,
      end_date: isForever ? null : endDate,
    });
  };

  const handleBulkAssign = (shiftId: string | null) => {
    if (!selectedEmployee || selectedDays.length === 0) return;
    const items = selectedDays.map(date => ({ date, shift_id: shiftId }));
    bulkRosterMutation.mutate(items);
  };

  const handleAssignAllWeek = (shiftId: string | null) => {
    if (!selectedEmployee) return;
    const items = weekDays.map(day => ({ date: format(day, 'yyyy-MM-dd'), shift_id: shiftId }));
    bulkRosterMutation.mutate(items);
  };

  const handleCopyPreviousWeek = async () => {
    if (!selectedEmployee || !allSchedules) return;
    const previousWeekStart = addDays(startOfWeek, -7);
    const previousWeekEnd = addDays(startOfWeek, -1);
    try {
      const previousSchedules = await fetchWorkspaceSchedules(wsId, format(previousWeekStart, 'yyyy-MM-dd'), format(previousWeekEnd, 'yyyy-MM-dd'));
      const empPrevious = previousSchedules.find(s => s.employee_id === selectedEmployee.id);
      if (!empPrevious) { toast.warning(t("shifts.messages.no_previous_data")); return; }
      const items = empPrevious.schedules.map((s, index) => ({ date: format(weekDays[index], 'yyyy-MM-dd'), shift_id: s.shift_id }));
      bulkRosterMutation.mutate(items);
    } catch (e: unknown) { toast.error(e instanceof Error ? e.message : String(e)); }
  };

  const handleCreateShift = () => {
    if (!newShiftName || !newShiftStart || !newShiftEnd) return;
    createShiftMutation.mutate({
      name: newShiftName,
      report_symbol: newShiftSymbol || undefined,
      start_time: newShiftStart + ":00",
      end_time: newShiftEnd + ":00",
      break_duration_minutes: newShiftBreak,
      early_punch_in_grace_period_minutes: newShiftGrace,
      is_break_overtime: false,
      ...(isFixedBreak && newBreakStart && newBreakEnd ? {
        break_start: newBreakStart + ":00",
        break_end: newBreakEnd + ":00"
      } : {}),
      is_break_auto_deduct: isAutoDeduct
    });
  };

  const handleUpdateShift = () => {
    if (!editingShift) return;
    updateShiftMutation.mutate({
      id: editingShift.id,
      data: {
        name: newShiftName,
        report_symbol: newShiftSymbol || null,
        start_time: newShiftStart,
        end_time: newShiftEnd,
        break_duration_minutes: newShiftBreak,
        early_punch_in_grace_period_minutes: newShiftGrace,
        is_break_overtime: false,
        ...(isFixedBreak && newBreakStart && newBreakEnd ? {
          break_start: newBreakStart,
          break_end: newBreakEnd
        } : {
          break_start: null,
          break_end: null
        }),
        is_break_auto_deduct: isAutoDeduct,
        is_correction: isCorrection,
        effective_date: isCorrection ? undefined : correctionEffectiveDate
      }
    });
  };

  const openEditModal = (shift: Shift) => {
    setEditingShift(shift);
    setNewShiftName(shift.name);
    setNewShiftSymbol(shift.report_symbol ?? "");
    setNewShiftStart(shift.start_time.substring(0, 5));
    setNewShiftEnd(shift.end_time.substring(0, 5));
    setNewShiftBreak(shift.break_duration_minutes);
    setNewShiftGrace(shift.early_punch_in_grace_period_minutes ?? 15);
    setNewBreakStart(shift.break_start?.substring(0, 5) ?? "");
    setNewBreakEnd(shift.break_end?.substring(0, 5) ?? "");
    setIsFixedBreak(!!shift.break_start);
    setIsAutoDeduct(shift.is_break_auto_deduct ?? true);
    setIsCorrection(false);
    setCorrectionEffectiveDate(format(new Date(), 'yyyy-MM-dd'));
  };

  const handleNavigateWeek = (direction: "prev" | "next") => {
    const d = new Date(currentDate);
    d.setDate(d.getDate() + (direction === "prev" ? -7 : 7));
    setCurrentDate(d);
    setSelectedDays([]);
  };

  return {
    activeTab,
    assignPatternMutation,
    bulkRosterMutation,
    correctionEffectiveDate,
    createShiftMutation,
    currentDate,
    deleteRosterMutation,
    editingShift,
    employeeSchedule,
    endDate,
    handleAssign,
    handleAssignAllWeek,
    handleAssignBulk: handleBulkAssign,
    handleCopyPreviousWeek,
    handleCreateShift,
    handleNavigateWeek,
    handleUpdateShift,
    history,
    isAutoDeduct,
    isCorrection,
    isCreateModalOpen,
    isFixedBreak,
    isForever,
    isShiftPickerOpen,
    newBreakEnd,
    newBreakStart,
    newShiftBreak,
    newShiftEnd,
    newShiftGrace,
    newShiftName,
    newShiftStart,
    newShiftSymbol,
    openEditModal,
    patterns,
    selectedDays,
    selectedEmployee,
    selectedPatternId,
    setActiveTab,
    setCorrectionEffectiveDate,
    setCurrentDate,
    setEditingShift,
    setEndDate,
    setIsAutoDeduct,
    setIsCorrection,
    setIsCreateModalOpen,
    setIsFixedBreak,
    setIsForever,
    setIsShiftPickerOpen,
    setNewBreakEnd,
    setNewBreakStart,
    setNewShiftBreak,
    setNewShiftEnd,
    setNewShiftGrace,
    setNewShiftName,
    setNewShiftStart,
    setNewShiftSymbol,
    setSelectedDays,
    setSelectedEmployee,
    setSelectedPatternId,
    setStartDate,
    setTargetDates,
    shifts,
    startDate,
    targetDates,
    updateShiftMutation,
    weekDays,
    wsId,
  };
}
