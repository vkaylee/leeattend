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
  Check
} from "lucide-react";
import { 
  fetchWorkingPatterns, 
  createWorkingPattern, 
  fetchPatternDetails,
  fetchShifts,
  getCurrentWorkspaceId,
  WorkingPattern,
  Shift
} from "../lib/api";
import { useToast } from "../components/Toast";
import Modal from "@/components/ui/Modal";
import { format } from "date-fns";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { Link } from "react-router-dom";

export default function WorkingPatternsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const { formatTime } = useDateTimeFormatter();

  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [newPatternName, setNewPatternName] = useState("");
  const [newPatternDesc, setNewPatternDesc] = useState("");
  
  // Weekly grid state: 0=Sun, 1=Mon, ..., 6=Sat. Value = shift_id or null (off)
  const [weeklyGrid, setWeeklyGrid] = useState<(string | null)[]>(Array(7).fill(null));

  const { data: patterns } = useQuery({
    queryKey: ["working-patterns", wsId],
    queryFn: () => fetchWorkingPatterns(wsId),
    enabled: !!wsId,
  });

  const { data: shifts } = useQuery({
    queryKey: ["shifts", wsId],
    queryFn: () => fetchShifts(wsId),
    enabled: !!wsId,
  });

  // Fetch details for expanded view (optional, or just show summary)
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

  const handleCreate = () => {
    if (!newPatternName) {
      toast.warning(t("patterns.validation_name_required"));
      return;
    }

    const details = weeklyGrid.map((shiftId, idx) => ({
      day_of_week: idx,
      shift_id: shiftId
    }));

    createMutation.mutate({
      name: newPatternName,
      description: newPatternDesc,
      details
    });
  };

  const daysOfWeek = [
    t("days.sunday"),
    t("days.monday"),
    t("days.tuesday"),
    t("days.wednesday"),
    t("days.thursday"),
    t("days.friday"),
    t("days.saturday")
  ];

  // Helper to pre-fill grid for "Office Standard" (Mon-Fri)
  const prefillStandard = () => {
    if (!shifts || shifts.length === 0) return;
    const defaultShift = shifts[0].id; // Pick first shift as default
    const newGrid = [...weeklyGrid];
    // Set Mon(1) to Fri(5)
    for (let i = 1; i <= 5; i++) newGrid[i] = defaultShift;
    // Sat(6), Sun(0) remain null
    setWeeklyGrid(newGrid);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col md:flex-row justify-between items-start md:items-center gap-4">
        <div>
          <div className="flex items-center gap-2 mb-2">
            <Link to="/shifts" className="text-slate-400 hover:text-primary-600 transition-colors">
              <ArrowLeft size={20} />
            </Link>
            <h1 className="text-3xl font-bold text-slate-900 tracking-tight">{t("patterns.title")}</h1>
          </div>
          <p className="text-slate-500">{t("patterns.subtitle")}</p>
        </div>
        <button 
          onClick={() => setIsCreateModalOpen(true)}
          className="bg-primary-600 hover:bg-primary-700 text-white font-black px-6 py-3 rounded-2xl flex items-center gap-2 shadow-lg shadow-primary-100 transition-all active:scale-95 text-xs uppercase tracking-widest"
        >
          <Plus size={16} /> {t("patterns.create_btn")}
        </button>
      </div>

      {/* Grid of Patterns */}
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-6">
        {patterns?.map(pattern => (
          <div key={pattern.id} className="bg-white border border-slate-200 rounded-[32px] p-6 shadow-sm hover:shadow-md transition-all group">
            <div className="flex items-start justify-between mb-4">
              <div className="flex items-center gap-3">
                <div className="w-12 h-12 bg-purple-100 text-purple-600 rounded-2xl flex items-center justify-center">
                  <CalendarDays size={24} />
                </div>
                <div>
                  <h3 className="font-bold text-lg text-slate-900 leading-tight">{pattern.name}</h3>
                  <p className="text-xs text-slate-500 mt-0.5 line-clamp-1">{pattern.description || t("common.no_description")}</p>
                </div>
              </div>
              <div className={`px-2 py-1 rounded-lg text-[10px] font-black uppercase tracking-widest ${pattern.is_active ? 'bg-emerald-50 text-emerald-600 border border-emerald-100' : 'bg-slate-50 text-slate-400'}`}>
                {pattern.is_active ? t("common.active") : t("common.inactive")}
              </div>
            </div>

            <div className="space-y-3">
              <div className="flex justify-between items-center text-xs font-bold text-slate-500 uppercase tracking-widest border-b border-slate-50 pb-2">
                <span>{t("patterns.weekly_schedule")}</span>
                <button 
                   onClick={() => setExpandedPatternId(expandedPatternId === pattern.id ? null : pattern.id)}
                   className="text-primary-600 hover:text-primary-700"
                >
                  {expandedPatternId === pattern.id ? t("common.collapse") : t("common.expand")}
                </button>
              </div>
              
              {expandedPatternId === pattern.id && patternDetails ? (
                <div className="space-y-2 animate-in slide-in-from-top-2">
                  {patternDetails.map(detail => {
                    const shift = shifts?.find(s => s.id === detail.shift_id);
                    return (
                      <div key={detail.day_of_week} className="flex items-center justify-between text-xs py-1">
                        <span className="font-bold text-slate-600 w-24">{daysOfWeek[detail.day_of_week]}</span>
                        {shift ? (
                          <div className="flex items-center gap-2 flex-1">
                            <span className="bg-blue-50 text-blue-700 px-2 py-0.5 rounded-md font-bold text-[10px] uppercase truncate max-w-[100px]">{shift.name}</span>
                            <span className="text-[10px] text-slate-400 font-mono">{formatTime(shift.start_time)}-{formatTime(shift.end_time)}</span>
                          </div>
                        ) : (
                          <span className="text-slate-300 font-bold text-[10px] uppercase tracking-widest">{t("shifts.off_day")}</span>
                        )}
                      </div>
                    );
                  })}
                </div>
              ) : (
                 <div className="flex gap-1">
                    {/* Tiny visual preview of the week */}
                    {Array.from({ length: 7 }).map((_, i) => (
                      <div key={i} className="flex-1 h-1.5 rounded-full bg-slate-100 overflow-hidden">
                         {/* We don't have details here unless expanded, so this is just a placeholder visual 
                             In a real app, we might include summary in the main list fetch 
                         */}
                         <div className="w-full h-full bg-slate-200" /> 
                      </div>
                    ))}
                 </div>
              )}
            </div>
            
            <div className="mt-6 pt-4 border-t border-slate-50 flex justify-end gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
               <button className="p-2 hover:bg-rose-50 text-slate-400 hover:text-rose-600 rounded-xl transition-all">
                 <Trash2 size={16} />
               </button>
            </div>
          </div>
        ))}

        {/* Create Card (Empty State) */}
        {(!patterns || patterns.length === 0) && (
          <button 
            onClick={() => setIsCreateModalOpen(true)}
            className="border-2 border-dashed border-slate-200 rounded-[32px] p-8 flex flex-col items-center justify-center text-center gap-4 hover:border-primary-300 hover:bg-primary-50/10 transition-all group min-h-[200px]"
          >
            <div className="w-14 h-14 bg-slate-50 rounded-2xl flex items-center justify-center text-slate-300 group-hover:bg-primary-50 group-hover:text-primary-500 transition-colors">
              <CalendarDays size={28} />
            </div>
            <div>
              <h3 className="font-bold text-lg text-slate-900">{t("patterns.empty_title")}</h3>
              <p className="text-sm text-slate-500">{t("patterns.empty_desc")}</p>
            </div>
          </button>
        )}
      </div>

      {/* Create Modal */}
      <Modal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
        title={t("patterns.create_title")}
        description={t("patterns.create_subtitle")}
        className="max-w-2xl"
      >
        <div className="space-y-6 pt-4">
          <div className="grid grid-cols-1 gap-4">
            <div className="space-y-2">
              <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("patterns.form.name")}</label>
              <input
                type="text"
                value={newPatternName}
                onChange={(e) => setNewPatternName(e.target.value)}
                placeholder="e.g., Office Standard, Night Shift Rotation"
                className="w-full bg-slate-50 border-none rounded-2xl px-5 py-3.5 font-bold text-slate-900 focus:ring-2 focus:ring-primary-500 transition-all"
              />
            </div>
            
            <div className="space-y-2">
               <label className="text-[10px] font-black text-slate-400 uppercase tracking-widest">{t("patterns.form.description")}</label>
               <input
                type="text"
                value={newPatternDesc}
                onChange={(e) => setNewPatternDesc(e.target.value)}
                placeholder="Optional description"
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
    </div>
  );
}
