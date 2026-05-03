import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Plus,
  Trash2,
  CalendarDays,
  Clock,
  Info,
  Check,
  Users,
  ChevronDown,
  ChevronUp,
  X
} from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format, startOfToday } from "date-fns";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import EmptyState from "@/components/ui/empty-state";
import FormField, { LABEL_BASE, INPUT_BASE } from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import PageHeader from "@/components/ui/page-header";
import StatusBadge from "@/components/ui/status-badge";
import { useToast } from "@/components/ui/toast";
import { useConfirm } from "@/hooks/use-confirm";
import { cn } from "@/lib/utils";

import {
  fetchWorkingPatterns,
  createWorkingPattern,
  fetchPatternDetails,
  fetchShifts,
  bulkAssignPattern,
  deleteWorkingPattern,
  getCurrentWorkspaceId,
  WorkingPattern
} from "../lib/api";

export default function WorkingPatternsPage() {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() ?? "";
  const queryClient = useQueryClient();
  const toast = useToast();
  const { confirm, ConfirmationDialog } = useConfirm();


  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isBulkModalOpen, setIsBulkModalOpen] = useState(false);

  const [newPatternName, setNewPatternName] = useState("");
  const [newPatternDesc, setNewPatternDesc] = useState("");

  // Bulk Assign State
  const [selectedPatternId, setSelectedPatternId] = useState<string>("");
  const [selectedEmployeeIds, setSelectedEmployeeIds] = useState<string[]>([]);
  const [assignStartDate, setAssignStartDate] = useState(format(startOfToday(), "yyyy-MM-01"));

  // Weekly grid state: 0=Sun, 1=Mon, ..., 6=Sat. Value = array of shift_ids (supports split shifts)
  const [weeklyGrid, setWeeklyGrid] = useState<string[][]>(Array.from({ length: 7 }, () => []));

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
    mutationFn: (data: Parameters<typeof createWorkingPattern>[1]) => createWorkingPattern(wsId, data),
    onSuccess: () => {
      toast.success(t("patterns.messages.create_success"));
      setIsCreateModalOpen(false);
      setNewPatternName("");
      setNewPatternDesc("");
      setWeeklyGrid(Array.from({ length: 7 }, () => []));
 
      void queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
    },
  });

  const bulkAssignMutation = useMutation({
    mutationFn: (data: Parameters<typeof bulkAssignPattern>[1]) => bulkAssignPattern(wsId, data),
    onSuccess: (data) => {
      if (data.success_count > 0) {
        toast.success(t("patterns.messages.bulk_assign_success_count", { count: data.success_count }));
        setIsBulkModalOpen(false);
        setSelectedEmployeeIds([]);
        void queryClient.invalidateQueries({ queryKey: ["schedules"] });
 
      } else if (data.failed_employees.length > 0) {
        toast.error(t("patterns.messages.bulk_assign_conflict", { count: data.failed_employees.length }));
      }
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (patternId: string) => deleteWorkingPattern(wsId, patternId),
    onSuccess: () => {
      toast.success(t("patterns.messages.delete_success"));
      void queryClient.invalidateQueries({ queryKey: ["working-patterns"] });
    },
  });

  const handleDelete = async (pattern: WorkingPattern) => {
    const ok = await confirm({
      title: t("common.warning", { defaultValue: "Warning" }),
      message: t("patterns.confirm_delete", { name: pattern.name }),
      variant: "danger"
    });
    if (ok) deleteMutation.mutate(pattern.id);
  };

  const addShiftToDay = (dayIndex: number, shiftId: string) => {
    setWeeklyGrid(previous => {
      const newGrid = previous.map(array => [...array]);
      if (!newGrid[dayIndex].includes(shiftId)) {
        newGrid[dayIndex] = [...newGrid[dayIndex], shiftId];
      }
      return newGrid;
    });
  };

  const removeShiftFromDay = (dayIndex: number, shiftId: string) => {
    setWeeklyGrid(previous => {
      const newGrid = previous.map(array => [...array]);
      newGrid[dayIndex] = newGrid[dayIndex].filter(id => id !== shiftId);
      return newGrid;
    });
  };

  const handleCreate = () => {
    if (!newPatternName) {
      toast.warning(t("patterns.validation_name_required"));
      return;
    }
    const details = weeklyGrid.flatMap((shiftIds, index) =>
      shiftIds.length > 0
        ? shiftIds.map(sid => ({ day_of_week: index, shift_id: sid }))
        : [{ day_of_week: index, shift_id: null as string | null }]
    );
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
    const newGrid = weeklyGrid.map(array => [...array]);
    for (let index = 1; index <= 5; index++) newGrid[index] = [defaultShift];
    setWeeklyGrid(newGrid);
  };

  if (!wsId) return <div className="p-12 text-center text-slate-500 dark:text-slate-400 font-bold italic uppercase tracking-widest text-xs opacity-50">{t("common.select_workspace")}</div>;

  return (
    <div className="space-y-10 animate-in fade-in duration-700 pb-20">
      <PageHeader
        actions={
          <>
            <Button icon={<Users size={18} />} onClick={() => { setIsBulkModalOpen(true); }} variant="outline">{t("patterns.bulk_assign_btn").toUpperCase()}</Button>
            <Button icon={<Plus size={18} />} onClick={() => { setIsCreateModalOpen(true); setWeeklyGrid(Array.from({ length: 7 }, () => [])); }}>{t("patterns.create_btn").toUpperCase()}</Button>
          </>
        }
        data-testid="patterns-title"
        icon={<CalendarDays size={24} />}
        subtitle={t("patterns.subtitle")}
        title={t("patterns.title")}
      />

      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-8">
        {isLoading ? (
          Array.from({ length: 3 }).map((_, index) => (
            <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 p-10 rounded-[2.5rem] shadow-sm animate-pulse h-80" key={index} />
          ))
        ) : patterns?.length === 0 ? (
          <EmptyState
            description={t("patterns.empty_desc")}
            icon={<CalendarDays size={40} />}
            title={t("patterns.empty_title")}
          />
        ) : patterns?.map(pattern => (
          <div className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] p-10 shadow-sm hover:shadow-2xl hover:shadow-primary-100/10 hover:scale-[1.02] transition-all group relative overflow-hidden flex flex-col" key={pattern.id}>
            <div className="absolute top-0 right-0 p-8 flex gap-3 translate-y-4 opacity-0 group-hover:translate-y-0 group-hover:opacity-100 transition-all duration-500">
              <button className="p-3 bg-white dark:bg-slate-900 text-slate-500 dark:text-slate-400 hover:text-rose-600 dark:text-rose-300 rounded-xl shadow-md border border-slate-100 dark:border-slate-800 transition-all active:scale-90" onClick={() => handleDelete(pattern)} title={t("common.delete")}><Trash2 size={18} /></button>
            </div>

            <div className="flex items-center gap-5 mb-8">
              <div className="w-16 h-16 bg-primary-50 dark:bg-primary-950 text-primary-600 dark:text-primary-400 rounded-[1.5rem] flex items-center justify-center shadow-inner border border-primary-100 dark:border-primary-800/50 group-hover:scale-110 group-hover:rotate-3 transition-all duration-500"><CalendarDays size={32} /></div>
              <div>
                <h3 className="font-bold text-2xl text-slate-700 dark:text-slate-200 leading-none uppercase tracking-tight mb-2">{pattern.name}</h3>
                <StatusBadge variant={pattern.is_active ? "success" : "neutral"}>{pattern.is_active ? t("status_active_caps") : t("status_off")}</StatusBadge>
              </div>
            </div>

            <p className="text-slate-500 dark:text-slate-400 text-xs mb-10 line-clamp-2 min-h-[3rem] font-bold uppercase tracking-tight opacity-60 leading-relaxed italic">{pattern.description ?? t("common.no_description")}</p>

            <button
              className="mt-auto flex items-center justify-between w-full p-5 bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl hover:bg-white dark:hover:bg-slate-900 hover:border-primary-100 dark:border-primary-800 transition-all group/btn"
              onClick={() => { setExpandedPatternId(expandedPatternId === pattern.id ? null : pattern.id); }}
            >
              <span className="text-[10px] font-bold text-slate-500 dark:text-slate-400 group-hover/btn:text-primary-600 dark:text-primary-400 uppercase tracking-widest transition-colors">{t("patterns.view_details").toUpperCase()}</span>
              {expandedPatternId === pattern.id ? <ChevronUp className="text-slate-300 dark:text-slate-600" size={16} /> : <ChevronDown className="text-slate-300 dark:text-slate-600" size={16} />}
            </button>

            {expandedPatternId === pattern.id && (
              <div className="mt-6 pt-6 border-t border-slate-100 dark:border-slate-800 animate-in slide-in-from-top-4 duration-500">
                <div className="flex flex-col gap-2">
                  {daysOfWeekShort.map((day, index) => {
                    const dayDetails = patternDetails?.filter(d => d.day_of_week === index) ?? [];
                    const hasShifts = dayDetails.some(d => d.shift_id);
                    return (
                      <div className={cn(
                        "flex items-center gap-3 px-4 py-2 rounded-xl border transition-all",
                        hasShifts
                          ? "bg-white dark:bg-slate-800/50 border-primary-100 dark:border-primary-800/50"
                          : "bg-slate-50/50 dark:bg-slate-800/20 border-slate-100 dark:border-slate-800 opacity-50"
                      )} key={index}>
                        <span className={cn(
                          "w-10 flex-shrink-0 text-[9px] font-bold uppercase tracking-widest",
                          hasShifts ? "text-slate-700 dark:text-slate-200" : "text-slate-400 dark:text-slate-500"
                        )}>{day}</span>
                        <div className="flex-1 flex items-center gap-1.5 flex-wrap">
                          {hasShifts ? (
                            dayDetails.filter(d => d.shift_id).map((d, index) => {
                              const shiftName = shifts?.find(s => s.id === d.shift_id)?.name;
                              return (
                                <div className="flex items-center gap-1 px-2 py-1 rounded-lg bg-primary-50 dark:bg-primary-950 border border-primary-100 dark:border-primary-800" key={index}>
                                  <Clock className="text-primary-500 dark:text-primary-400" size={8} />
                                  <span className="text-[8px] font-bold text-primary-600 dark:text-primary-400 uppercase tracking-tight">{shiftName ?? '—'}</span>
                                </div>
                              );
                            })
                          ) : (
                            <span className="text-[9px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest italic">{t("status_off")}</span>
                          )}
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
      <Modal icon={<Plus size={24} />} isOpen={isCreateModalOpen} maxWidth="xl" onClose={() => { setIsCreateModalOpen(false); }} subtitle={t("patterns.modal.subtitle")} title={t("patterns.modal.create_title")}>
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
            <FormField inputClassName="font-bold tracking-tight" label={t("patterns.form.name_label")} onChange={e => { setNewPatternName(e.target.value); }} placeholder={t("patterns.form.name_placeholder")} required value={newPatternName} />
            <FormField label={t("patterns.form.desc_label")} onChange={e => { setNewPatternDesc(e.target.value); }} placeholder={t("patterns.form.desc_placeholder")} value={newPatternDesc} />
          </div>

          <div className="space-y-6">
            <div className="flex items-center justify-between px-1">
              <h4 className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] flex items-center gap-3"><Clock className="text-primary-500" size={16} /> {t("patterns.form.weekly_grid_title").toUpperCase()}</h4>
              <Button className="text-[9px] text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 border-primary-100 dark:border-primary-800 hover:bg-primary-100 dark:hover:bg-primary-900" onClick={prefillStandard} size="sm" variant="outline">
                {t("patterns.form.prefill_standard").toUpperCase()}
              </Button>
            </div>

            <div className="flex flex-col gap-3">
              {daysOfWeekFull.map((day, index) => {
                const dayShifts = weeklyGrid[index] || [];
                return (
                  <div className={cn(
                    "flex items-center gap-4 px-5 py-3 rounded-2xl border transition-all",
                    dayShifts.length > 0
                      ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800/50 shadow-sm"
                      : "bg-slate-50/50 dark:bg-slate-800/30 border-slate-100 dark:border-slate-800"
                  )} key={index}>
                    {/* Day label */}
                    <div className="w-20 flex-shrink-0">
                      <span className={cn(
                        "text-[10px] font-bold uppercase tracking-widest",
                        dayShifts.length > 0 ? "text-slate-700 dark:text-slate-200" : "text-slate-400 dark:text-slate-500"
                      )}>{day}</span>
                    </div>

                    {/* Shift chips */}
                    <div className="flex-1 flex items-center gap-2 flex-wrap min-h-[2rem]">
                      {dayShifts.map(shiftId => {
                        const shiftName = shifts?.find(s => s.id === shiftId)?.name ?? shiftId;
                        return (
                          <div className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-50 dark:bg-primary-950 border border-primary-200 dark:border-primary-800 rounded-xl shadow-sm animate-in zoom-in-95 duration-300" key={shiftId}>
                            <Clock className="text-primary-500 dark:text-primary-400 flex-shrink-0" size={10} />
                            <span className="text-[10px] font-bold text-primary-700 dark:text-primary-300 uppercase tracking-tight">{shiftName.toUpperCase()}</span>
                            <button
                              aria-label={`Remove ${shiftName ?? ""}`}
                              className="p-0.5 rounded-md text-primary-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-950/50 transition-colors flex-shrink-0 ml-0.5"
                              onClick={() => { removeShiftFromDay(index, shiftId); }}
                              type="button"
                            >
                              <X size={12} />
                            </button>
                          </div>
                        );
                      })}
                      {dayShifts.length === 0 && (
                        <span className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest italic">{t("status_off")}</span>
                      )}
                    </div>

                    {/* Add shift button */}
                    <Combobox
                      className="w-36 flex-shrink-0 !py-1 !px-2 rounded-xl text-[9px] font-bold uppercase tracking-tight outline-none border border-dashed border-primary-200 dark:border-primary-800 bg-transparent text-primary-500 dark:text-primary-400 cursor-pointer transition-all hover:bg-primary-50 dark:hover:bg-primary-950/50"
                      onChange={value => { if (value) addShiftToDay(index, value); }}
                      options={shifts?.filter(s => !dayShifts.includes(s.id)).map(s => ({ value: s.id, label: s.name.toUpperCase() })) ?? []}
                      placeholder={`+ ${t("patterns.form.add_shift") || "ADD"}`}
                      value=""
                    />
                  </div>
                );
              })}
            </div>
          </div>

          <div className="pt-8 border-t border-slate-50 dark:border-slate-800 grid grid-cols-2 gap-6">
            <Button onClick={() => { setIsCreateModalOpen(false); }} size="lg" variant="outline">{t("common.cancel").toUpperCase()}</Button>
            <Button onClick={handleCreate} size="lg" variant="secondary">{t("patterns.modal.submit_btn").toUpperCase()}</Button>
          </div>
        </div>
      </Modal>

      {/* Bulk Assign Modal */}
      <Modal icon={<Users size={24} />} isOpen={isBulkModalOpen} maxWidth="2xl" onClose={() => { setIsBulkModalOpen(false); }} subtitle={t("patterns.modal.bulk_subtitle")} title={t("patterns.bulk_assign_btn")}>
        <div className="space-y-10 pt-4 animate-in fade-in duration-500">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-10 px-1">
            <div className="space-y-4">
              <label className={LABEL_BASE}>{t("patterns.form.select_pattern_label").toUpperCase()}</label>
              <div className="grid grid-cols-1 gap-3 max-h-64 overflow-y-auto pr-2 scrollbar-hide">
                {patterns?.map(p => (
                  <button
                    className={cn("flex items-center justify-between p-5 rounded-2xl border transition-all text-left group",
                      selectedPatternId === p.id ? "bg-white dark:bg-slate-900 border-primary-600 shadow-xl shadow-primary-100 dark:shadow-black/30/20 ring-2 ring-primary-600/10" : "bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900 hover:border-primary-200 dark:border-primary-800"
                    )}
                    key={p.id}
                    onClick={() => { setSelectedPatternId(p.id); }}
                  >
                    <div className="flex items-center gap-4">
                      <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center transition-colors",
                        selectedPatternId === p.id ? "bg-primary-600 text-white" : "bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 border border-slate-100 dark:border-slate-800 group-hover:text-primary-500 dark:group-hover:text-primary-400"
                      )}><CalendarDays size={20} /></div>
                      <span className={cn("font-bold text-xs uppercase tracking-tight", selectedPatternId === p.id ? "text-slate-700 dark:text-slate-200" : "text-slate-500 dark:text-slate-400")}>{p.name}</span>
                    </div>
                    {selectedPatternId === p.id && <Check className="text-primary-600 dark:text-primary-400" size={18} />}
                  </button>
                ))}
              </div>
            </div>

            <div className="space-y-4">
              <label className={LABEL_BASE}>{t("patterns.form.start_date_label").toUpperCase()}</label>
              <div className="relative group">
                <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors"><Info size={18} /></div>
                <input className={cn(INPUT_BASE, "pl-14 font-bold tracking-tight shadow-inner")} onChange={e => { setAssignStartDate(e.target.value); }} type="date" value={assignStartDate} />
              </div>
              <p className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase italic leading-relaxed px-1 opacity-60">{t("patterns.form.start_date_note")}</p>
            </div>
          </div>

          <div className="space-y-4 px-1">
            <label className={LABEL_BASE}>{t("patterns.form.select_employees_label").toUpperCase()}</label>
            <EmployeeSearch
              className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 rounded-[2rem] p-8 shadow-inner"
              multiple={true}
              onChange={(value) => { setSelectedEmployeeIds(value as string[]); }}
              value={selectedEmployeeIds}
              workspaceId={wsId}
            />
          </div>

          <div className="pt-8 border-t border-slate-50 dark:border-slate-800 grid grid-cols-2 gap-6">
            <Button onClick={() => { setIsBulkModalOpen(false); }} size="lg" variant="outline">{t("common.cancel").toUpperCase()}</Button>
            <Button disabled={!selectedPatternId || selectedEmployeeIds.length === 0} onClick={handleBulkAssign} size="lg" variant="secondary">
              {t("patterns.modal.assign_btn").toUpperCase()}
            </Button>
          </div>
        </div>
      </Modal>
      <ConfirmationDialog />
    </div>
  );
}