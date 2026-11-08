import { Users, Undo2 } from "lucide-react";
import { useCallback, useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import type { useShifts } from "@/pages/shifts/hooks/use-shifts";

import { EmployeeSearch } from "@/components/common/employee-search";
import { BulkEmployeePickerModal } from "@/components/shifts/bulk-employee-picker-modal";
import Button from "@/components/ui/button";
import { Employee } from "@/lib/api/types";

import { EmployeeChecklist } from "./roster-toolbar-parts";

export function MonthViewControls({
  hook, selectedEmployeesMeta,
}: {
  hook: ReturnType<typeof useShifts>;
  selectedEmployeesMeta?: Employee[];
}) {
  const { t } = useTranslation();
  const selectedMetaRef = useRef<Employee[]>(selectedEmployeesMeta ?? []);
  useEffect(() => { selectedMetaRef.current = selectedEmployeesMeta ?? []; }, [selectedEmployeesMeta]);

  const [undoSnapshot, setUndoSnapshot] = useState<{ ids: string[]; meta: Employee[] } | null>(null);
  const undoTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const clearUndoTimer = useCallback(() => { if (undoTimer.current) { clearTimeout(undoTimer.current); undoTimer.current = null; } }, []);
  useEffect(() => clearUndoTimer, [clearUndoTimer]);

  const handleClearAll = useCallback(() => {
    const ids = [...hook.selectedEmployeeIds];
    const meta = [...selectedMetaRef.current];
    if (ids.length === 0) return;
    setUndoSnapshot({ ids, meta });
    selectedMetaRef.current = [];
    hook.setSelectedEmployeeIds([]);
    hook.setSelectedEmployees([]);
    clearUndoTimer();
    undoTimer.current = setTimeout(() => { setUndoSnapshot(null); }, 6000);
  }, [hook, clearUndoTimer]);

  const handleUndo = useCallback(() => {
    if (!undoSnapshot) return;
    selectedMetaRef.current = undoSnapshot.meta;
    hook.setSelectedEmployeeIds(undoSnapshot.ids);
    hook.setSelectedEmployees(undoSnapshot.meta);
    setUndoSnapshot(null);
    clearUndoTimer();
  }, [undoSnapshot, hook, clearUndoTimer]);

  const [isBulkOpen, setIsBulkOpen] = useState(false);
  const handleBulkCommit = useCallback((ids: string[], meta: Employee[]) => {
    selectedMetaRef.current = meta;
    hook.setSelectedEmployeeIds(ids);
    hook.setSelectedEmployees(meta);
  }, [hook]);

  return (
    <>
      <div className="flex items-center gap-2 w-full md:max-w-2xl">
        <div className="flex-1 min-w-0">
          <EmployeeSearch clearOnEscape data-testid="monthly-employee-search" departments={hook.departments} hideChips multiple
          onChange={(val) => {
            const ids = Array.isArray(val) ? val : (val ? [val] : []);
            const filteredMeta = selectedMetaRef.current.filter(e => ids.includes(e.id));
            selectedMetaRef.current = filteredMeta;
            hook.setSelectedEmployeeIds(ids);
            hook.setSelectedEmployees(filteredMeta);
          }}
          onSelectEmployee={(emp) => {
            if (!selectedMetaRef.current.some(e => e.id === emp.id)) {
              selectedMetaRef.current = [...selectedMetaRef.current, emp];
              hook.setSelectedEmployees(selectedMetaRef.current);
            }
          }}
          placeholder={t("shifts.monthly.search_placeholder")}
          value={hook.selectedEmployeeIds}  />
        </div>
        <Button className="flex h-[var(--ui-height-md)] shrink-0 items-center gap-2 rounded-2xl border border-slate-100 dark:border-slate-700 bg-slate-50 dark:bg-slate-800 px-4 text-sm font-bold uppercase tracking-tight text-slate-600 dark:text-slate-300 hover:border-primary-200 dark:hover:border-primary-800 shadow-inner transition-all cursor-pointer"
          data-testid="monthly-bulk-select" onClick={() => { setIsBulkOpen(true); }} type="button" variant="ghost">
          <Users size={16} /><span className="hidden sm:inline">{t("shifts.monthly.bulk_select")}</span>
        </Button>
      </div>
      {isBulkOpen ? (
        <BulkEmployeePickerModal departments={hook.departments} initialIds={hook.selectedEmployeeIds} initialMeta={selectedEmployeesMeta ?? []} isOpen={isBulkOpen}
          onClose={() => { setIsBulkOpen(false); }} onCommit={handleBulkCommit}  />
      ) : null}
      {!!selectedEmployeesMeta && selectedEmployeesMeta.length > 0 && (
        <EmployeeChecklist employees={selectedEmployeesMeta} hook={hook} onClearAll={handleClearAll} />
      )}
      {!!undoSnapshot && (
        <div className="flex items-center justify-between gap-3 px-3.5 py-2.5 bg-slate-800 dark:bg-slate-700 text-white rounded-2xl animate-in fade-in slide-in-from-top-2 duration-300">
          <span className="text-xs font-bold uppercase tracking-widest">{t("shifts.monthly.cleared_selection", { count: undoSnapshot.ids.length })}</span>
          <Button className="flex items-center gap-1.5 px-3 py-1.5 bg-white/10 hover:bg-white/20 text-white rounded-xl text-[10px] font-extrabold uppercase tracking-widest transition-all active:scale-95 cursor-pointer"
            onClick={handleUndo} variant="ghost"><Undo2 size={14} /> {t("shifts.monthly.undo")}</Button>
        </div>
      )}
    </>
  );
}
