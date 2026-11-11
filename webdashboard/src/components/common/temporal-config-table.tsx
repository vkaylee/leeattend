import type { TFunction } from "i18next";

import { Plus, Calendar, History, Save, Calculator, AlertCircle, CheckCircle2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useDateTimeFormatter } from "@/hooks/use-datetime";
import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import { ConfigRecord } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface Props {
  isLoading?: boolean;
  onSave: (data: { std_working_days: number; hours_per_day: number; max_ot_hours_per_day: number | null; max_ot_hours_per_month: number | null; effective_date: string }) => Promise<void>;
  records: ConfigRecord[];
  title: string;
}

interface TemporalConfigAddFormProps {
  handleSave: () => Promise<void>;
  newDays: number;
  newEffectiveDate: string;
  newHours: number;
  newMaxOtDay: string;
  newMaxOtMonth: string;
  setIsAdding: (val: boolean) => void;
  setNewDays: (val: number) => void;
  setNewEffectiveDate: (val: string) => void;
  setNewHours: (val: number) => void;
  setNewMaxOtDay: (val: string) => void;
  setNewMaxOtMonth: (val: string) => void;
  t: TFunction;
}

function TemporalConfigAddForm({
  newEffectiveDate,
  setNewEffectiveDate,
  newDays,
  setNewDays,
  newHours,
  setNewHours,
  newMaxOtDay,
  setNewMaxOtDay,
  newMaxOtMonth,
  setNewMaxOtMonth,
  setIsAdding,
  handleSave,
  t
}: TemporalConfigAddFormProps) {
  return (
    <div className="bg-white dark:bg-slate-900 border-2 border-primary-100 dark:border-primary-800 rounded-[2.5rem] p-4 sm:p-8 shadow-xl shadow-primary-50/50 animate-in slide-in-from-top-4 duration-500">
      <div className="flex items-center gap-3 mb-8 text-primary-700 dark:text-primary-300">
        <div className="w-8 h-8 rounded-xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center shadow-inner"><Plus size={16} /></div>
        <Typography className="text-[10px] font-bold uppercase tracking-[0.2em]" variant="h4">{t("payroll.settings.add_version")}</Typography>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-8 mb-8">
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.settings.effective_date")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all" onChange={e => { setNewEffectiveDate(e.target.value); }} type="date" value={newEffectiveDate} />
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.settings.std_days")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono" onChange={e => { setNewDays(parseFloat(e.target.value)); }} type="number" value={newDays} />
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.settings.hours_per_day")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono" onChange={e => { setNewHours(parseFloat(e.target.value)); }} type="number" value={newHours} />
        </div>
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-8 mb-8">
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.settings.max_ot_hours_per_day")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono" onChange={e => { setNewMaxOtDay(e.target.value); }} placeholder={t("payroll.settings.no_limit") || ""} step="0.5" type="number" value={newMaxOtDay} />
          <Typography className="text-[9px] text-slate-400 ml-1" variant="p">{t("payroll.settings.ot_empty_hint")}</Typography>
        </div>
        <div className="space-y-3">
          <label className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest ml-1">{t("payroll.settings.max_ot_hours_per_month")}</label>
          <input className="w-full bg-slate-50 dark:bg-slate-800 border border-slate-100 dark:border-slate-800 rounded-2xl px-5 py-4 text-sm font-bold focus:ring-4 focus:ring-primary-500/10 focus:bg-white dark:focus:bg-slate-900 outline-none transition-all font-mono" onChange={e => { setNewMaxOtMonth(e.target.value); }} placeholder={t("payroll.settings.no_limit") || ""} step="0.5" type="number" value={newMaxOtMonth} />
          <Typography className="text-[9px] text-slate-400 ml-1" variant="p">{t("payroll.settings.ot_empty_hint")}</Typography>
        </div>
      </div>

      <div className="p-6 bg-amber-50 dark:bg-amber-950/50 border border-amber-100 dark:border-amber-800 rounded-[1.5rem] flex gap-4 mb-8 shadow-inner">
        <AlertCircle className="w-6 h-6 text-amber-500 shrink-0" />
        <Typography className="text-[10px] text-amber-800 leading-relaxed font-bold uppercase tracking-tight opacity-80" variant="p">{t("payroll.settings.date_warning")}</Typography>
      </div>

      <div className="flex justify-end gap-4">
        <Button className="px-8 py-4 rounded-2xl font-bold text-[10px] uppercase tracking-widest text-slate-500 dark:text-slate-400 hover:bg-slate-50 dark:hover:bg-slate-800 transition-all" onClick={() => { setIsAdding(false); }} variant="ghost">{t("common.cancel")}</Button>
        <Button className="flex items-center gap-3 bg-primary-600 text-white px-10 py-4 rounded-2xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-xl shadow-primary-100 dark:shadow-black/30 active:scale-95" onClick={handleSave} variant="primary"><Save size={16} /> {t("common.save").toUpperCase()}</Button>
      </div>
    </div>
  );
}

