import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Link } from "@tanstack/react-router";
import {
  CalendarRange,
  Plus,
  User,
  LayoutGrid,
  List,
  Zap
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format, addDays } from "date-fns";

import { EmployeeSearch } from "@/components/common/employee-search";
import { useToast } from "@/components/ui/toast";
import {
  fetchEmployees,
  fetchShifts,
  createShift,
  updateShift,
  fetchWorkspaceSchedules,
  fetchAssignmentHistory,
  upsertRoster,
  bulkUpsertRoster,
  deleteRoster,
  getCurrentWorkspaceId,
  fetchWorkingPatterns,
  assignPattern
} from "@/lib/api";
import {
  AssignPatternRequest,
  Shift,
  Employee
} from "@/lib/api/types";
import { ROUTES } from "@/lib/routes";

import { AssignmentHistory } from "../components/shifts/assignment-history";
import { CreateShiftModal } from "../components/shifts/create-shift-modal";
import { EditShiftModal } from "../components/shifts/edit-shift-modal";
import { PatternAssignmentForm } from "../components/shifts/pattern-assignment-form";
import { PriorityLegend } from "../components/shifts/priority-legend";
import { ShiftList } from "../components/shifts/shift-list";
import { ShiftPickerModal } from "../components/shifts/shift-picker-modal";
import { WeeklyRosterGrid } from "../components/shifts/weekly-roster-grid";



type TabType = "weekly-roster" | "assignments" | "directory";

