import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  CalendarRange, 
  Plus, 
  Clock, 
  Calendar, 
  User, 
  Trash2, 
  Save,
  ChevronRight,
  ChevronLeft,
  Info,
  X,
  LayoutGrid,
  List,
  History,
  ShieldCheck,
  Zap,
  Building2
} from "lucide-react";
import { 
  fetchEmployees, 
  fetchShifts, 
  assignShift, 
  createShift,
  updateShift,
  fetchWorkspace,
  fetchWorkspaceSchedules,
  fetchAssignmentHistory,
  upsertRoster,
  bulkUpsertRoster,
  deleteRoster,
  getCurrentWorkspaceId,
  fetchWorkingPatterns,
  assignPattern,
  Employee,
  Shift
} from "../lib/api";
import { useToast } from "../components/Toast";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { format, addDays } from "date-fns";
import { vi, enUS, zhCN } from "date-fns/locale";
import { cn } from "@/lib/utils";

const dateLocales: Record<string, any> = {
  vi: vi,
  en: enUS,
  zh: zhCN,
};

import { Combobox } from "@/components/ui/Combobox";
import { MousePointer2, Copy, Trash, Calendar as CalendarIcon } from "lucide-react";
import Modal from "@/components/ui/Modal";
import { ShiftList } from "../components/shifts/ShiftList";
import { AssignmentHistory } from "../components/shifts/AssignmentHistory";
import { ScheduleTracePopover } from "../components/shifts/ScheduleTracePopover";
import { PriorityLegend } from "../components/shifts/PriorityLegend";

import { Link } from "react-router-dom";

type TabType = "weekly-roster" | "assignments" | "directory";

