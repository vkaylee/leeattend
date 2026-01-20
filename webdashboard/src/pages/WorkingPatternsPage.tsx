import React, { useState } from "react";
import { useTranslation } from "react-i18next";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { 
  Plus, 
  Trash2, 
  CalendarDays,
  Clock,
  ArrowLeft,
  Info,
  Check,
  Users,
  Settings2
} from "lucide-react";
import { 
  fetchWorkingPatterns, 
  createWorkingPattern, 
  fetchPatternDetails,
  fetchShifts,
  fetchEmployees,
  bulkAssignPattern,
  deleteWorkingPattern,
  getCurrentWorkspaceId,
  WorkingPattern,
  Shift,
  Employee
} from "../lib/api";
import { useToast } from "../components/Toast";
import Modal from "@/components/ui/Modal";
import { format, startOfToday } from "date-fns";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Link } from "react-router-dom";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";

export default function WorkingPatternsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatTime } = useDateTimeFormatter();

  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isBulkModalOpen, setIsBulkModalOpen] = useState(false);
  
  const [newPatternName, setNewPatternName] = useState("");
  const [newPatternDesc, setNewPatternDesc] = useState("");
  
  // Bulk Assign State
  const [selectedPatternId, setSelectedPatternId] = useState<string>("");
  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [assignStartDate, setAssignStartDate] = useState(format(startOfToday(), "yyyy-MM-01"));

  // Weekly grid state: 0=Sun, 1=Mon, ..., 6=Sat. Value = shift_id or null (off)
  const [weeklyGrid, setWeeklyGrid] = useState<(string | null)[]>(Array(7).fill(null));

  const { data: patterns, isLoading } = useQuery({
    queryKey: ["working-patterns", wsId],
    queryFn: () => fetchWorkingPatterns(wsId),
    enabled: !!wsId,
  });

  const { data: shifts } = useQuery({
    queryKey: ["shifts", wsId],
    queryFn: () => fetchShifts(wsId),
    enabled: !!wsId,
  });

  const { data: employees } = useQuery({
    queryKey: ["employees", wsId],
    queryFn: () => fetchEmployees(wsId),
    enabled: !!wsId && isBulkModalOpen,
  });

  // Fetch details for expanded view
  const [expandedPatternId, setExpandedPatternId] = useState<string | null>(null);
  
  const { data: patternDetails } = useQuery({
    queryKey: ["pattern-details", wsId, expandedPatternId],
    queryFn: () => fetchPatternDetails(wsId, expandedPatternId!),
    enabled: !!wsId && !!expandedPatternId,
  });

  const createMutation = useMutation({
    mutationFn: (data: any) => createWorkingPattern(wsId, data),
    onSuccess: () => {
      toast.success(t("patterns.messages.create_success"));
      setIsCreateModalOpen(false);
      setNewPatternName("");
      setNewPatternDesc("");
      setWeeklyGrid(Array(7).fill(null));
      queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
    },
    onError: (error: any) => toast.error(error.message, t("patterns.messages.create_error"))
  });

  const bulkAssignMutation = useMutation({
    mutationFn: (data: any) => bulkAssignPattern(wsId, data),
    onSuccess: (data) => {
      if (data.success_count > 0) {
        toast.success(t("patterns.messages.bulk_assign_success_count", { count: data.success_count }));
        setIsBulkModalOpen(false);
        setSelectedEmployeeIds([]);
        queryClient.invalidateQueries({ queryKey: ["schedules"] });
      } else if (data.failed_employees.length > 0) {
        toast.error(t("patterns.messages.bulk_assign_conflict", { count: data.failed_employees.length }));
      }
    },
    onError: (error: any) => toast.error(error.message, t("patterns.messages.bulk_assign_error"))
  });

  const deleteMutation = useMutation({
    mutationFn: (patternId: string) => deleteWorkingPattern(wsId, patternId),
    onSuccess: () => {
      toast.success(t("patterns.messages.delete_success"));
      queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
    },
    onError: (error: any) => toast.error(error.message)
  });

  const handleDelete = (pattern: WorkingPattern) => {
    if (confirm(t("patterns.confirm_delete", { name: pattern.name }))) {
      deleteMutation.mutate(pattern.id);
    }
  };

  const handleCreate = () => {
    if (!newPatternName) {
      toast.warning(t("patterns.validation_name_required"));
      return;
    }
    const details = weeklyGrid.map((shiftId, idx) => ({ day_of_week: idx, shift_id: shiftId }));
    createMutation.mutate({ name: newPatternName, description: newPatternDesc, details });
  };

  const handleBulkAssign = () => {
    if (!selectedPatternId || selectedEmployeeIds.length === 0) return;
    bulkAssignMutation.mutate({ employee_ids: selectedEmployeeIds, pattern_id: selectedPatternId, start_date: assignStartDate });
  };

  const daysOfWeekShort = [
    t("days.sunday_short", "CN"), t("days.monday_short", "T2"), t("days.tuesday_short", "T3"),
    t("days.wednesday_short", "T4"), t("days.thursday_short", "T5"), t("days.friday_short", "T6"), t("days.saturday_short", "T7")
  ];

  const prefillStandard = () => {
    if (!shifts || shifts.length === 0) return;
    const defaultShift = shifts[0].id;
    const newGrid = [...weeklyGrid];
    for (let i = 1; i <= 5; i++) newGrid[i] = defaultShift;
    setWeeklyGrid(newGrid);
  };

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <div className="flex justify-between items-center px-2">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50"><CalendarDays size={20} /></div>
          <div><h3 className="text-lg font-bold text-slate-800 tracking-tight">{t("patterns.title")}</h3><p className="text-xs text-slate-500 font-medium">{t("patterns.subtitle")}</p></div>
        </div>
        <div className="flex gap-3">
          <button onClick={() => setIsBulkModalOpen(true)} className="flex items-center gap-2 bg-white border border-slate-200 hover:bg-slate-50 text-slate-700 px-5 py-2.5 rounded-2xl transition-all shadow-sm font-black text-[10px] uppercase tracking-widest active:scale-95"><Users size={14} /> {t("patterns.bulk_assign_btn")}</button>
          <button onClick={() => { setIsCreateModalOpen(true); setWeeklyGrid(Array(7).fill(null)); }} className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 font-black text-[10px] uppercase tracking-widest active:scale-95"><Plus size={14} /> {t("patterns.create_btn")}</button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
        {isLoading ? (
          Array.from({ length: 3 }).map((_, i) => (<div key={i} className="bg-white border border-slate-100 p-8 rounded-[32px] shadow-sm animate-pulse h-64" />))
        ) : patterns?.length === 0 ? (
          <div className="col-span-full py-24 text-center border-2 border-dashed border-slate-100 rounded-[40px] bg-slate-50/30 flex flex-col items-center"><div className="w-20 h-20 bg-white rounded-full flex items-center justify-center shadow-sm mb-4"><CalendarDays size={32} className="text-slate-300" /></div><h3 className="text-lg font-bold text-slate-900">{t("patterns.empty_title")}</h3><p className="text-slate-400 text-sm max-w-xs mt-1">{t("patterns.empty_desc")}</p></div>
        ) : patterns?.map(pattern => (
          <div key={pattern.id} className="bg-white border border-slate-100 rounded-[32px] p-8 shadow-sm hover:shadow-xl hover:shadow-primary-100/20 hover:scale-[1.02] transition-all group relative overflow-hidden flex flex-col">
            <div className="absolute top-0 right-0 p-6 flex gap-2 translate-y-2 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all">
              <button onClick={() => handleDelete(pattern)} className="p-2.5 bg-white text-slate-400 hover:text-rose-600 rounded-xl transition-all shadow-md border border-slate-100" title={t("common.delete")}>
                <Trash2 size={16} />
              </button>
            </div>
            <div className="flex items-center gap-4 mb-6">
              <div className="w-14 h-14 bg-primary-50 text-primary-600 rounded-2xl flex items-center justify-center shadow-inner border border-primary-100/50"><CalendarDays size={28} /></div>
              <div><h3 className="font-black text-xl text-slate-900 leading-tight uppercase tracking-tight">{pattern.name}</h3><div className={cn("mt-1.5 px-2 py-0.5 w-fit rounded-lg text-[9px] font-black uppercase tracking-widest border", pattern.is_active ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-slate-100 text-slate-400 border-slate-200")}>{pattern.is_active ? t("common.active") : t("common.inactive")}</div></div>
            </div>
export default function WorkingPatternsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatTime } = useDateTimeFormatter();

  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isBulkModalOpen, setIsBulkModalOpen] = useState(false);
  const [newPatternName, setNewPatternName] = useState("");
  const [newPatternDesc, setNewPatternDesc] = useState("");
  const [selectedPatternId, setSelectedPatternId] = useState<string>("");
  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [assignStartDate, setAssignStartDate] = useState(format(startOfToday(), "yyyy-MM-01"));
  const [weeklyGrid, setWeeklyGrid] = useState<(string | null)[]>(Array(7).fill(null));

  const { data: patterns, isLoading } = useQuery({ queryKey: ["working-patterns", wsId], queryFn: () => fetchWorkingPatterns(wsId), enabled: !!wsId });
  const { data: shifts } = useQuery({ queryKey: ["shifts", wsId], queryFn: () => fetchShifts(wsId), enabled: !!wsId });
  const [expandedPatternId, setExpandedPatternId] = useState<string | null>(null);
  const { data: patternDetails } = useQuery({ queryKey: ["pattern-details", wsId, expandedPatternId], queryFn: () => fetchPatternDetails(wsId, expandedPatternId!), enabled: !!wsId && !!expandedPatternId });

  const createMutation = useMutation({
    mutationFn: (data: any) => createWorkingPattern(wsId, data),
    onSuccess: () => { toast.success(t("patterns.messages.create_success")); setIsCreateModalOpen(false); setNewPatternName(""); setNewPatternDesc(""); setWeeklyGrid(Array(7).fill(null)); queryClient.invalidateQueries({ queryKey: ["working-patterns"] }); },
  });

  const bulkAssignMutation = useMutation({
    mutationFn: (data: any) => bulkAssignPattern(wsId, data),
    onSuccess: (data) => {
      if (data.success_count > 0) { toast.success(t("patterns.messages.bulk_assign_success_count", { count: data.success_count })); setIsBulkModalOpen(false); setSelectedEmployeeIds([]); queryClient.invalidateQueries({ queryKey: ["schedules"] }); }
      else if (data.failed_employees.length > 0) { toast.error(t("patterns.messages.bulk_assign_conflict", { count: data.failed_employees.length })); }
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (patternId: string) => deleteWorkingPattern(wsId, patternId),
    onSuccess: () => { toast.success(t("patterns.messages.delete_success")); queryClient.invalidateQueries({ queryKey: ["working-patterns"] }); },
  });

  const handleDelete = (pattern: WorkingPattern) => { if (confirm(t("patterns.confirm_delete", { name: pattern.name }))) { deleteMutation.mutate(pattern.id); } };
  const handleCreate = () => {
    if (!newPatternName) { toast.warning(t("patterns.validation_name_required")); return; }
    const details = weeklyGrid.map((shiftId, idx) => ({ day_of_week: idx, shift_id: shiftId }));
    createMutation.mutate({ name: newPatternName, description: newPatternDesc, details });
  };

  const handleBulkAssign = () => { if (!selectedPatternId || selectedEmployeeIds.length === 0) return; bulkAssignMutation.mutate({ employee_ids: selectedEmployeeIds, pattern_id: selectedPatternId, start_date: assignStartDate }); };

  const daysOfWeekShort = [ t("days.sunday_short"), t("days.monday_short"), t("days.tuesday_short"), t("days.wednesday_short"), t("days.thursday_short"), t("days.friday_short"), t("days.saturday_short") ];
  const daysOfWeekFull = [ t("days.sunday"), t("days.monday"), t("days.tuesday"), t("days.wednesday"), t("days.thursday"), t("days.friday"), t("days.saturday") ];

  const prefillStandard = () => {
    if (!shifts || shifts.length === 0) return; const defaultShift = shifts[0].id; const newGrid = [...weeklyGrid];
    for (let i = 1; i <= 5; i++) newGrid[i] = defaultShift; setWeeklyGrid(newGrid);
  };

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50"><CalendarDays size={24} /></div>
          <div><h3 className="text-2xl font-black tracking-tight text-slate-900 uppercase leading-none">{t("patterns.title")}</h3><p className="text-[10px] font-black text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">{t("patterns.subtitle")}</p></div>
        </div>
        <div className="flex items-center gap-4">
          <button onClick={() => setIsBulkModalOpen(true)} className="flex items-center gap-3 bg-white border border-slate-200 text-slate-700 px-8 py-3 rounded-2xl hover:bg-slate-50 transition-all font-black text-xs uppercase tracking-widest shadow-xl active:scale-95"><Users size={18} /> {t("patterns.bulk_assign_btn").toUpperCase()}</button>
          <button onClick={() => { setIsCreateModalOpen(true); setWeeklyGrid(Array(7).fill(null)); }} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-black text-xs uppercase tracking-widest shadow-xl shadow-primary-100 active:scale-95 hover:bg-primary-700"><Plus size={18} /> {t("patterns.create_btn").toUpperCase()}</button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-8">
        {isLoading ? (
          Array.from({ length: 3 }).map((_, i) => (<div key={i} className="bg-white border border-slate-100 p-10 rounded-[2.5rem] shadow-sm animate-pulse h-80" />))
        ) : patterns?.length === 0 ? (
          <div className="col-span-full py-32 text-center border-2 border-dashed border-slate-100 rounded-[3rem] bg-slate-50/30 flex flex-col items-center animate-in zoom-in-95 duration-500">
            <div className="w-24 h-24 bg-white rounded-full flex items-center justify-center shadow-sm mb-6 border border-slate-100"><CalendarDays size={40} className="text-slate-200" /></div>
            <h3 className="text-xl font-bold text-slate-900 uppercase tracking-tight">{t("patterns.empty_title")}</h3>
            <p className="text-slate-400 text-sm font-black uppercase tracking-widest mt-1 opacity-50 italic">{t("patterns.empty_desc")}</p>
          </div>
        ) : patterns?.map(pattern => (
          <div key={pattern.id} className="bg-white border border-slate-100 rounded-[2.5rem] p-10 shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 hover:scale-[1.02] transition-all group relative overflow-hidden flex flex-col">
            <div className="absolute top-0 right-0 p-8 flex gap-3 translate-y-4 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-500">
              <button onClick={() => handleDelete(pattern)} className="p-3 bg-white text-slate-400 hover:text-rose-600 rounded-xl shadow-md border border-slate-100 transition-all active:scale-90" title={t("common.delete")}><Trash2 size={18} /></button>
            </div>
            <div className="flex items-center gap-5 mb-8">
              <div className="w-16 h-16 bg-primary-50 text-primary-600 rounded-[1.5rem] flex items-center justify-center shadow-inner border border-primary-100/50 group-hover:scale-110 group-hover:rotate-3 transition-all duration-500"><CalendarDays size={32} /></div>
              <div>
                <h3 className="font-black text-2xl text-slate-900 leading-none uppercase tracking-tight mb-2">{pattern.name}</h3>
                <span className={cn("px-3 py-1 rounded-full text-[9px] font-black uppercase tracking-widest border shadow-sm", pattern.is_active ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-slate-50 text-slate-400 border-slate-100 opacity-60")}>{pattern.is_active ? t("common.active").toUpperCase() : t("common.inactive").toUpperCase()}</span>
              </div>
            </div>
            <p className="text-slate-500 text-xs mb-10 line-clamp-2 min-h-[3rem] font-bold uppercase tracking-tight opacity-60 leading-relaxed italic">{pattern.description || t("common.no_description")}</p>
            
            <div className="mt-auto space-y-6 pt-8 border-t border-slate-50">
              <div className="flex justify-between items-center px-1">
                <span className="text-[10px] font-black uppercase tracking-[0.25em] text-slate-400 opacity-70">{t("patterns.weekly_schedule")}</span>
                <button onClick={() => setExpandedPatternId(expandedPatternId === pattern.id ? null : pattern.id)} className="text-[10px] font-black uppercase tracking-widest text-primary-600 hover:text-primary-700 transition-all active:scale-90">
                  {expandedPatternId === pattern.id ? t("common.collapse").toUpperCase() : t("common.expand").toUpperCase()}
                </button>
              </div>
              
              {expandedPatternId === pattern.id && patternDetails ? (
                <div className="space-y-3 animate-in slide-in-from-top-4 duration-500 p-6 bg-slate-50 rounded-[2rem] border border-slate-100 shadow-inner">
                  {patternDetails.map(detail => {
                    const shift = shifts?.find(s => s.id === detail.shift_id);
                    return (
                      <div key={detail.day_of_week} className="flex items-center justify-between text-[10px] py-2 border-b border-slate-200/50 last:border-0 px-1">
                        <span className="font-black text-slate-500 w-24 uppercase tracking-widest">{daysOfWeekFull[detail.day_of_week]}</span>
                        {shift ? (<div className="flex items-center gap-3 flex-1 justify-end"><span className="bg-white text-primary-600 px-2.5 py-1 rounded-lg font-black border border-primary-100 shadow-sm uppercase tracking-tighter text-[9px]">{shift.name}</span><span className="text-[10px] text-slate-400 font-mono font-bold tracking-tighter opacity-70">{formatTime(shift.start_time)}—{formatTime(shift.end_time)}</span></div>) : (<span className="text-slate-300 font-black tracking-[0.2em] italic uppercase opacity-50">{t("shifts.off_day")}</span>)}
                      </div>
                    );
                  })}
                </div>
              ) : (
                 <div className="flex gap-2 px-1 pb-2">
                    {Array.from({ length: 7 }).map((_, i) => (
                      <div key={i} className="flex-1 flex flex-col items-center gap-3">
                        <div className="h-3 w-full rounded-full bg-slate-100 overflow-hidden shadow-inner border border-slate-200/20"><div className={cn("h-full transition-all duration-1000", i >= 1 && i <= 5 ? "bg-primary-500 shadow-lg shadow-primary-500/50" : "bg-slate-200/50")} style={{ width: '100%' }} /></div>
                        <span className="text-[9px] font-black text-slate-400 uppercase tracking-widest opacity-60 leading-none">{daysOfWeekShort[i]}</span>
                      </div>
                    ))}
                 </div>
              )}
            </div>
          </div>
        ))}
      </div>

      <Modal isOpen={isCreateModalOpen} onClose={() => setIsCreateModalOpen(false)} title={t("patterns.create_title")} subtitle={t("patterns.create_subtitle")} icon={<CalendarDays size={24} />} maxWidth="2xl">
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1"><label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.name")}</label><input type="text" value={newPatternName} onChange={(e) => setNewPatternName(e.target.value)} placeholder={t("patterns.form.name_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-6 py-4 font-black text-sm text-slate-900 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all uppercase tracking-tight shadow-sm" /></div>
            <div className="space-y-3 px-1"><label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.description")}</label><input type="text" value={newPatternDesc} onChange={(e) => setNewPatternDesc(e.target.value)} placeholder={t("patterns.form.desc_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-6 py-4 font-bold text-slate-700 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all text-sm shadow-sm" /></div>
          </div>
          <div className="bg-slate-50 rounded-[2.5rem] p-10 border border-slate-100 shadow-inner space-y-10">
             <div className="flex items-center justify-between border-b border-slate-200/50 pb-6"><div className="flex items-center gap-3"><Clock size={20} className="text-primary-500 opacity-50" /><h4 className="text-[10px] font-black text-slate-500 uppercase tracking-[0.25em]">{t("patterns.weekly_config")}</h4></div><button onClick={prefillStandard} className="text-[10px] font-black text-primary-600 uppercase tracking-[0.2em] hover:text-primary-700 transition-all bg-white px-4 py-2 rounded-xl border border-primary-100 shadow-sm active:scale-95">{t("patterns.actions.prefill_standard").toUpperCase()}</button></div>
             <div className="grid grid-cols-1 md:grid-cols-2 gap-x-12 gap-y-6">
                {daysOfWeekFull.map((dayName, idx) => (
                  <div key={idx} className="flex items-center justify-between gap-6 p-1 group">
                     <div className="w-28 font-black text-[11px] text-slate-400 uppercase tracking-[0.2em] group-hover:text-primary-600 transition-colors">{dayName}</div>
                     <select value={weeklyGrid[idx] || ""} onChange={(e) => { const val = e.target.value || null; const newGrid = [...weeklyGrid]; newGrid[idx] = val; setWeeklyGrid(newGrid); }} className={cn("flex-1 rounded-[1.25rem] border py-3.5 px-6 font-black text-[11px] transition-all outline-none focus:ring-4 focus:ring-primary-500/10 uppercase tracking-widest shadow-sm cursor-pointer", weeklyGrid[idx] ? "bg-white text-primary-600 border-primary-100" : "bg-slate-100/50 text-slate-400 border-transparent hover:border-slate-200")}>
                        <option value="">{t("shifts.off_day").toUpperCase()}</option>
                        {shifts?.map(s => (<option key={s.id} value={s.id}>{s.name} ({formatTime(s.start_time)}-{formatTime(s.end_time)})</option>))}
                     </select>
                  </div>
                ))}
             </div>
          </div>
          <div className="flex gap-6 pt-4 border-t border-slate-50">
            <button onClick={() => setIsCreateModalOpen(false)} className="flex-1 px-8 py-4 bg-white border border-slate-200 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button onClick={handleCreate} disabled={createMutation.isPending} className="flex-[1.5] bg-primary-600 hover:bg-primary-700 text-white font-black px-8 py-4 rounded-2xl flex items-center justify-center gap-4 transition-all active:scale-95 disabled:opacity-50 shadow-2xl shadow-primary-100 text-xs uppercase tracking-widest"><Check size={20} /> {t("patterns.create_submit").toUpperCase()}</button>
          </div>
        </div>
      </Modal>

      <Modal isOpen={isBulkModalOpen} onClose={() => setIsBulkModalOpen(false)} title={t("patterns.bulk_assign_title")} subtitle={t("patterns.bulk_assign_subtitle")} icon={<Users size={24} />} maxWidth="2xl">
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.select_pattern")}</label>
              <select value={selectedPatternId} onChange={(e) => setSelectedPatternId(e.target.value)} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-6 py-4 font-black text-sm text-slate-900 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all uppercase tracking-tight shadow-sm appearance-none cursor-pointer"><option value="">{t("patterns.form.select_pattern_placeholder").toUpperCase()}</option>{patterns?.map(p => (<option key={p.id} value={p.id}>{p.name.toUpperCase()}</option>))}</select>
            </div>
            <div className="space-y-3 px-1">
               <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.start_date")}</label>
               <input type="date" value={assignStartDate} onChange={(e) => setAssignStartDate(e.target.value)} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-6 py-4 font-black text-sm text-slate-900 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all shadow-sm" />
            </div>
          </div>
          <div className="space-y-4">
            <div className="flex items-center justify-between mb-2 px-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-[0.25em] opacity-70">{t("patterns.form.select_employees")}</label>
              <div className="flex gap-4"><button onClick={async () => { const allEmps = await fetchEmployees(wsId); setSelectedEmployeeIds(allEmps.map(e => e.id)); }} className="text-[9px] font-black text-primary-600 uppercase tracking-widest hover:bg-primary-50 px-3 py-1 rounded-lg transition-all">{t("common.select_all").toUpperCase()}</button><button onClick={() => setSelectedEmployeeIds([])} className="text-[9px] font-black text-slate-400 uppercase tracking-widest hover:bg-slate-50 px-3 py-1 rounded-lg transition-all">{t("common.deselect_all").toUpperCase()}</button></div>
            </div>
            <div className="bg-slate-50/50 p-2 rounded-[2.5rem] border border-slate-100 shadow-inner"><EmployeeSearch workspaceId={wsId} multiple={true} value={selectedEmployeeIds} onChange={setSelectedEmployeeIds} placeholder={t("patterns.form.select_employees_placeholder")} /></div>
          </div>
          <div className="flex gap-6 pt-4 border-t border-slate-50">
            <button onClick={() => setIsBulkModalOpen(false)} className="flex-1 px-8 py-4 bg-white border border-slate-200 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button onClick={handleBulkAssign} disabled={bulkAssignMutation.isPending || selectedEmployeeIds.length === 0 || !selectedPatternId} className="flex-[1.5] bg-primary-600 hover:bg-primary-700 text-white font-black px-8 py-4 rounded-2xl flex items-center justify-center gap-4 transition-all active:scale-95 disabled:opacity-50 shadow-2xl shadow-primary-100 text-xs uppercase tracking-widest"><Check size={20} /> {t("patterns.bulk_assign_submit").toUpperCase()}</button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
            <div className="mt-auto space-y-4 pt-6 border-t border-slate-50">
              <div className="flex justify-between items-center px-1">
                <span className="text-[10px] font-black uppercase tracking-widest text-slate-400">{t("patterns.weekly_schedule")}</span>
                <button onClick={() => setExpandedPatternId(expandedPatternId === pattern.id ? null : pattern.id)} className="text-[9px] font-black uppercase tracking-widest text-primary-600 hover:text-primary-700 transition-colors">
                  {expandedPatternId === pattern.id ? t("common.collapse") : t("common.expand")}
                </button>
              </div>
              {expandedPatternId === pattern.id && patternDetails ? (
                <div className="space-y-2 animate-in slide-in-from-top-2 p-4 bg-slate-50 rounded-[1.5rem] border border-slate-100 shadow-inner">
                  {patternDetails.map(detail => {
                    const shift = shifts?.find(s => s.id === detail.shift_id);
                    return (
                      <div key={detail.day_of_week} className="flex items-center justify-between text-[10px] py-1 border-b border-slate-200/50 last:border-0">
                        <span className="font-black text-slate-500 w-20 uppercase tracking-tighter">{daysOfWeekFull[detail.day_of_week]}</span>
                        {shift ? (<div className="flex items-center gap-2 flex-1 justify-end"><span className="bg-white text-primary-700 px-2 py-0.5 rounded-lg font-black border border-primary-100 truncate max-w-[120px] shadow-sm uppercase tracking-tighter">{shift.name}</span><span className="text-[9px] text-slate-400 font-mono font-bold">{formatTime(shift.start_time)}-{formatTime(shift.end_time)}</span></div>) : (<span className="text-slate-300 font-black tracking-widest italic uppercase">{t("shifts.off_day")}</span>)}
                      </div>
                    );
                  })}
                </div>
              ) : (
                 <div className="flex gap-1.5 px-1 pb-1">
                    {Array.from({ length: 7 }).map((_, i) => (
                      <div key={i} className="flex-1 flex flex-col items-center gap-2">
                        <div className="h-2 w-full rounded-full bg-slate-100 overflow-hidden shadow-inner"><div className={cn("w-full h-full transition-all duration-1000", i >= 1 && i <= 5 ? "bg-primary-500" : "bg-slate-200")} /></div>
                        <span className="text-[8px] font-black text-slate-400 uppercase tracking-tighter">{daysOfWeekShort[i]}</span>
                      </div>
                    ))}
                 </div>
              )}
            </div>
          </div>
        ))}
      </div>

      <Modal isOpen={isCreateModalOpen} onClose={() => setIsCreateModalOpen(false)} title={t("patterns.create_title")} subtitle={t("patterns.create_subtitle")} icon={<CalendarDays size={24} />} maxWidth="2xl">
        <div className="space-y-8 pt-4">
          <div className="grid grid-cols-1 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.name")}</label>
              <input type="text" value={newPatternName} onChange={(e) => setNewPatternName(e.target.value)} placeholder={t("patterns.form.name_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 font-black text-sm text-slate-900 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all uppercase tracking-tight" />
            </div>
            <div className="space-y-2">
               <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.description")}</label>
               <input type="text" value={newPatternDesc} onChange={(e) => setNewPatternDesc(e.target.value)} placeholder={t("patterns.form.desc_placeholder")} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 font-bold text-slate-700 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all text-sm" />
            </div>
          </div>
          <div className="bg-slate-50 rounded-[2.5rem] p-8 border border-slate-100 shadow-inner space-y-6">
             <div className="flex items-center justify-between mb-2">
                <h4 className="text-[10px] font-black text-slate-400 uppercase tracking-[0.2em] flex items-center gap-2"><Clock size={14} /> {t("patterns.weekly_config")}</h4>
                <button onClick={prefillStandard} className="text-[10px] font-black text-primary-600 uppercase tracking-widest hover:underline">{t("patterns.actions.prefill_standard")}</button>
             </div>
             <div className="space-y-3">
                {daysOfWeekFull.map((dayName, idx) => (
                  <div key={idx} className="flex items-center gap-6 p-1">
                     <div className="w-32 font-black text-xs text-slate-500 uppercase tracking-tighter">{dayName}</div>
                     <select value={weeklyGrid[idx] || ""} onChange={(e) => { const val = e.target.value || null; const newGrid = [...weeklyGrid]; newGrid[idx] = val; setWeeklyGrid(newGrid); }} className={cn("flex-1 rounded-xl border border-slate-200 py-3 px-5 font-black text-xs transition-all outline-none focus:ring-4 focus:ring-primary-500/10 uppercase tracking-tight shadow-sm", weeklyGrid[idx] ? "bg-white text-primary-600 border-primary-100" : "bg-slate-100 text-slate-400")}>
                        <option value="">{t("shifts.off_day").toUpperCase()}</option>
                        {shifts?.map(s => (<option key={s.id} value={s.id}>{s.name} ({formatTime(s.start_time)}-{formatTime(s.end_time)})</option>))}
                     </select>
                  </div>
                ))}
             </div>
          </div>
          <div className="flex gap-4 pt-2">
            <button onClick={() => setIsCreateModalOpen(false)} className="flex-1 px-8 py-4 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-50 transition-all">{t("common.cancel")}</button>
            <button onClick={handleCreate} disabled={createMutation.isPending} className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-black px-8 py-4 rounded-2xl flex items-center justify-center gap-3 transition-all active:scale-95 disabled:opacity-50 shadow-lg shadow-primary-100 text-xs uppercase tracking-widest"><Check size={18} /> {t("patterns.create_submit")}</button>
          </div>
        </div>
      </Modal>

      <Modal isOpen={isBulkModalOpen} onClose={() => setIsBulkModalOpen(false)} title={t("patterns.bulk_assign_title")} subtitle={t("patterns.bulk_assign_subtitle")} icon={<Users size={24} />} maxWidth="2xl">
        <div className="space-y-8 pt-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.select_pattern")}</label>
              <select value={selectedPatternId} onChange={(e) => setSelectedPatternId(e.target.value)} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 font-black text-sm text-slate-900 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all uppercase tracking-tight shadow-sm"><option value="">{t("patterns.form.select_pattern_placeholder")}</option>{patterns?.map(p => (<option key={p.id} value={p.id}>{p.name}</option>))}</select>
            </div>
            <div className="space-y-2">
               <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.start_date")}</label>
               <input type="date" value={assignStartDate} onChange={(e) => setAssignStartDate(e.target.value)} className="w-full bg-slate-50 border border-slate-100 rounded-2xl px-5 py-4 font-black text-sm text-slate-900 focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all" />
            </div>
          </div>
          <div className="space-y-3">
            <div className="flex items-center justify-between mb-1 px-1">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("patterns.form.select_employees")}</label>
              <div className="flex gap-3"><button onClick={async () => { const allEmps = await fetchEmployees(wsId); setSelectedEmployeeIds(allEmps.map(e => e.id)); }} className="text-[10px] font-black text-primary-600 uppercase tracking-widest hover:underline">{t("common.select_all")}</button><button onClick={() => setSelectedEmployeeIds([])} className="text-[10px] font-black text-slate-400 uppercase tracking-widest hover:underline">{t("common.deselect_all")}</button></div>
            </div>
            <EmployeeSearch workspaceId={wsId} multiple={true} value={selectedEmployeeIds} onChange={setSelectedEmployeeIds} placeholder={t("patterns.form.select_employees_placeholder")} />
          </div>
          <div className="flex gap-4 pt-4 border-t border-slate-50">
            <button onClick={() => setIsBulkModalOpen(false)} className="flex-1 px-8 py-4 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-100 transition-all">{t("common.cancel")}</button>
            <button onClick={handleBulkAssign} disabled={bulkAssignMutation.isPending || selectedEmployeeIds.length === 0 || !selectedPatternId} className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-black px-8 py-4 rounded-2xl flex items-center justify-center gap-3 transition-all active:scale-95 disabled:opacity-50 shadow-lg shadow-primary-100 text-xs uppercase tracking-widest"><Check size={18} /> {t("patterns.bulk_assign_submit")}</button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