export default function ShiftsPage() {
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

  // Queries
  const { data: employeesData } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId, 1, 3000),
    enabled: !!wsId,
  });
  const _employees = employeesData?.data ?? [];

  const { data: shifts } = useQuery({
    queryKey: ["shifts", wsId],
    queryFn: () => fetchShifts(wsId),
    enabled: !!wsId,
  });

  const { data: patterns } = useQuery({
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
  const { data: allSchedules } = useQuery({
    queryKey: ["schedules", wsId, format(startOfWeek, 'yyyy-MM-dd')],
    queryFn: () => fetchWorkspaceSchedules(
      wsId,
      format(weekDays[0], 'yyyy-MM-dd'),
      format(weekDays[6], 'yyyy-MM-dd')
    ),
    enabled: !!wsId,
  });

  const { data: history } = useQuery({
    queryKey: ["assignment_history", wsId],
    queryFn: () => fetchAssignmentHistory(wsId),
    enabled: !!wsId,
  });

  // Find schedule for selected employee
  const employeeSchedule = allSchedules?.find(s => s.employee_id === selectedEmployee?.id);

  // Mutations
  const assignPatternMutation = useMutation({
    mutationFn: (data: AssignPatternRequest) => assignPattern(wsId, selectedEmployee!.id, data),
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

  const _rosterMutation = useMutation({
    mutationFn: (data: { date: string; shift_id: string | null; notes?: string }) => upsertRoster(wsId, selectedEmployee!.id, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.update_roster_success"));
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.update_roster_error")); }
  });

  const bulkRosterMutation = useMutation({
    mutationFn: (items: { date: string; shift_id: string | null }[]) => bulkUpsertRoster(wsId, selectedEmployee!.id, items),
    onSuccess: () => {
      toast.success(t("shifts.messages.bulk_update_success"));
      setSelectedDays([]);
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.bulk_update_error")); }
  });

  const deleteRosterMutation = useMutation({
    mutationFn: (date: string) => deleteRoster(wsId, selectedEmployee!.id, date),
    onSuccess: () => {
      toast.success(t("shifts.messages.remove_override_success"));
      void queryClient.invalidateQueries({ queryKey: ["schedules"] });
      void queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: Error) => { toast.error(error.message, t("shifts.messages.remove_override_error")); }
  });

  const createShiftMutation = useMutation({
    mutationFn: (data: Partial<Shift>) => createShift(wsId, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.create_shift_success", { name: newShiftName }));
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

  const updateShiftMutation = useMutation({
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



  return (
    <div className="space-y-6">
      {/* Header section */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
          <h1 className="text-3xl font-bold text-slate-700 dark:text-slate-200 tracking-tight" data-testid="shifts-title">{t("shifts.title")}</h1>
          <p className="text-slate-500 dark:text-slate-400 mt-1">{t("shifts.subtitle")}</p>
        </div>
        <div className="flex items-center gap-3">
          <Link className="bg-white dark:bg-slate-900 hover:bg-slate-50 dark:hover:bg-slate-800 text-slate-700 dark:text-slate-200 font-bold px-4 py-2.5 rounded-2xl flex items-center gap-2 border border-slate-200 dark:border-slate-700 shadow-sm transition-all" search={{ tab: "patterns" } as never} to={ROUTES.SETTINGS.TIME}>
            <CalendarRange className="w-4 h-4" />
            <span className="text-xs uppercase tracking-widest">{t("patterns.manage_btn")}</span>
          </Link>
          <button className="bg-primary-600 hover:bg-primary-700 text-white font-bold px-6 py-2.5 rounded-2xl flex items-center gap-2 shadow-lg shadow-primary-100 transition-all" onClick={() => { setIsCreateModalOpen(true); }}>
            <Plus className="w-5 h-5" />
            <span className="text-xs uppercase tracking-widest">{t("shifts.create_btn")}</span>
          </button>
        </div>
      </div>

      {/* Tabs Navigation */}
      <div className="flex items-center gap-1 bg-slate-100 dark:bg-slate-700/50 p-1.5 rounded-[22px] w-fit border border-slate-200 dark:border-slate-700/50">
        <button className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all ${activeTab === 'weekly-roster' ? 'bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800' : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700'}`} onClick={() => { setActiveTab("weekly-roster"); }}>
          <LayoutGrid size={14} /> {t("shifts.tabs.weekly_roster")}
        </button>
        <button className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all ${activeTab === 'assignments' ? 'bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800' : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700'}`} onClick={() => { setActiveTab("assignments"); }}>
          <Zap size={14} /> {t("shifts.tabs.fixed_assignments", "Patterns & Rules")}
        </button>
        <button className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-bold uppercase tracking-widest transition-all ${activeTab === 'directory' ? 'bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-sm border border-slate-100 dark:border-slate-800' : 'text-slate-500 dark:text-slate-400 hover:text-slate-700 dark:hover:text-slate-200 dark:text-slate-700'}`} onClick={() => { setActiveTab("directory"); }}>
          <List size={14} /> {t("shifts.tabs.directory")}
        </button>
      </div>

      <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
        {activeTab !== "directory" ? (
          <div className="max-w-md mb-8">
            <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mb-2 block ml-1">{t("common.employees")}</label>
            <EmployeeSearch
              clearOnEscape
              data-testid="employee-select"
              keyboardShortcut={["/", "Enter"]}
              onChange={(id) => { if (!id) setSelectedEmployee(null); }}
              onSelectEmployee={(emp) => { setSelectedEmployee(emp); }}
              placeholder={t("shifts.empty_state.title")}
              value={selectedEmployee?.id ?? ""}
              workspaceId={wsId}
            />
          </div>
        ) : null}

        {!selectedEmployee && activeTab !== "directory" ? (
          <div className="bg-white dark:bg-slate-900 border border-slate-200 dark:border-slate-700 rounded-[40px] h-[500px] flex flex-col items-center justify-center text-center p-12 shadow-sm">
            <div className="bg-slate-50 dark:bg-slate-800 p-6 rounded-[32px] mb-6 animate-pulse text-slate-300 dark:text-slate-600"><User className="w-12 h-12" /></div>
            <h3 className="text-xl font-bold text-slate-700 dark:text-slate-200">{t("shifts.empty_state.title")}</h3>
            <p className="text-slate-500 dark:text-slate-400 mt-2 max-w-xs mx-auto text-sm">{t("shifts.empty_state.desc")}</p>
          </div>
        ) : activeTab === "weekly-roster" ? (
          /* TAB: WEEKLY ROSTER */
          <div className="space-y-6">
            <WeeklyRosterGrid
              currentDate={currentDate}
              employeeSchedule={employeeSchedule}
              onAssignAllWeek={() => {
                setTargetDates(weekDays.map(d => format(d, 'yyyy-MM-dd')));
                setIsShiftPickerOpen(true);
              }}
              onClearSelection={() => { setSelectedDays([]); }}
              onClearWeek={() => { handleAssignAllWeek(null); }}
              onCopyPreviousWeek={handleCopyPreviousWeek}
              onDeleteRoster={(date) => { deleteRosterMutation.mutate(date); }}
              onNavigateWeek={handleNavigateWeek}
              onOpenShiftPicker={(dates) => {
                setTargetDates(dates);
                setIsShiftPickerOpen(true);
              }}
              onToggleDaySelection={(dateString) => {
                if (selectedDays.includes(dateString)) {
                  setSelectedDays(selectedDays.filter(d => d !== dateString));
                } else {
                  setSelectedDays([...selectedDays, dateString]);
                }
              }}
              selectedDays={selectedDays}
              selectedEmployee={selectedEmployee!}
              weekDays={weekDays}
            />
            <PriorityLegend />
            <AssignmentHistory history={history} selectedEmployee={selectedEmployee} />
          </div>
        ) : activeTab === "assignments" ? (
          /* TAB: ASSIGNMENTS (Patterns & Rules) */
          <PatternAssignmentForm
            endDate={endDate}
            isForever={isForever}
            isLoading={assignPatternMutation.isPending}
            onAssign={handleAssign}
            patterns={patterns}
            selectedEmployee={selectedEmployee!}
            selectedPatternId={selectedPatternId}
            setEndDate={setEndDate}
            setIsForever={setIsForever}
            setSelectedPatternId={setSelectedPatternId}
            setStartDate={setStartDate}
            startDate={startDate}
          />
        ) : (
          /* TAB: DIRECTORY */
          <ShiftList onEdit={openEditModal} shifts={shifts} />
        )}
      </div>

      {/* Modals */}
      <CreateShiftModal
        isAutoDeduct={isAutoDeduct}
        isFixedBreak={isFixedBreak}
        isLoading={createShiftMutation.isPending}
        isOpen={isCreateModalOpen}
        newBreakEnd={newBreakEnd}
        newBreakStart={newBreakStart}
        newShiftBreak={newShiftBreak}
        newShiftEnd={newShiftEnd}
        newShiftName={newShiftName}
        newShiftStart={newShiftStart}
        newShiftSymbol={newShiftSymbol}
        onClose={() => { setIsCreateModalOpen(false); }}
        onCreateShift={handleCreateShift}
        setIsAutoDeduct={setIsAutoDeduct}
        setIsFixedBreak={setIsFixedBreak}
        setNewBreakEnd={setNewBreakEnd}
        setNewBreakStart={setNewBreakStart}
        setNewShiftBreak={setNewShiftBreak}
        setNewShiftEnd={setNewShiftEnd}
        setNewShiftName={setNewShiftName}
        setNewShiftStart={setNewShiftStart}
        setNewShiftSymbol={setNewShiftSymbol}
      />

      <ShiftPickerModal
        isOpen={isShiftPickerOpen}
        onClose={() => {
          setIsShiftPickerOpen(false);
          setTargetDates([]);
        }}
        onSelectShift={handleBulkAssign}
        shifts={shifts}
        targetDates={targetDates}
      />

      <EditShiftModal
        correctionEffectiveDate={correctionEffectiveDate}
        editingShift={editingShift}
        isAutoDeduct={isAutoDeduct}
        isCorrection={isCorrection}
        isFixedBreak={isFixedBreak}
        isLoading={updateShiftMutation.isPending}
        newBreakEnd={newBreakEnd}
        newBreakStart={newBreakStart}
        newShiftBreak={newShiftBreak}
        newShiftEnd={newShiftEnd}
        newShiftGrace={newShiftGrace}
        newShiftName={newShiftName}
        newShiftStart={newShiftStart}
        newShiftSymbol={newShiftSymbol}
        onClose={() => { setEditingShift(null); }}
        onUpdateShift={handleUpdateShift}
        setCorrectionEffectiveDate={setCorrectionEffectiveDate}
        setIsAutoDeduct={setIsAutoDeduct}
        setIsCorrection={setIsCorrection}
        setIsFixedBreak={setIsFixedBreak}
        setNewBreakEnd={setNewBreakEnd}
        setNewBreakStart={setNewBreakStart}
        setNewShiftBreak={setNewShiftBreak}
        setNewShiftEnd={setNewShiftEnd}
        setNewShiftGrace={setNewShiftGrace}
        setNewShiftName={setNewShiftName}
        setNewShiftStart={setNewShiftStart}
        setNewShiftSymbol={setNewShiftSymbol}
      />
    </div>
  );
}
