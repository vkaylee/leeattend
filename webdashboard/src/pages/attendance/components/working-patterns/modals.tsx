import {
  CalendarDays,
  Check,
  Clock,
  Info,
  Plus,
  Users,
  X,
  RefreshCw,
} from "lucide-react";
import React from "react";
import { useTranslation } from "react-i18next";

import { EmployeeSearch } from "@/components/common/employee-search";
import Button from "@/components/ui/button";
import { Combobox } from "@/components/ui/combobox";
import FormField, { INPUT_BASE, LABEL_BASE } from "@/components/ui/form-field";
import Modal from "@/components/ui/modal";
import { Shift } from "@/lib/api/types";
import { cn } from "@/lib/utils";

import { useWorkingPatterns } from "../../hooks/use-working-patterns";

interface PatternModalsProps {
  hook: ReturnType<typeof useWorkingPatterns>;
}

export function PatternModals({ hook }: PatternModalsProps) {
  return (
    <>
      <CreatePatternModal hook={hook} />
      <BulkAssignModal hook={hook} />
    </>
  );
}

function CreatePatternModal({ hook }: PatternModalsProps) {
  const { t } = useTranslation();
  const {
    addShiftToDay,
    handleCreate,
    isCreateModalOpen,
    newPatternDesc,
    newPatternName,
    prefillStandard,
    removeShiftFromDay,
    setIsCreateModalOpen,
    setNewPatternDesc,
    setNewPatternName,
    shifts,
    weeklyGrid,
    patternType,
    setPatternType,
    cycleLengthDays,
    setCycleLengthDays,
  } = hook;

  const daysOfWeekFull = [
    t("days.sunday"),
    t("days.monday"),
    t("days.tuesday"),
    t("days.wednesday"),
    t("days.thursday"),
    t("days.friday"),
    t("days.saturday"),
  ];

  const gridRows = patternType === "weekly"
    ? daysOfWeekFull
    : Array.from({ length: cycleLengthDays }, (_, i) => t("patterns.form.day_index", { index: i + 1 }));

  return (
    <Modal icon={<Plus size={24} />} isOpen={isCreateModalOpen} maxWidth="xl" onClose={() => { setIsCreateModalOpen(false); }} subtitle={t("patterns.modal.subtitle")} title={t("patterns.modal.create_title")}>
      <div className="space-y-10 pt-4 animate-in fade-in duration-500">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          <FormField inputClassName="font-bold tracking-tight" label={t("patterns.form.name_label")} onChange={(e) => { setNewPatternName(e.target.value); }} placeholder={t("patterns.form.name_placeholder")} required value={newPatternName} />
          <FormField label={t("patterns.form.desc_label")} onChange={(e) => { setNewPatternDesc(e.target.value); }} placeholder={t("patterns.form.desc_placeholder")} value={newPatternDesc} />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 items-start">
          <div className="space-y-4">
            <label className={LABEL_BASE}>{t("patterns.form.pattern_type")?.toUpperCase() || "PATTERN TYPE"}</label>
            <div className="flex bg-slate-50 dark:bg-slate-800 p-1.5 rounded-2xl border border-slate-100 dark:border-slate-700/50">
              <button
                type="button"
                onClick={() => setPatternType("weekly")}
                className={cn(
                  "flex-1 flex items-center justify-center gap-2.5 py-3 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all",
                  patternType === "weekly"
                    ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-md border border-slate-100 dark:border-slate-850"
                    : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-350"
                )}
              >
                <CalendarDays size={14} />
                {t("patterns.form.pattern_type_weekly") || "Weekly"}
              </button>
              <button
                type="button"
                onClick={() => setPatternType("rotation")}
                className={cn(
                  "flex-1 flex items-center justify-center gap-2.5 py-3 rounded-xl text-[10px] font-bold uppercase tracking-wider transition-all",
                  patternType === "rotation"
                    ? "bg-white dark:bg-slate-900 text-primary-600 dark:text-primary-400 shadow-md border border-slate-100 dark:border-slate-850"
                    : "text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-350"
                )}
              >
                <RefreshCw size={14} />
                {t("patterns.form.pattern_type_rotation") || "Rotation"}
              </button>
            </div>
          </div>

          {patternType === "rotation" && (
            <div className="space-y-4 animate-in fade-in slide-in-from-top-2 duration-305">
              <FormField
                label={t("patterns.form.cycle_length") || "Cycle Length (Days)"}
                type="number"
                required
                value={String(cycleLengthDays)}
                onChange={(e) => setCycleLengthDays(Math.max(1, Math.min(366, parseInt(e.target.value) || 1)))}
                placeholder={t("patterns.form.cycle_length_placeholder") || "e.g. 6"}
                inputClassName="font-bold tracking-tight"
              />
            </div>
          )}
        </div>

        <div className="space-y-6">
          <div className="flex items-center justify-between px-1">
            <h4 className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-[0.3em] flex items-center gap-3">
              <Clock className="text-primary-500" size={16} /> 
              {(patternType === "weekly" 
                ? t("patterns.form.weekly_grid_title") 
                : t("patterns.form.cycle_grid_title") || "Cycle Schedule Grid").toUpperCase()}
            </h4>
            {patternType === "weekly" && (
              <Button className="text-[9px] text-primary-600 dark:text-primary-400 bg-primary-50 dark:bg-primary-950 border-primary-100 dark:border-primary-800 hover:bg-primary-100 dark:hover:bg-primary-900" onClick={() => { prefillStandard(); }} size="sm" variant="outline">{t("patterns.form.prefill_standard").toUpperCase()}</Button>
            )}
          </div>
          <div className="flex flex-col gap-3 max-h-96 overflow-y-auto pr-1">
            {gridRows.map((day, index) => (
              <DayRow addShiftToDay={addShiftToDay} day={day} dayShifts={weeklyGrid[index] || []} index={index} key={index} removeShiftFromDay={removeShiftFromDay} shifts={shifts} />
            ))}
          </div>
        </div>
        <div className="pt-8 border-t border-slate-50 dark:border-slate-800 grid grid-cols-2 gap-6">
          <Button onClick={() => { setIsCreateModalOpen(false); }} size="lg" variant="outline">{t("common.cancel").toUpperCase()}</Button>
          <Button onClick={() => { handleCreate(); }} size="lg" variant="secondary">{t("patterns.modal.submit_btn").toUpperCase()}</Button>
        </div>
      </div>
    </Modal>
  );
}