interface TemporalConfigTableContentProps {
  activeIndex: number;
  formatDate: (date: string) => string;
  now: Date;
  sortedRecords: ConfigRecord[];
  t: TFunction;
}

function TemporalConfigTableContent({ sortedRecords, activeIndex, now, t, formatDate }: TemporalConfigTableContentProps) {
  return (
    <table className="w-full min-w-[720px] text-left border-collapse">
      <thead>
        <tr className="bg-slate-50 dark:bg-slate-800/50 border-b border-slate-100 dark:border-slate-800">
          <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("payroll.settings.status")}</th>
          <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest">{t("payroll.settings.effective_date")}</th>
          <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("payroll.settings.std_days")}</th>
          <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("payroll.settings.hours_per_day")}</th>
          <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("payroll.settings.max_ot_day_short")}</th>
          <th className="px-8 py-5 text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest text-right">{t("payroll.settings.max_ot_month_short")}</th>
        </tr>
      </thead>
      <tbody className="divide-y divide-slate-50 dark:divide-slate-800">
        {sortedRecords.map((record, index: number) => (
          <TemporalConfigRow
            activeIndex={activeIndex}
            formatDate={formatDate}
            index={index}
            key={index}
            now={now}
            record={record}
            t={t}
          />
        ))}
      </tbody>
    </table>
  );
}

function TemporalConfigStatusBadge({ 
  isVirtual, 
  isActive, 
  isFuture, 
  t 
}: { 
  isActive: boolean; 
  isFuture: boolean; 
  isVirtual: boolean; 
  t: TFunction 
}) {
  if (isVirtual) {
    return (
      <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-amber-50 dark:bg-amber-950 text-amber-600 dark:text-amber-300 border-amber-100 dark:border-amber-800 shadow-sm">
        <AlertCircle size={12} /> {t("payroll.settings.virtual")}
      </span>
    );
  }
  if (isActive) {
    return (
      <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-emerald-50 dark:bg-emerald-950 text-emerald-600 dark:text-emerald-300 border-emerald-100 dark:border-emerald-800 shadow-sm">
        <CheckCircle2 size={12} /> {t("common.active")}
      </span>
    );
  }
  if (isFuture) {
    return (
      <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-blue-50 dark:bg-blue-950 text-blue-600 dark:text-blue-300 border-blue-100 dark:border-blue-800 shadow-sm">
        <Calendar size={12} /> {t("common.upcoming")}
      </span>
    );
  }
  return (
    <span className="inline-flex items-center gap-1.5 px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border bg-slate-50 dark:bg-slate-800 text-slate-500 dark:text-slate-400 border-slate-100 dark:border-slate-800 opacity-60">
      <History size={12} /> {t("common.expired")}
    </span>
  );
}

function TemporalConfigRow({
  record,
  index,
  activeIndex,
  now,
  t,
  formatDate,
}: {
  activeIndex: number;
  formatDate: (date: string) => string;
  index: number;
  now: Date;
  record: ConfigRecord;
  t: TFunction;
}) {
  const isActive = index === activeIndex;
  const isFuture = new Date(record.effective_date) > now;

  return (
    <tr className={cn("group transition-all hover:bg-slate-50 dark:hover:bg-slate-800/50", isActive && "bg-emerald-50 dark:bg-emerald-950/30")}>
      <td className="px-8 py-6">
        <TemporalConfigStatusBadge 
          isActive={isActive} 
          isFuture={isFuture} 
          isVirtual={!!record.is_virtual} 
          t={t} 
        />
      </td>
      <td className="px-8 py-6"><div className={cn("text-sm font-bold uppercase tracking-tight", isActive ? "text-slate-700 dark:text-slate-200" : "text-slate-500 dark:text-slate-400")}>{record.is_virtual ? t("common.from_start") : formatDate(record.effective_date)}</div></td>
      <td className="px-8 py-6 text-right font-mono font-bold"><span className={cn("text-sm", isActive ? "text-primary-600 dark:text-primary-400" : "text-slate-500 dark:text-slate-400")}>{record.std_working_days} {t("common.days_short")}</span></td>
      <td className="px-8 py-6 text-right font-mono font-bold"><span className={cn("text-sm", isActive ? "text-slate-700 dark:text-slate-200" : "text-slate-300 dark:text-slate-600")}>{record.hours_per_day}h / {t("common.day_short")}</span></td>
      <td className="px-8 py-6 text-right font-mono font-bold"><span className={cn("text-sm", isActive ? "text-amber-600 dark:text-amber-400" : "text-slate-300 dark:text-slate-600")}>{record.max_ot_hours_per_day != null ? `${record.max_ot_hours_per_day}h` : "∞"}</span></td>
      <td className="px-8 py-6 text-right font-mono font-bold"><span className={cn("text-sm", isActive ? "text-amber-600 dark:text-amber-400" : "text-slate-300 dark:text-slate-600")}>{record.max_ot_hours_per_month != null ? `${record.max_ot_hours_per_month}h` : "∞"}</span></td>
    </tr>
  );
}

