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
  Settings2,
  ChevronDown,
  ChevronUp,
  Search
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
import { cn } from "@/lib/utils";

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
  });

  const deleteMutation = useMutation({
    mutationFn: (patternId: string) => deleteWorkingPattern(wsId, patternId),
    onSuccess: () => {
      toast.success(t("patterns.messages.delete_success"));
      queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
    },
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
    const details = weeklyGrid.map((shiftId, idx) => ({
      day_of_week: idx,
      shift_id: shiftId,
    }));
    createMutation.mutate({
      name: newPatternName,
      description: newPatternDesc,
      details,
    });
  };

  const handleBulkAssign = () => {
    if (!selectedPatternId || selectedEmployeeIds.length === 0) return;
    bulkAssignMutation.mutate({
      employee_ids: selectedEmployeeIds,
      pattern_id: selectedPatternId,
      start_date: assignStartDate,
    });
  };

  const daysOfWeekShort = [
    t("days.sunday_short"), t("days.monday_short"), t("days.tuesday_short"),
    t("days.wednesday_short"), t("days.thursday_short"), t("days.friday_short"), t("days.saturday_short")
  ];

  const daysOfWeekFull = [
    t("days.sunday"), t("days.monday"), t("days.tuesday"),
    t("days.wednesday"), t("days.thursday"), t("days.friday"), t("days.saturday")
  ];

  const prefillStandard = () => {
    if (!shifts || shifts.length === 0) return;
    const defaultShift = shifts[0].id;
    const newGrid = [...weeklyGrid];
    for (let i = 1; i <= 5; i++) newGrid[i] = defaultShift;
    setWeeklyGrid(newGrid);
  };

  if (!wsId) return <div className="p-12 text-center text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-8 px-2">
        <div className="flex items-center gap-4">
          <div className="w-12 h-12 rounded-[1.25rem] bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50"><CalendarDays size={24} /></div>
          <div>
            <h3 data-testid="patterns-title" className="text-2xl font-black tracking-tight text-slate-900 uppercase leading-none">{t("patterns.title")}</h3>
            <p className="text-[10px] font-black text-slate-400 uppercase tracking-widest mt-1.5 opacity-70">{t("patterns.subtitle")}</p>
          </div>
        </div>
        <div className="flex items-center gap-4">
          <button onClick={() => setIsBulkModalOpen(true)} className="flex items-center gap-3 bg-white border border-slate-200 text-slate-700 px-8 py-3 rounded-2xl hover:bg-slate-50 transition-all font-black text-xs uppercase tracking-widest shadow-xl active:scale-95"><Users size={18} /> {t("patterns.bulk_assign_btn").toUpperCase()}</button>
          <button onClick={() => { setIsCreateModalOpen(true); setWeeklyGrid(Array(7).fill(null)); }} className="flex items-center gap-3 bg-primary-600 text-white px-8 py-3 rounded-2xl font-black text-xs uppercase tracking-widest shadow-xl shadow-primary-100 active:scale-95 hover:bg-primary-700"><Plus size={18} /> {t("patterns.create_btn").toUpperCase()}</button>
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-8">
        {isLoading ? (
          Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-white border border-slate-100 p-10 rounded-[2.5rem] shadow-sm animate-pulse h-80" />
          ))
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
                <div className={cn("px-3 py-1 w-fit rounded-full text-[9px] font-black uppercase tracking-widest border shadow-sm", 
                  pattern.is_active ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-slate-100 text-slate-400 border-slate-200"
                )}>{pattern.is_active ? t("common.active").toUpperCase() : t("common.inactive").toUpperCase()}</div>
              </div>
            </div>

            <p className="text-slate-500 text-xs mb-10 line-clamp-2 min-h-[3rem] font-bold uppercase tracking-tight opacity-60 leading-relaxed italic">{pattern.description || t("common.no_description")}</p>

            <button 
              onClick={() => setExpandedPatternId(expandedPatternId === pattern.id ? null : pattern.id)}
              className="mt-auto flex items-center justify-between w-full p-5 bg-slate-50 border border-slate-100 rounded-2xl hover:bg-white hover:border-primary-100 transition-all group/btn"
            >
              <span className="text-[10px] font-black text-slate-400 group-hover/btn:text-primary-600 uppercase tracking-widest transition-colors">{t("patterns.view_details").toUpperCase()}</span>
              {expandedPatternId === pattern.id ? <ChevronUp size={16} className="text-slate-300" /> : <ChevronDown size={16} className="text-slate-300" />}
            </button>

            {expandedPatternId === pattern.id && (
              <div className="mt-6 pt-6 border-t border-slate-100 animate-in slide-in-from-top-4 duration-500">
                <div className="grid grid-cols-7 gap-2">
                  {daysOfWeekShort.map((day, idx) => {
                    const detail = patternDetails?.find(d => d.day_of_week === idx);
                    return (
                      <div key={idx} className="flex flex-col items-center gap-2">
                        <span className="text-[8px] font-black text-slate-400 uppercase tracking-tighter">{day}</span>
                        <div className={cn("w-full aspect-square rounded-lg flex items-center justify-center border shadow-sm", 
                          detail?.shift_id ? "bg-primary-50 border-primary-100 text-primary-600" : "bg-slate-50 border-slate-100 text-slate-300 opacity-40"
                        )}>
                          {detail?.shift_id ? <Clock size={12} /> : <Check size={12} className="opacity-0" />}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Create Modal */}
      <Modal isOpen={isCreateModalOpen} onClose={() => setIsCreateModalOpen(false)} title={t("patterns.modal.create_title")} subtitle={t("patterns.modal.subtitle")} icon={<Plus size={24} />} maxWidth="xl">
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <div className="space-y-3 px-1"><label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.name_label")}</label><input required className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm uppercase tracking-tight shadow-sm" placeholder={t("patterns.form.name_placeholder")} value={newPatternName} onChange={e => setNewPatternName(e.target.value)} /></div>
            <div className="space-y-3 px-1"><label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.desc_label")}</label><input className="w-full px-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-medium text-sm shadow-sm" placeholder={t("patterns.form.desc_placeholder")} value={newPatternDesc} onChange={e => setNewPatternDesc(e.target.value)} /></div>
          </div>

          <div className="space-y-6">
            <div className="flex items-center justify-between px-1">
              <h4 className="text-[10px] font-black text-slate-400 uppercase tracking-[0.3em] flex items-center gap-3"><Clock size={16} className="text-primary-500" /> {t("patterns.form.weekly_grid_title").toUpperCase()}</h4>
              <button onClick={prefillStandard} className="text-[9px] font-black text-primary-600 uppercase tracking-widest hover:text-primary-700 transition-colors bg-primary-50 px-4 py-2 rounded-xl border border-primary-100 shadow-sm active:scale-95">{t("patterns.form.prefill_standard").toUpperCase()}</button>
            </div>
            
            <div className="grid grid-cols-1 md:grid-cols-7 gap-4">
              {daysOfWeekFull.map((day, idx) => (
                <div key={idx} className="space-y-3">
                  <label className="text-[9px] font-black text-slate-400 uppercase tracking-tighter block text-center truncate">{day}</label>
                  <div className="relative group">
                    <select 
                      className={cn("w-full p-3 rounded-2xl text-[10px] font-black uppercase tracking-tight outline-none border transition-all appearance-none cursor-pointer shadow-sm text-center",
                        weeklyGrid[idx] ? "bg-white border-primary-200 text-primary-600 ring-2 ring-primary-500/5 shadow-lg shadow-primary-100/20" : "bg-slate-50 border-slate-100 text-slate-400"
                      )}
                      value={weeklyGrid[idx] || ""}
                      onChange={e => { const newGrid = [...weeklyGrid]; newGrid[idx] = e.target.value || null; setWeeklyGrid(newGrid); }}
                    >
                      <option value="">OFF</option>
                      {shifts?.map(s => <option key={s.id} value={s.id}>{s.name.toUpperCase()}</option>)}
                    </select>
                    {weeklyGrid[idx] && <div className="absolute -top-1.5 -right-1.5 w-4 h-4 bg-primary-600 text-white rounded-full flex items-center justify-center shadow-md animate-in zoom-in-50"><Check size={8} strokeWidth={4} /></div>}
                  </div>
                </div>
              ))}
            </div>
          </div>

          <div className="pt-8 border-t border-slate-50 grid grid-cols-2 gap-6">
            <button onClick={() => setIsCreateModalOpen(false)} className="px-8 py-4 bg-white border border-slate-200 text-slate-500 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button onClick={handleCreate} className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-[0.2em] hover:bg-black transition-all shadow-2xl shadow-slate-400 active:scale-95">{t("patterns.modal.submit_btn").toUpperCase()}</button>
          </div>
        </div>
      </Modal>

      {/* Bulk Assign Modal */}
      <Modal isOpen={isBulkModalOpen} onClose={() => setIsBulkModalOpen(false)} title={t("patterns.bulk_assign_btn")} subtitle={t("patterns.modal.bulk_subtitle")} icon={<Users size={24} />} maxWidth="2xl">
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-10 px-1">
            <div className="space-y-4">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.select_pattern_label").toUpperCase()}</label>
              <div className="grid grid-cols-1 gap-3 max-h-64 overflow-y-auto pr-2 scrollbar-hide">
                {patterns?.map(p => (
                  <button 
                    key={p.id} 
                    onClick={() => setSelectedPatternId(p.id)}
                    className={cn("flex items-center justify-between p-5 rounded-2xl border transition-all text-left group",
                      selectedPatternId === p.id ? "bg-white border-primary-600 shadow-xl shadow-primary-100/20 ring-2 ring-primary-600/10" : "bg-slate-50 border-slate-100 hover:bg-white hover:border-primary-200"
                    )}
                  >
                    <div className="flex items-center gap-4">
                      <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center transition-colors",
                        selectedPatternId === p.id ? "bg-primary-600 text-white" : "bg-white text-slate-300 border border-slate-100 group-hover:text-primary-500"
                      )}><CalendarDays size={20} /></div>
                      <span className={cn("font-black text-xs uppercase tracking-tight", selectedPatternId === p.id ? "text-slate-900" : "text-slate-500")}>{p.name}</span>
                    </div>
                    {selectedPatternId === p.id && <Check size={18} className="text-primary-600" />}
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-4">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.start_date_label").toUpperCase()}</label>
              <div className="relative group">
                <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none text-slate-300 group-focus-within:text-primary-500 transition-colors"><Info size={18} /></div>
                <input type="date" className="w-full pl-14 pr-6 py-4 bg-slate-50 border border-slate-100 rounded-2xl focus:ring-4 focus:ring-primary-500/10 focus:bg-white outline-none transition-all font-black text-sm tracking-tight shadow-inner" value={assignStartDate} onChange={e => setAssignStartDate(e.target.value)} />
              </div>
              <p className="text-[9px] text-slate-400 font-bold uppercase italic leading-relaxed px-1 opacity-60">{t("patterns.form.start_date_note")}</p>
            </div>
          </div>

          <div className="space-y-4 px-1">
            <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest ml-1">{t("patterns.form.select_employees_label").toUpperCase()}</label>
            <EmployeeSearch 
              workspaceId={wsId}
              value={selectedEmployeeIds} 
              onChange={setSelectedEmployeeIds} 
              multiple={true}
              className="bg-slate-50 border-slate-100 rounded-[2rem] p-8 shadow-inner"
            />
          </div>

          <div className="pt-8 border-t border-slate-50 grid grid-cols-2 gap-6">
            <button onClick={() => setIsBulkModalOpen(false)} className="px-8 py-4 bg-white border border-slate-200 text-slate-500 rounded-2xl font-black text-xs uppercase tracking-widest hover:bg-slate-50 transition-all active:scale-95">{t("common.cancel").toUpperCase()}</button>
            <button 
              disabled={!selectedPatternId || selectedEmployeeIds.length === 0} 
              onClick={handleBulkAssign} 
              className="px-8 py-4 bg-slate-900 text-white rounded-2xl font-black text-xs uppercase tracking-[0.2em] hover:bg-black transition-all shadow-2xl shadow-slate-400 active:scale-95 disabled:opacity-50"
            >
              {t("patterns.modal.assign_btn").toUpperCase()}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}