interface DayRowProps {
  addShiftToDay: (d: number, s: string) => void;
  day: string;
  dayShifts: string[];
  index: number;
  removeShiftFromDay: (d: number, s: string) => void;
  shifts: Shift[];
}

function DayRow({ day, dayShifts, index, removeShiftFromDay, shifts, addShiftToDay }: DayRowProps) {
  const { t } = useTranslation();
  return (
    <div className={cn("flex items-center gap-4 px-5 py-3 rounded-2xl border transition-all", dayShifts.length > 0 ? "bg-white dark:bg-slate-900 border-primary-100 dark:border-primary-800/50 shadow-sm" : "bg-slate-50/50 dark:bg-slate-800/30 border-slate-100 dark:border-slate-800")}>
      <div className="w-24 flex-shrink-0"><span className={cn("text-[10px] font-bold uppercase tracking-widest", dayShifts.length > 0 ? "text-slate-700 dark:text-slate-200" : "text-slate-400 dark:text-slate-500")}>{day}</span></div>
      <div className="flex-1 flex items-center gap-2 flex-wrap min-h-[2rem]">
        {dayShifts.map((shiftId) => {
          const shiftName = shifts.find((s) => s.id === shiftId)?.name ?? shiftId;
          return (
            <div className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-50 dark:bg-primary-950 border border-primary-200 dark:border-primary-800 rounded-xl shadow-sm animate-in zoom-in-95 duration-300" key={shiftId}>
              <Clock className="text-primary-500 dark:text-primary-400 flex-shrink-0" size={10} />
              <span className="text-[10px] font-bold text-primary-700 dark:text-primary-300 uppercase tracking-tight">{shiftName.toUpperCase()}</span>
              <button aria-label={`Remove ${shiftName}`} className="p-0.5 rounded-md text-primary-400 hover:text-rose-500 hover:bg-rose-50 dark:hover:bg-rose-950/50 transition-colors flex-shrink-0 ml-0.5" onClick={() => { removeShiftFromDay(index, shiftId); }} type="button"><X size={12} /></button>
            </div>
          );
        })}
        {dayShifts.length === 0 && <span className="text-[10px] font-bold text-slate-300 dark:text-slate-600 uppercase tracking-widest italic">{t("status_off")}</span>}
      </div>
      <Combobox className="w-36 flex-shrink-0 !py-1 !px-2 rounded-xl text-[9px] font-bold uppercase tracking-tight outline-none border border-dashed border-primary-200 dark:border-primary-800 bg-transparent text-primary-500 dark:text-primary-400 cursor-pointer transition-all hover:bg-primary-50 dark:hover:bg-primary-950/50" onChange={(v) => { if (v) addShiftToDay(index, v); }} options={shifts.filter((s) => !dayShifts.includes(s.id)).map((s) => ({ label: s.name.toUpperCase(), value: s.id }))} placeholder={`+ ${t("patterns.form.add_shift") || "ADD"}`} value="" />
    </div>
  );
}