export default function ShiftsPage() {
  const { t, i18n } = useTranslation();
  const currentLocale = dateLocales[i18n.language.substring(0, 2)] || enUS;
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatDate, formatTime, formatDateTime } = useDateTimeFormatter();
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
  const employees = employeesData?.data || [];

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
  const weekDays = Array.from({ length: 7 }, (_, i) => {
    const d = new Date(startOfWeek);
    d.setDate(startOfWeek.getDate() + i);
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
    mutationFn: (data: any) => assignPattern(wsId, selectedEmployee!.id, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.assign_success"));
      setSelectedPatternId("");
      setEndDate("");
      setIsForever(true);
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: any) => toast.error(error.message, t("shifts.messages.assign_error"))
  });

  const rosterMutation = useMutation({
    mutationFn: (data: any) => upsertRoster(wsId, selectedEmployee!.id, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.update_roster_success"));
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: any) => toast.error(error.message, t("shifts.messages.update_roster_error"))
  });

  const bulkRosterMutation = useMutation({
    mutationFn: (items: any[]) => bulkUpsertRoster(wsId, selectedEmployee!.id, items),
    onSuccess: () => {
      toast.success(t("shifts.messages.bulk_update_success"));
      setSelectedDays([]);
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: any) => toast.error(error.message, t("shifts.messages.bulk_update_error"))
  });

  const deleteRosterMutation = useMutation({
    mutationFn: (date: string) => deleteRoster(wsId, selectedEmployee!.id, date),
    onSuccess: () => {
      toast.success(t("shifts.messages.remove_override_success"));
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: any) => toast.error(error.message, t("shifts.messages.remove_override_error"))
  });

  const createShiftMutation = useMutation({
    mutationFn: (data: any) => createShift(wsId, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.create_shift_success", { name: newShiftName }));
      setNewShiftName("");
      setNewShiftStart("08:00");
      setNewShiftEnd("17:00");
      setNewShiftBreak(60);
      setNewShiftGrace(15);
      setNewBreakStart("");
      setNewBreakEnd("");
      setIsFixedBreak(false);
      setIsCreateModalOpen(false);
      setActiveTab("directory");
      queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
    },
    onError: (error: any) => toast.error(error.message, t("shifts.messages.create_shift_error"))
  });

  const updateShiftMutation = useMutation({
    mutationFn: ({ id, data }: { id: string, data: any }) => updateShift(wsId, id, data),
    onSuccess: () => {
      toast.success(t("shifts.messages.update_shift_success"));
      setEditingShift(null);
      queryClient.invalidateQueries({ queryKey: ["shifts", wsId] });
      queryClient.invalidateQueries({ queryKey: ["schedules"] });
      queryClient.invalidateQueries({ queryKey: ["assignment_history"] });
    },
    onError: (error: any) => toast.error(error.message, t("shifts.messages.update_shift_error"))
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
    const prevWeekStart = addDays(startOfWeek, -7);
    const prevWeekEnd = addDays(startOfWeek, -1);
    try {
      const prevSchedules = await fetchWorkspaceSchedules(wsId, format(prevWeekStart, 'yyyy-MM-dd'), format(prevWeekEnd, 'yyyy-MM-dd'));
      const empPrev = prevSchedules.find(s => s.employee_id === selectedEmployee.id);
      if (!empPrev) { toast.warning(t("shifts.messages.no_previous_data")); return; }
      const items = empPrev.schedules.map((s, idx) => ({ date: format(weekDays[idx], 'yyyy-MM-dd'), shift_id: s.shift_id }));
      bulkRosterMutation.mutate(items);
    } catch (e: any) { toast.error(e.message); }
  };

  const handleCreateShift = () => {
    if (!newShiftName || !newShiftStart || !newShiftEnd) return;
    createShiftMutation.mutate({
      name: newShiftName, start_time: newShiftStart + ":00", end_time: newShiftEnd + ":00",
      break_duration_minutes: newShiftBreak,
      early_punch_in_grace_period_minutes: newShiftGrace,
      break_is_overtime: false,
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
        name: newShiftName, start_time: newShiftStart, end_time: newShiftEnd,
        break_duration_minutes: newShiftBreak,
        early_punch_in_grace_period_minutes: newShiftGrace,
        break_is_overtime: false,
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
    setNewShiftStart(shift.start_time.substring(0, 5));
    setNewShiftEnd(shift.end_time.substring(0, 5));
    setNewShiftBreak(shift.break_duration_minutes);
    setNewShiftGrace(shift.early_punch_in_grace_period_minutes ?? 15);
    setNewBreakStart(shift.break_start?.substring(0, 5) || "");
    setNewBreakEnd(shift.break_end?.substring(0, 5) || "");
    setIsFixedBreak(!!shift.break_start);
    setIsAutoDeduct(shift.is_break_auto_deduct ?? true);
    setIsCorrection(false);
    setCorrectionEffectiveDate(format(new Date(), 'yyyy-MM-dd'));
  };

  const employeeOptions = employees.map(emp => ({
    value: emp.id, label: emp.full_name, description: emp.employee_id_number,
    icon: <div className="w-6 h-6 rounded-lg bg-slate-100 flex items-center justify-center text-[10px] font-bold text-slate-500">{emp.full_name[0]}</div>
  })) || [];

  return (
    <div className="space-y-6">
      {/* Header section */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
          <h1 data-testid="shifts-title" className="text-3xl font-bold text-slate-900 tracking-tight">{t("shifts.title")}</h1>
          <p className="text-slate-500 mt-1">{t("shifts.subtitle")}</p>
        </div>
        <div className="flex items-center gap-3">
          <Link to="/working-patterns" className="bg-white hover:bg-slate-50 text-slate-700 font-bold px-4 py-2.5 rounded-2xl flex items-center gap-2 border border-slate-200 shadow-sm transition-all">
            <CalendarRange className="w-4 h-4" />
            <span className="text-xs uppercase tracking-widest">{t("patterns.manage_btn")}</span>
          </Link>
          <button onClick={() => setIsCreateModalOpen(true)} className="bg-primary-600 hover:bg-primary-700 text-white font-black px-6 py-2.5 rounded-2xl flex items-center gap-2 shadow-lg shadow-primary-100 transition-all">
            <Plus className="w-5 h-5" />
            <span className="text-xs uppercase tracking-widest">{t("shifts.create_btn")}</span>
          </button>
        </div>
      </div>

      {/* Tabs Navigation */}
      <div className="flex items-center gap-1 bg-slate-100/50 p-1.5 rounded-[22px] w-fit border border-slate-200/50">
        <button onClick={() => setActiveTab("weekly-roster")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'weekly-roster' ? 'bg-white text-primary-600 shadow-sm border border-slate-100' : 'text-slate-500 hover:text-slate-700'}`}>
          <LayoutGrid size={14} /> {t("shifts.tabs.weekly_roster")}
        </button>
        <button onClick={() => setActiveTab("assignments")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'assignments' ? 'bg-white text-primary-600 shadow-sm border border-slate-100' : 'text-slate-500 hover:text-slate-700'}`}>
          <Zap size={14} /> {t("shifts.tabs.fixed_assignments", "Patterns & Rules")}
        </button>
        <button onClick={() => setActiveTab("directory")} className={`flex items-center gap-2 px-6 py-2.5 rounded-[18px] text-xs font-black uppercase tracking-widest transition-all ${activeTab === 'directory' ? 'bg-white text-primary-600 shadow-sm border border-slate-100' : 'text-slate-500 hover:text-slate-700'}`}>
          <List size={14} /> {t("shifts.tabs.directory")}
        </button>
      </div>

      <div className="animate-in fade-in slide-in-from-bottom-4 duration-500">
        {activeTab !== "directory" && (
          <div className="max-w-md mb-8">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest mb-2 block ml-1">{t("common.employees")}</label>
            <Combobox data-testid="employee-select" options={employeeOptions} value={selectedEmployee?.id} onChange={(id) => setSelectedEmployee(employees.find(e => e.id === id) || null)} placeholder={t("shifts.empty_state.title")} searchPlaceholder={t("common.search_employee")} className="bg-white border-slate-200 shadow-sm h-12 rounded-[20px]" />
          </div>
        )}

        {!selectedEmployee && activeTab !== "directory" ? (
          <div className="bg-white border border-slate-200 rounded-[40px] h-[500px] flex flex-col items-center justify-center text-center p-12 shadow-sm">
            <div className="bg-slate-50 p-6 rounded-[32px] mb-6 animate-pulse text-slate-300"><User className="w-12 h-12" /></div>
            <h3 className="text-xl font-bold text-slate-900">{t("shifts.empty_state.title")}</h3>
            <p className="text-slate-500 mt-2 max-w-xs mx-auto text-sm">{t("shifts.empty_state.desc")}</p>
          </div>
        ) : activeTab === "weekly-roster" ? (
          /* TAB: WEEKLY ROSTER */
          <div className="space-y-6">
            <div className="flex items-center justify-between bg-white p-4 rounded-[32px] border border-slate-200 shadow-sm">
              <div className="flex items-center gap-4">
                <div className="w-12 h-12 bg-primary-600 rounded-2xl flex items-center justify-center text-white text-lg font-black shadow-lg shadow-primary-100">{selectedEmployee!.full_name[0]}</div>
                <div>
                  <h2 className="text-lg font-bold text-slate-900 leading-tight">{selectedEmployee!.full_name}</h2>
                  <p className="text-[10px] font-black uppercase tracking-widest text-slate-400">{t("shifts.current_week_roster")}</p>
                </div>
              </div>
              
              <div className="flex items-center gap-3">
                <div className="w-56">
                  <Combobox options={[{ value: "copy-prev", label: t("shifts.actions.copy_previous"), icon: <Copy className="w-4 h-4" /> }, { value: "assign-all", label: t("shifts.actions.assign_all_week"), icon: <CalendarIcon className="w-4 h-4" /> }, { value: "clear-week", label: t("shifts.actions.clear_week"), icon: <Trash className="w-4 h-4" /> }]} onChange={(val) => { if (val === "copy-prev") handleCopyPreviousWeek(); else if (val === "clear-week") handleAssignAllWeek(null); else if (val === "assign-all") { setTargetDates(weekDays.map(d => format(d, 'yyyy-MM-dd'))); setIsShiftPickerOpen(true); } }} placeholder={t("common.quick_actions")} className="h-10 bg-slate-100 border-slate-200 text-[10px] py-2 px-3 rounded-xl uppercase tracking-tighter" />
                </div>
                <button onClick={() => { const d = new Date(currentDate); d.setDate(d.getDate() - 7); setCurrentDate(d); setSelectedDays([]); }} className="p-2.5 hover:bg-slate-100 border border-slate-100 rounded-xl transition-all"><ChevronLeft size={18} /></button>
                <div className="px-4 py-2 bg-slate-50 border border-slate-200 rounded-xl text-xs font-bold text-slate-600 uppercase tracking-tighter">{formatDate(weekDays[0])} - {formatDate(weekDays[6])}</div>
                <button onClick={() => { const d = new Date(currentDate); d.setDate(d.getDate() + 7); setCurrentDate(d); setSelectedDays([]); }} className="p-2.5 hover:bg-slate-100 border border-slate-100 rounded-xl transition-all"><ChevronRight size={18} /></button>
              </div>
            </div>

            {selectedDays.length > 0 && (
              <div className="bg-primary-600 text-white p-4 rounded-[24px] flex items-center justify-between shadow-xl shadow-primary-100 animate-in slide-in-from-top-4 duration-300">
                <div className="flex items-center gap-4">
                  <span className="text-xs font-black uppercase tracking-widest">{t("shifts.days_selected", { count: selectedDays.length })}</span>
                  <div className="h-4 w-px bg-white/20" />
                  <button onClick={() => { setTargetDates(selectedDays); setIsShiftPickerOpen(true); }} className="bg-white/10 border border-white/20 rounded-xl px-4 py-2 text-[10px] font-black uppercase tracking-widest hover:bg-white/20 transition-all flex items-center gap-2"><Plus size={14} /> {t("shifts.assign_selected_btn")}</button>
                </div>
                <button onClick={() => setSelectedDays([])} className="hover:bg-white/10 p-2 rounded-xl transition-all"><X size={18} /></button>
              </div>
            )}

            <PriorityLegend />

            <div className="grid grid-cols-1 md:grid-cols-7 gap-4">
              {weekDays.map((day, idx) => {
                const dateStr = format(day, 'yyyy-MM-dd');
                const isSelected = selectedDays.includes(dateStr);
                const effectiveSchedule = employeeSchedule?.schedules.find(s => s.date === dateStr);
                const isToday = day.toDateString() === new Date().toDateString();
                const source = effectiveSchedule?.source as any;
                return (
                  <div key={idx} onClick={() => isSelected ? setSelectedDays(selectedDays.filter(d => d !== dateStr)) : setSelectedDays([...selectedDays, dateStr])} className={`bg-white border p-4 rounded-[32px] flex flex-col min-h-[350px] transition-all relative group cursor-pointer ${isSelected ? 'border-primary-500 ring-4 ring-primary-50 shadow-md' : isToday ? 'border-primary-300 ring-4 ring-primary-50 shadow-lg' : 'border-slate-100 hover:border-slate-200 shadow-sm'}`}>
                    <div className="text-center mb-5 pb-5 border-b border-slate-50 relative">
                      {isSelected && <div className="absolute -top-1 -right-1 bg-primary-600 text-white rounded-full p-1 shadow-sm"><ShieldCheck size={12} /></div>}
                      <div className={`text-[10px] font-black uppercase tracking-tighter ${isToday ? 'text-primary-600' : 'text-slate-400'}`}>{format(day, "EEEE", { locale: currentLocale })}</div>
                      <div className={`text-2xl font-black mt-1 ${isToday ? 'text-primary-600' : 'text-slate-900'}`}>{day.getDate()}</div>
                    </div>
                    <div className="flex-1 space-y-4">
                      <div className="flex justify-center items-center gap-1">
                        {source === "roster" && <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-purple-50 text-purple-600 border border-purple-100 uppercase tracking-tighter flex items-center gap-1"><Zap size={8} /> {t("shifts.types.flexible")}</span>}
                        {source === "pattern" && <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-blue-50 text-blue-600 border border-blue-100 uppercase tracking-tighter flex items-center gap-1"><ShieldCheck size={8} /> {t("patterns.title")}</span>}
                        {source === "department_pattern" && <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-slate-50 text-slate-600 border border-slate-200 uppercase tracking-tighter flex items-center gap-1"><Building2 size={8} /> {t("patterns.title")}</span>}
                        {source === "fixed" && <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-emerald-50 text-emerald-600 border border-emerald-100 uppercase tracking-tighter flex items-center gap-1"><ShieldCheck size={8} /> {t("shifts.types.fixed")}</span>}
                        {source && (source === "department" || source === "department_fixed") && <span className="text-[9px] font-black px-2 py-0.5 rounded-full bg-slate-50 text-slate-500 border border-slate-100 uppercase tracking-tighter flex items-center gap-1"><Building2 size={8} /> {t("common.department")}</span>}
                        {effectiveSchedule && <ScheduleTracePopover traces={effectiveSchedule.traces || []} source={effectiveSchedule.source} />}
                      </div>
                      {effectiveSchedule?.shift_id ? (
                        <div className={`p-4 rounded-[24px] relative group/item animate-in zoom-in-95 border ${source === 'roster' ? 'bg-purple-50/50 border-purple-100' : 'bg-primary-50 border-blue-100'}`}>
                          <div className={`font-black text-xs mb-1 uppercase tracking-tight leading-tight ${source === 'roster' ? 'text-purple-700' : 'text-blue-700'}`}>{effectiveSchedule.shift_name}</div>
                          <div className={`flex items-center gap-1.5 text-[10px] font-bold ${source === 'roster' ? 'text-purple-500' : 'text-blue-500'}`}>{formatTime(effectiveSchedule.start_time)} - {formatTime(effectiveSchedule.end_time)}</div>
                          {effectiveSchedule.break_duration_minutes !== null && <div className="mt-2 pt-2 border-t border-black/5 flex items-center gap-1.5 text-[9px] font-black uppercase tracking-tighter text-slate-400"><Clock size={8} /> {effectiveSchedule.break_start ? <span>{t("common.break")}: {effectiveSchedule.break_start.substring(0, 5)} - {effectiveSchedule.break_end?.substring(0, 5)}</span> : <span>{t("common.break")}: {effectiveSchedule.break_duration_minutes}m</span>}</div>}
                          {source === 'roster' && <button onClick={(e) => { e.stopPropagation(); deleteRosterMutation.mutate(dateStr); }} className="absolute -top-2 -right-2 bg-white border border-slate-200 rounded-full p-1.5 shadow-md opacity-0 group-hover/item:opacity-100 transition-opacity hover:bg-rose-50 hover:border-rose-200 group/btn"><X size={12} className="text-slate-400 group-hover/btn:text-rose-500" /></button>}
                        </div>
                      ) : <div className="bg-slate-50/50 border border-slate-100 border-dashed p-6 rounded-[24px] text-[10px] text-slate-400 font-black text-center uppercase tracking-widest flex flex-col items-center justify-center gap-2">{t("shifts.off_day")}</div>}
                    </div>
                  </div>
                );
              })}
            </div>
            <AssignmentHistory history={history} selectedEmployee={selectedEmployee} />
          </div>
        ) : activeTab === "assignments" ? (
          /* TAB: ASSIGNMENTS (Patterns & Rules) */
          <div className="bg-white border border-slate-200 rounded-[40px] p-10 shadow-sm animate-in fade-in zoom-in-95 duration-300">
            <div className="flex items-center gap-6 mb-10 pb-10 border-b border-slate-50">
              <div className="w-20 h-20 bg-primary-600 rounded-[28px] flex items-center justify-center text-white text-3xl font-black shadow-xl shadow-primary-100">{selectedEmployee!.full_name[0]}</div>
              <div>
                <h2 className="text-2xl font-bold text-slate-900">{selectedEmployee!.full_name}</h2>
                <p className="text-[10px] font-black uppercase tracking-widest text-slate-400 mt-1">{t("shifts.fixed_assignment_subtitle")}</p>
              </div>
            </div>
            <div className="space-y-10">
              <div className="space-y-4">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest flex items-center gap-2 ml-1"><CalendarRange size={12} /> {t("patterns.select_pattern")}</label>
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                  {patterns?.map(pattern => (
                    <button key={pattern.id} onClick={() => setSelectedPatternId(pattern.id)} className={`p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group ${selectedPatternId === pattern.id ? 'border-primary-600 bg-primary-50/50' : 'border-slate-100 hover:border-slate-200 bg-slate-50/30'}`}>
                      <div className={`font-black text-sm uppercase tracking-tight ${selectedPatternId === pattern.id ? 'text-primary-700' : 'text-slate-700'}`}>{pattern.name}</div>
                      <p className="text-[10px] text-slate-400 font-bold mt-1 line-clamp-1">{pattern.description || t("common.no_description")}</p>
                      {selectedPatternId === pattern.id && <div className="absolute top-4 right-4 w-3 h-3 bg-primary-600 rounded-full animate-pulse" />}
                    </button>
                  ))}
                </div>
              </div>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                <div className="space-y-3">
                  <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest flex items-center gap-2 ml-1"><Calendar size={12} /> {t("common.effective_from")}</label>
                  <input type="date" value={startDate} onChange={(e) => setStartDate(e.target.value)} className="w-full bg-slate-50 border border-slate-200 rounded-2xl px-5 py-4 font-bold text-slate-700 focus:ring-2 focus:ring-primary-500 outline-none transition-all" />
                </div>
                <div className="space-y-3">
                  <div className="flex justify-between items-center px-1">
                    <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest flex items-center gap-2"><Calendar size={12} /> {t("common.end_date")}</label>
                    <button onClick={() => setIsForever(!isForever)} className={`text-[9px] font-black px-3 py-1 rounded-full border transition-all uppercase tracking-tighter ${isForever ? 'bg-primary-600 text-white border-primary-600' : 'bg-slate-100 text-slate-400 border-slate-200'}`}>{isForever ? t("common.indefinite").toUpperCase() : t("common.definite").toUpperCase()}</button>
                  </div>
                  <input type="date" disabled={isForever} value={endDate} onChange={(e) => setEndDate(e.target.value)} className={`w-full border rounded-2xl px-5 py-4 font-bold transition-all outline-none ${isForever ? 'bg-slate-100 text-slate-300 border-slate-50' : 'bg-slate-50 text-slate-700 border-slate-200 focus:ring-2 focus:ring-primary-500'}`} />
                </div>
              </div>
              <button onClick={handleAssign} disabled={assignPatternMutation.isPending} className="w-full bg-slate-900 text-white font-black py-5 rounded-[28px] flex items-center justify-center gap-3 hover:bg-black transition-all active:scale-[0.98] disabled:opacity-50 shadow-2xl shadow-slate-200 uppercase tracking-widest text-xs"><Save size={18} /> {t("shifts.assign_btn")}</button>
            </div>
          </div>
        ) : (
          /* TAB: DIRECTORY */
          <ShiftList shifts={shifts} onEdit={openEditModal} />
        )}
      </div>
      {/* Modals remain same... */}

      {/* CREATE SHIFT MODAL */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title={t("shifts.modal.setup_title")}
        subtitle={t("shifts.modal.setup_subtitle")}
      >
        <div className="space-y-6 pt-4">
          <div className="space-y-4">
            <div className="grid grid-cols-1 gap-4">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.table.shift_name")}</label>
                <input
                  type="text"
                  value={newShiftName}
                  onChange={(e) => setNewShiftName(e.target.value)}
                  placeholder={t("shifts.modal.name_placeholder")}
                  className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                />
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.modal.start_time")}</label>
                <input
                  type="time"
                  value={newShiftStart}
                  onChange={(e) => setNewShiftStart(e.target.value)}
                  className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                />
              </div>
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.modal.end_time")}</label>
                <input
                  type="time"
                  value={newShiftEnd}
                  onChange={(e) => setNewShiftEnd(e.target.value)}
                  className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                />
              </div>
            </div>

            <div className="space-y-4 bg-slate-50 p-6 rounded-[32px] border border-slate-100">
              <div className="flex items-center justify-between">
                <h4 className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.modal.break_config")}</h4>
                <div className="flex items-center gap-2">
                  <span className="text-[10px] font-bold text-slate-500">{t("shifts.types.fixed")}</span>
                  <button
                    type="button"
                    onClick={() => setIsFixedBreak(!isFixedBreak)}
                    className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${isFixedBreak ? 'bg-primary-600' : 'bg-slate-200'}`}
                  >
                    <span className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform ${isFixedBreak ? 'translate-x-5' : 'translate-x-1'}`} />
                  </button>
                </div>
              </div>

              {isFixedBreak ? (
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <label className="text-[10px] font-bold text-slate-500 uppercase">{t("shifts.modal.break_start")}</label>
                    <input
                      type="time"
                      value={newBreakStart}
                      onChange={(e) => setNewBreakStart(e.target.value)}
                      className="w-full bg-white border-none rounded-xl px-4 py-2.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                    />
                  </div>
                  <div className="space-y-2">
                    <label className="text-[10px] font-bold text-slate-500 uppercase">{t("shifts.modal.break_end")}</label>
                    <input
                      type="time"
                      value={newBreakEnd}
                      onChange={(e) => setNewBreakEnd(e.target.value)}
                      className="w-full bg-white border-none rounded-xl px-4 py-2.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                    />
                  </div>
                </div>
              ) : (
                <div className="space-y-2">
                  <label className="text-[10px] font-bold text-slate-500 uppercase">{t("shifts.modal.break_duration")} ({t("common.minutes")})</label>
                  <input
                    type="number"
                    value={newShiftBreak}
                    onChange={(e) => setNewShiftBreak(parseInt(e.target.value) || 0)}
                    className="w-full bg-white border-none rounded-xl px-4 py-2.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                  />
                </div>
              )}

              <div className="flex items-center gap-3 pt-2">
                <input
                  type="checkbox"
                  id="auto_deduct"
                  checked={isAutoDeduct}
                  onChange={(e) => setIsAutoDeduct(e.target.checked)}
                  className="w-5 h-5 rounded-lg border-slate-200 text-primary-600 focus:ring-primary-500"
                />
                <label htmlFor="auto_deduct" className="text-sm font-bold text-slate-700">
                  {t("shifts.modal.auto_deduct_label")}
                </label>
              </div>
            </div>
          </div>

          <div className="flex gap-3">
            <button
              onClick={() => setIsCreateModalOpen(false)}
              className="flex-1 px-6 py-4 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-100 transition-all"
            >
              {t("common.cancel")}
            </button>
            <button
              onClick={handleCreateShift}
              disabled={createShiftMutation.isPending}
              className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-black px-6 py-4 rounded-2xl flex items-center justify-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 disabled:opacity-50 text-xs uppercase tracking-widest"
            >
              <Save className="w-4 h-4" /> {t("shifts.modal.save_shift_btn")}
            </button>
          </div>
        </div>
      </Modal>

      {/* SHIFT PICKER MODAL */}
      <Modal
        isOpen={isShiftPickerOpen}
        onClose={() => {
          setIsShiftPickerOpen(false);
          setTargetDates([]);
        }}
        title={t("shifts.modal.select_shift_title")}
        subtitle={t("shifts.modal.select_shift_subtitle", { count: targetDates.length })}
      >
        <div className="space-y-6 pt-4">
          <div className="grid grid-cols-1 gap-3">
            {shifts?.map(shift => (
              <button
                key={shift.id}
                onClick={() => {
                  handleBulkAssign(shift.id);
                  setIsShiftPickerOpen(false);
                  setTargetDates([]);
                }}
                className="p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group hover:border-primary-500 hover:bg-primary-50/30 border-slate-100 bg-slate-50/30"
              >
                <div className="font-black text-sm uppercase tracking-tight text-slate-700">{shift.name}</div>
                <div className="flex flex-wrap gap-x-3 gap-y-1 mt-1.5">
                  <div className="text-[10px] text-slate-400 font-bold flex items-center gap-1">
                    <Clock size={10} /> {formatTime(shift.start_time)} - {formatTime(shift.end_time)}
                  </div>
                  <div className="text-[10px] text-slate-400 font-bold flex items-center gap-1">
                    <Info size={10} />
                    {shift.break_start ? (
                      <span>{shift.break_start.substring(0, 5)}-{shift.break_end?.substring(0, 5)}</span>
                    ) : (
                      <span>{t("common.break")} {shift.break_duration_minutes}m</span>
                    )}
                  </div>
                </div>
              </button>
            ))}

            <button
              onClick={() => {
                handleBulkAssign(null);
                setIsShiftPickerOpen(false);
                setTargetDates([]);
              }}
              className="p-5 rounded-[28px] border-2 text-left transition-all relative overflow-hidden group hover:border-slate-300 hover:bg-slate-100 border-slate-100 bg-slate-50/30 border-dashed"
            >
              <div className="font-black text-sm uppercase tracking-tight text-slate-500">{t("shifts.off_day")}</div>
              <div className="text-[10px] text-slate-400 font-bold mt-1.5">
                {t("shifts.modal.mark_as_off")}
              </div>
            </button>
          </div>

          <button
            onClick={() => {
              setIsShiftPickerOpen(false);
              setTargetDates([]);
            }}
            className="w-full px-6 py-4 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-100 transition-all"
          >
            {t("common.cancel")}
          </button>
        </div>
      </Modal>

      {/* EDIT SHIFT MODAL */}
      <Modal
        isOpen={!!editingShift}
        onClose={() => setEditingShift(null)}
        title={t("shifts.modal.setup_title")}
        subtitle={t("shifts.modal.edit_subtitle")}
      >
        <div className="space-y-6 pt-4">
          <div className="space-y-4">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.table.shift_name")}</label>
              <input
                type="text"
                value={newShiftName}
                onChange={(e) => setNewShiftName(e.target.value)}
                placeholder={t("shifts.modal.name_placeholder")}
                className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.modal.start_time")}</label>
                <input
                  type="time"
                  value={newShiftStart}
                  onChange={(e) => setNewShiftStart(e.target.value)}
                  className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                />
              </div>
              <div className="space-y-2">
                <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.modal.end_time")}</label>
                <input
                  type="time"
                  value={newShiftEnd}
                  onChange={(e) => setNewShiftEnd(e.target.value)}
                  className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                />
              </div>
            </div>

            <div className="space-y-4 bg-slate-50 p-6 rounded-[32px] border border-slate-100">
              <div className="flex items-center justify-between">
                <h4 className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("shifts.modal.break_config")}</h4>
                <div className="flex items-center gap-2">
                  <span className="text-[10px] font-bold text-slate-500">{t("shifts.types.fixed")}</span>
                  <button
                    type="button"
                    onClick={() => setIsFixedBreak(!isFixedBreak)}
                    className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${isFixedBreak ? 'bg-primary-600' : 'bg-slate-200'}`}
                  >
                    <span className={`inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform ${isFixedBreak ? 'translate-x-5' : 'translate-x-1'}`} />
                  </button>
                </div>
              </div>

              {isFixedBreak ? (
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <label className="text-[10px] font-bold text-slate-500 uppercase">{t("shifts.modal.break_start")}</label>
                    <input
                      type="time"
                      value={newBreakStart}
                      onChange={(e) => setNewBreakStart(e.target.value)}
                      className="w-full bg-white border-none rounded-xl px-4 py-2.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                    />
                  </div>
                  <div className="space-y-2">
                    <label className="text-[10px] font-bold text-slate-500 uppercase">{t("shifts.modal.break_end")}</label>
                    <input
                      type="time"
                      value={newBreakEnd}
                      onChange={(e) => setNewBreakEnd(e.target.value)}
                      className="w-full bg-white border-none rounded-xl px-4 py-2.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                    />
                  </div>
                </div>
              ) : (
                <div className="space-y-2">
                  <label className="text-[10px] font-bold text-slate-500 uppercase">{t("shifts.modal.break_duration")} ({t("common.minutes")})</label>
                  <input
                    type="number"
                    value={newShiftBreak}
                    onChange={(e) => setNewShiftBreak(parseInt(e.target.value) || 0)}
                    className="w-full bg-white border-none rounded-xl px-4 py-2.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                  />
                </div>
              )}

              <div className="flex items-center gap-3 pt-2">
                <input
                  type="checkbox"
                  id="auto_deduct_edit"
                  checked={isAutoDeduct}
                  onChange={(e) => setIsAutoDeduct(e.target.checked)}
                  className="w-5 h-5 rounded-lg border-slate-200 text-primary-600 focus:ring-primary-500"
                />
                <label htmlFor="auto_deduct_edit" className="text-sm font-bold text-slate-700">
                  {t("shifts.modal.auto_deduct_label")}
                </label>
              </div>

              {/* Grace Period */}
              <div className="space-y-3 pt-2">
                <div className="flex items-center gap-2">
                  <label className="text-[10px] font-bold text-slate-500 uppercase">{t("shifts.grace_period")}</label>
                  <div className="group relative">
                    <Info size={12} className="text-slate-300 cursor-help" />
                    <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 w-48 p-3 bg-slate-800 text-white text-[10px] rounded-xl opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
                      {t("shifts.grace_period_desc")}
                    </div>
                  </div>
                </div>
                <input
                  type="number"
                  min="0"
                  value={newShiftGrace}
                  onChange={(e) => setNewShiftGrace(parseInt(e.target.value) || 0)}
                  className="w-full bg-white border-none rounded-xl px-4 py-2.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
                />
              </div>
            </div>

            <div className="bg-amber-50 p-6 rounded-[32px] border border-amber-100 space-y-4">
              <div className="flex items-center gap-3">
                <div className="bg-amber-100 p-2 rounded-xl text-amber-600">
                  <Zap size={20} />
                </div>
                <div>
                  <h4 className="text-sm font-black text-amber-900 uppercase tracking-tight">{t("shifts.modal.correction_label")}</h4>
                  <p className="text-[10px] font-bold text-amber-700 leading-tight">{t("shifts.modal.correction_desc")}</p>
                </div>
              </div>

              <div className="flex items-center gap-3 bg-white/50 p-3 rounded-2xl">
                <input
                  type="checkbox"
                  id="is_correction"
                  checked={isCorrection}
                  onChange={(e) => setIsCorrection(e.target.checked)}
                  className="w-5 h-5 rounded-lg border-amber-200 text-amber-600 focus:ring-amber-500"
                />
                <label htmlFor="is_correction" className="text-xs font-black text-amber-800 uppercase tracking-widest">
                  {t("shifts.modal.correction_warning")}
                </label>
              </div>

              {!isCorrection && (
                <div className="space-y-2 animate-in fade-in slide-in-from-top-2 duration-300">
                  <label className="text-[10px] font-black text-amber-700 uppercase tracking-widest">{t("shifts.modal.effective_date_label")}</label>
                  <input
                    type="date"
                    value={correctionEffectiveDate}
                    onChange={(e) => setCorrectionEffectiveDate(e.target.value)}
                    className="w-full bg-white border-amber-200 rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-amber-500 transition-all"
                  />
                  <p className="text-[9px] font-bold text-amber-600 leading-tight italic px-1">
                    {t("shifts.modal.effective_date_desc")}
                  </p>
                </div>
              )}
            </div>
          </div>

          <div className="flex gap-3">
            <button
              onClick={() => setEditingShift(null)}
              className="flex-1 px-6 py-4 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-100 transition-all"
            >
              {t("common.cancel")}
            </button>
            <button
              onClick={handleUpdateShift}
              disabled={updateShiftMutation.isPending}
              className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-black px-6 py-4 rounded-2xl flex items-center justify-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 disabled:opacity-50 text-xs uppercase tracking-widest"
            >
              <Save className="w-4 h-4" /> {t("shifts.modal.update_config_btn")}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