function TemporalConfigHeader({ title, t, isAdding, setIsAdding }: { isAdding: boolean; setIsAdding: (val: boolean) => void; t: TFunction; title: string }) {
  return (
    <div className="flex flex-col md:flex-row md:items-center justify-between gap-6 px-2">
      <div className="flex items-center gap-4">
        <div className="w-10 h-10 rounded-2xl bg-primary-50 dark:bg-primary-950 flex items-center justify-center text-primary-600 dark:text-primary-400 shadow-sm border border-primary-100 dark:border-primary-800/50"><Calculator size={20} /></div>
        <div>
          <Typography className="text-lg font-bold text-slate-800 dark:text-slate-100 tracking-tight uppercase" variant="h3">{title}</Typography>
          <Typography className="text-[10px] font-bold text-slate-500 dark:text-slate-400 uppercase tracking-widest mt-0.5" variant="p">{t("payroll.settings.policy_desc")}</Typography>
        </div>
      </div>
      {!isAdding && (
        <Button className="flex items-center justify-center gap-2 bg-primary-600 text-white px-6 py-2.5 rounded-2xl font-bold text-[10px] uppercase tracking-widest hover:bg-primary-700 transition-all shadow-lg shadow-primary-100 active:scale-95" onClick={() => { setIsAdding(true); }} variant="ghost"><Plus size={14} /> {t("common.add_new").toUpperCase()}</Button>
      )}
    </div>
  );
}

export default function TemporalConfigTable({ title, records, onSave, isLoading }: Props) {
  const { t } = useTranslation();
  const {
    onClickCapture: handleHorizontalDragClick,
    setRef: setHorizontalDragScrollRef,
  } = useHorizontalDragScroll<HTMLDivElement>();
  const { formatDate } = useDateTimeFormatter();
  const [isAdding, setIsAdding] = useState(false);
  const [newDays, setNewDays] = useState<number>(26);
  const [newHours, setNewHours] = useState<number>(8);
  const [newMaxOtDay, setNewMaxOtDay] = useState<string>("4");
  const [newMaxOtMonth, setNewMaxOtMonth] = useState<string>("40");

  const firstOfMonth = new Date(); firstOfMonth.setDate(1);
  const [newEffectiveDate, setNewEffectiveDate] = useState(firstOfMonth.toISOString().split('T')[0]);

  const handleSave = async () => {
    await onSave({
      std_working_days: newDays,
      hours_per_day: newHours,
      max_ot_hours_per_day: newMaxOtDay === '' ? null : parseFloat(newMaxOtDay),
      max_ot_hours_per_month: newMaxOtMonth === '' ? null : parseFloat(newMaxOtMonth),
      effective_date: newEffectiveDate,
    });
    setIsAdding(false);
  };

  const displayRecords = [...records];
  if (displayRecords.length === 0 && !isLoading) {
    displayRecords.push({ std_working_days: 26, hours_per_day: 8, effective_date: "1970-01-01", is_virtual: true });
  }

  const sortedRecords = displayRecords.sort((a, b) => new Date(b.effective_date).getTime() - new Date(a.effective_date).getTime());
  const now = new Date();
  const activeIndex = sortedRecords.findIndex(r => new Date(r.effective_date) <= now);

  return (
    <div className="space-y-8 animate-in fade-in duration-500">
      <TemporalConfigHeader
        isAdding={isAdding}
        setIsAdding={setIsAdding}
        t={t}
        title={title}
      />

      {isAdding ? (
        <TemporalConfigAddForm
          handleSave={handleSave}
          newDays={newDays}
          newEffectiveDate={newEffectiveDate}
          newHours={newHours}
          newMaxOtDay={newMaxOtDay}
          newMaxOtMonth={newMaxOtMonth}
          setIsAdding={setIsAdding}
          setNewDays={setNewDays}
          setNewEffectiveDate={setNewEffectiveDate}
          setNewHours={setNewHours}
          setNewMaxOtDay={setNewMaxOtDay}
          setNewMaxOtMonth={setNewMaxOtMonth}
          t={t}
        />
      ) : null}

      <div
        className="bg-white dark:bg-slate-900 border border-slate-100 dark:border-slate-800 rounded-[2.5rem] overflow-x-auto overscroll-contain shadow-sm"
        onClickCapture={handleHorizontalDragClick}
        ref={setHorizontalDragScrollRef}
      >
        {isLoading ? (
          <div className="p-20 text-center"><div className="w-10 h-10 border-4 border-primary-600 border-t-transparent rounded-full animate-spin mx-auto"></div></div>
        ) : (
          <TemporalConfigTableContent
            activeIndex={activeIndex}
            formatDate={formatDate}
            now={now}
            sortedRecords={sortedRecords}
            t={t}
          />
        )}
      </div>
    </div>
  );
}