function BulkAssignModal({ hook }: PatternModalsProps) {
  const { t } = useTranslation();
  const {
    assignStartDate,
    handleBulkAssign,
    isBulkModalOpen,
    patterns,
    selectedEmployeeIds,
    selectedPatternId,
    setAssignStartDate,
    setIsBulkModalOpen,
    setSelectedEmployeeIds,
    setSelectedPatternId,
    wsId,
    assignAnchorDate,
    setAssignAnchorDate,
  } = hook;

  const selectedPattern = patterns.find((p) => p.id === selectedPatternId);
  const isRotation = selectedPattern?.pattern_type === "rotation";

  return (
    <Modal icon={<Users size={24} />} isOpen={isBulkModalOpen} maxWidth="2xl" onClose={() => { setIsBulkModalOpen(false); }} subtitle={t("patterns.modal.bulk_subtitle")} title={t("patterns.bulk_assign_btn")}>
      <div className="space-y-10 pt-4 animate-in fade-in duration-500">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-10 px-1">
          <div className="space-y-4">
            <label className={LABEL_BASE}>{t("patterns.form.select_pattern_label").toUpperCase()}</label>
            <div className="grid grid-cols-1 gap-3 max-h-80 overflow-y-auto pr-2 scrollbar-hide">
              {patterns.map((p) => (
                <button className={cn("flex items-center justify-between p-5 rounded-2xl border transition-all text-left group", selectedPatternId === p.id ? "bg-white dark:bg-slate-900 border-primary-600 shadow-xl shadow-primary-100 dark:shadow-black/30 ring-2 ring-primary-600/10" : "bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 hover:bg-white dark:hover:bg-slate-900 hover:border-primary-200 dark:border-primary-800")} key={p.id} onClick={() => { setSelectedPatternId(p.id); }} type="button">
                  <div className="flex items-center gap-4">
                    <div className={cn("w-10 h-10 rounded-xl flex items-center justify-center transition-colors", selectedPatternId === p.id ? "bg-primary-600 text-white" : "bg-white dark:bg-slate-900 text-slate-300 dark:text-slate-600 border border-slate-100 dark:border-slate-800 group-hover:text-primary-500 dark:group-hover:text-primary-400")}><CalendarDays size={20} /></div>
                    <div>
                      <span className={cn("font-bold text-xs uppercase tracking-tight block", selectedPatternId === p.id ? "text-slate-700 dark:text-slate-200" : "text-slate-500 dark:text-slate-400")}>{p.name}</span>
                      <span className="text-[9px] text-slate-400 dark:text-slate-500 font-medium">
                        {p.pattern_type === "rotation" 
                          ? `${t("patterns.form.pattern_type_rotation") || "Rotation"} (${p.cycle_length_days}d)` 
                          : t("patterns.form.pattern_type_weekly") || "Weekly"}
                      </span>
                    </div>
                  </div>
                  {selectedPatternId === p.id && <Check className="text-primary-600 dark:text-primary-400" size={18} />}
                </button>
              ))}
            </div>
          </div>
          <div className="space-y-6">
            <div className="space-y-4">
              <label className={LABEL_BASE}>{t("patterns.form.start_date_label").toUpperCase()}</label>
              <div className="relative group">
                <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors"><Info size={18} /></div>
                <input className={cn(INPUT_BASE, "pl-14 font-bold tracking-tight shadow-inner")} onChange={(e) => { setAssignStartDate(e.target.value); }} type="date" value={assignStartDate} />
              </div>
              <p className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase italic leading-relaxed px-1 opacity-60">{t("patterns.form.start_date_note")}</p>
            </div>

            {isRotation && (
              <div className="space-y-4 animate-in fade-in slide-in-from-top-2 duration-300">
                <label className={LABEL_BASE}>{t("patterns.form.anchor_date_label")?.toUpperCase() || "ANCHOR DATE"}</label>
                <div className="relative group">
                  <div className="absolute inset-y-0 left-0 pl-5 flex items-center pointer-events-none text-slate-300 dark:text-slate-600 group-focus-within:text-primary-500 dark:group-focus-within:text-primary-400 transition-colors"><CalendarDays size={18} /></div>
                  <input className={cn(INPUT_BASE, "pl-14 font-bold tracking-tight shadow-inner")} onChange={(e) => { setAssignAnchorDate(e.target.value); }} type="date" value={assignAnchorDate} />
                </div>
                <p className="text-[9px] text-slate-500 dark:text-slate-400 font-bold uppercase italic leading-relaxed px-1 opacity-60">
                  {t("patterns.form.anchor_date_note") || "Anchor Date determines Day 1 of the rotating cycle."}
                </p>
              </div>
            )}
          </div>
        </div>
        <div className="space-y-4 px-1">
          <label className={LABEL_BASE}>{t("patterns.form.select_employees_label").toUpperCase()}</label>
          <EmployeeSearch className="bg-slate-50 dark:bg-slate-800 border-slate-100 dark:border-slate-800 rounded-[2rem] p-8 shadow-inner" multiple={true} onChange={(v) => { setSelectedEmployeeIds(v as string[]); }} value={selectedEmployeeIds} workspaceId={wsId} />
        </div>
        <div className="pt-8 border-t border-slate-50 dark:border-slate-800 grid grid-cols-2 gap-6">
          <Button onClick={() => { setIsBulkModalOpen(false); }} size="lg" variant="outline">{t("common.cancel").toUpperCase()}</Button>
          <Button disabled={!selectedPatternId || selectedEmployeeIds.length === 0} onClick={() => { handleBulkAssign(); }} size="lg" variant="secondary">{t("patterns.modal.assign_btn").toUpperCase()}</Button>
        </div>
      </div>
    </Modal>
  );
}
