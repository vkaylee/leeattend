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
      {/* Header */}
      <div className="flex justify-between items-center px-2">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-2xl bg-primary-50 flex items-center justify-center text-primary-600 shadow-sm border border-primary-100/50">
            <CalendarDays size={20} />
          </div>
          <div>
            <h3 className="text-lg font-bold text-slate-800 tracking-tight">{t("patterns.title")}</h3>
            <p className="text-xs text-slate-500 font-medium">{t("patterns.subtitle")}</p>
          </div>
        </div>
        <div className="flex gap-3">
          <button 
            onClick={() => setIsBulkModalOpen(true)}
            className="flex items-center gap-2 bg-white border border-slate-200 hover:bg-slate-50 text-slate-700 px-5 py-2.5 rounded-2xl transition-all shadow-sm font-black text-[10px] uppercase tracking-widest active:scale-95"
          >
            <Users size={14} /> {t("patterns.bulk_assign_btn")}
          </button>
          <button 
            onClick={() => setIsCreateModalOpen(true)}
            className="flex items-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 font-black text-[10px] uppercase tracking-widest active:scale-95"
          >
            <Plus size={14} /> {t("patterns.create_btn")}
          </button>
        </div>
      </div>

      {/* Grid of Patterns */}
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
        {isLoading ? (
          Array.from({ length: 3 }).map((_, i) => (
            <div key={i} className="bg-white border border-slate-100 p-8 rounded-[32px] shadow-sm animate-pulse">
              <div className="w-14 h-14 bg-slate-100 rounded-2xl mb-6" />
              <div className="h-6 bg-slate-100 rounded-lg w-2/3 mb-4" />
              <div className="h-12 bg-slate-50 rounded-xl w-full" />
            </div>
          ))
        ) : patterns?.length === 0 ? (
          <div className="col-span-full py-24 text-center border-2 border-dashed border-slate-100 rounded-[40px] bg-slate-50/30 flex flex-col items-center">
            <div className="w-20 h-20 bg-white rounded-full flex items-center justify-center shadow-sm mb-4">
              <CalendarDays size={32} className="text-slate-300" />
            </div>
            <h3 className="text-lg font-bold text-slate-900">{t("patterns.empty_title")}</h3>
            <p className="text-slate-400 text-sm max-w-xs mt-1">{t("patterns.empty_desc")}</p>
          </div>
        ) : patterns?.map(pattern => (
          <div key={pattern.id} className="bg-white border border-slate-100 rounded-[32px] p-8 shadow-sm hover:shadow-xl hover:shadow-primary-100/20 hover:scale-[1.02] transition-all group relative overflow-hidden flex flex-col">
            <div className="absolute top-0 right-0 p-6 flex gap-2 translate-y-2 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all">
              <button 
                onClick={() => handleDelete(pattern)}
                className="p-2.5 bg-white text-slate-400 hover:text-rose-600 rounded-xl transition-all shadow-md border border-slate-100"
                title={t("common.delete")}
              >
                <Trash2 size={16} />
              </button>
            </div>

            <div className="flex items-center gap-4 mb-6">
              <div className="w-14 h-14 bg-purple-50 text-purple-600 rounded-2xl flex items-center justify-center shadow-inner border border-purple-100/50">
                <CalendarDays size={28} />
              </div>
              <div>
                <h3 className="font-black text-xl text-slate-900 leading-tight uppercase tracking-tight">{pattern.name}</h3>
                <div className={`mt-1.5 px-2 py-0.5 w-fit rounded-lg text-[9px] font-black uppercase tracking-widest ${pattern.is_active ? 'bg-emerald-50 text-emerald-600 border border-emerald-100' : 'bg-slate-100 text-slate-400'}`}>
                  {pattern.is_active ? t("common.active") : t("common.inactive")}
                </div>
              </div>
            </div>

            <p className="text-slate-500 text-xs mb-8 line-clamp-2 min-h-[2.5rem] font-bold leading-relaxed opacity-70">
              {pattern.description || t("common.no_description")}
            </p>

            <div className="mt-auto space-y-4 pt-6 border-t border-slate-50">
              <div className="flex justify-between items-center">
                <span className="text-[10px] font-black uppercase tracking-widest text-slate-400">{t("patterns.weekly_schedule")}</span>
                <button 
                   onClick={() => setExpandedPatternId(expandedPatternId === pattern.id ? null : pattern.id)}
                   className="text-[10px] font-black uppercase tracking-widest text-primary-600 hover:text-primary-700 transition-colors"
                >
                  {expandedPatternId === pattern.id ? t("common.collapse") : t("common.expand")}
                </button>
              </div>
              
              {expandedPatternId === pattern.id && patternDetails ? (
                <div className="space-y-2 animate-in slide-in-from-top-2 p-4 bg-slate-50 rounded-2xl border border-slate-100">
                  {patternDetails.map(detail => {
                    const shift = shifts?.find(s => s.id === detail.shift_id);
                    return (
                      <div key={detail.day_of_week} className="flex items-center justify-between text-[10px] py-1">
                        <span className="font-black text-slate-500 w-20">{daysOfWeekShort[detail.day_of_week]}</span>
                        {shift ? (
                          <div className="flex items-center gap-2 flex-1 justify-end">
                            <span className="bg-white text-primary-700 px-2 py-0.5 rounded-md font-black border border-primary-100 truncate max-w-[120px] shadow-sm">{shift.name}</span>
                            <span className="text-[9px] text-slate-400 font-mono font-bold">{formatTime(shift.start_time)}-{formatTime(shift.end_time)}</span>
                          </div>
                        ) : (
                          <span className="text-slate-300 font-black tracking-widest italic">{t("shifts.off_day")}</span>
                        )}
                      </div>
                    );
                  })}
                </div>
              ) : (
                 <div className="flex gap-1.5 px-1">
                    {/* Visual Dots Preview */}
                    {Array.from({ length: 7 }).map((_, i) => (
                      <div key={i} className="flex-1 flex flex-col items-center gap-1.5">
                        <div className="h-1.5 w-full rounded-full bg-slate-100 overflow-hidden shadow-inner">
                           <div className={`w-full h-full ${i >= 1 && i <= 5 ? 'bg-primary-400' : 'bg-slate-200'}`} /> 
                        </div>
                        <span className="text-[8px] font-black text-slate-300">{daysOfWeekShort[i]}</span>
                      </div>
                    ))}
                 </div>
              )}
            </div>
          </div>
        ))}
      </div>

      {/* Create Modal */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title={t("patterns.create_title")}
        subtitle={t("patterns.create_subtitle")}
        maxWidth="2xl"
      >
        <div className="space-y-6 pt-4">
          <div className="grid grid-cols-1 gap-4">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("patterns.form.name")}</label>
              <input
                type="text"
                value={newPatternName}
                onChange={(e) => setNewPatternName(e.target.value)}
                placeholder={t("patterns.form.name_placeholder")}
                className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>
            
            <div className="space-y-2">
               <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("patterns.form.description")}</label>
               <input
                type="text"
                value={newPatternDesc}
                onChange={(e) => setNewPatternDesc(e.target.value)}
                placeholder={t("patterns.form.desc_placeholder")}
                className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-700 focus:ring-2 focus:ring-primary-500 transition-all text-sm"
              />
            </div>
          </div>

          <div className="bg-slate-50 rounded-[32px] p-6 border border-slate-100">
             <div className="flex items-center justify-between mb-4">
                <h4 className="text-xs font-black text-slate-500 uppercase tracking-widest">{t("patterns.weekly_config")}</h4>
                <button 
                  onClick={prefillStandard}
                  className="text-[10px] font-bold text-primary-600 hover:text-primary-700 hover:underline"
                >
                  {t("patterns.actions.prefill_standard")}
                </button>
             </div>

             <div className="space-y-3">
                {daysOfWeek.map((dayName, idx) => (
                  <div key={idx} className="flex items-center gap-4">
                     <div className="w-24 font-bold text-sm text-slate-700">{dayName}</div>
                     <select
                        value={weeklyGrid[idx] || ""}
                        onChange={(e) => {
                          const val = e.target.value || null;
                          const newGrid = [...weeklyGrid];
                          newGrid[idx] = val;
                          setWeeklyGrid(newGrid);
                        }}
                        className={`flex-1 rounded-xl border-none py-2.5 px-4 font-bold text-xs transition-all outline-none focus:ring-2 focus:ring-primary-500 ${weeklyGrid[idx] ? 'bg-white text-slate-900 shadow-sm' : 'bg-slate-100 text-slate-400'}`}
                     >
                        <option value="">{t("shifts.off_day")}</option>
                        {shifts?.map(s => (
                          <option key={s.id} value={s.id}>
                            {s.name} ({formatTime(s.start_time)} - {formatTime(s.end_time)})
                          </option>
                        ))}
                     </select>
                  </div>
                ))}
             </div>
          </div>

          <div className="flex gap-3 pt-2">
            <button
              onClick={() => setIsCreateModalOpen(false)}
              className="flex-1 px-6 py-4 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-100 transition-all"
            >
              {t("common.cancel")}
            </button>
            <button
              onClick={handleCreate}
              disabled={createMutation.isPending}
              className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-black px-6 py-4 rounded-2xl flex items-center justify-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 disabled:opacity-50 text-xs uppercase tracking-widest"
            >
              <Check className="w-4 h-4" /> {t("patterns.create_submit")}
            </button>
          </div>
        </div>
      </Modal>

      {/* Bulk Assign Modal */}
      <Modal
        isOpen={isBulkModalOpen}
        onClose={() => setIsBulkModalOpen(false)}
        title={t("patterns.bulk_assign_title")}
        subtitle={t("patterns.bulk_assign_subtitle")}
        maxWidth="2xl"
      >
        <div className="space-y-6 pt-4">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("patterns.form.select_pattern")}</label>
              <select
                value={selectedPatternId}
                onChange={(e) => setSelectedPatternId(e.target.value)}
                className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
              >
                <option value="">{t("patterns.form.select_pattern_placeholder")}</option>
                {patterns?.map(p => (
                  <option key={p.id} value={p.id}>{p.name}</option>
                ))}
              </select>
            </div>
            
            <div className="space-y-2">
               <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("patterns.form.start_date")}</label>
               <input
                type="date"
                value={assignStartDate}
                onChange={(e) => setAssignStartDate(e.target.value)}
                className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>
          </div>

          <div className="space-y-2">
            <div className="flex items-center justify-between mb-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("patterns.form.select_employees")}</label>
              <div className="flex gap-2">
                <button 
                  onClick={async () => {
                    const allEmps = await fetchEmployees(wsId);
                    setSelectedEmployeeIds(allEmps.map(e => e.id));
                  }}
                  className="text-[10px] font-bold text-primary-600 hover:underline"
                >
                  {t("common.select_all")}
                </button>
                <button 
                  onClick={() => setSelectedEmployeeIds([])}
                  className="text-[10px] font-bold text-slate-400 hover:underline"
                >
                  {t("common.deselect_all")}
                </button>
              </div>
            </div>
            
            <EmployeeSearch
              workspaceId={wsId}
              multiple={true}
              value={selectedEmployeeIds}
              onChange={setSelectedEmployeeIds}
              placeholder={t("patterns.form.select_employees_placeholder", "Search employees...")}
            />
          </div>

          <div className="flex gap-3 pt-2">
            <button
              onClick={() => setIsBulkModalOpen(false)}
              className="flex-1 px-6 py-4 rounded-2xl font-black text-xs uppercase tracking-widest text-slate-500 hover:bg-slate-100 transition-all"
            >
              {t("common.cancel")}
            </button>
            <button
              onClick={handleBulkAssign}
              disabled={bulkAssignMutation.isPending || selectedEmployeeIds.length === 0 || !selectedPatternId}
              className="flex-1 bg-primary-600 hover:bg-primary-700 text-white font-black px-6 py-4 rounded-2xl flex items-center justify-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 disabled:opacity-50 text-xs uppercase tracking-widest"
            >
              <Check className="w-4 h-4" /> {t("patterns.bulk_assign_submit")}
            </button>
          </div>
        </div>
      </Modal>
    </div>
  );
}
