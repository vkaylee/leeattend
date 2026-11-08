import { Download, Loader2 } from "lucide-react";
import React, { useState } from "react";
import { useTranslation } from "react-i18next";

import { format } from "date-fns";

import Button from "@/components/ui/button";
import { Typography } from "@/components/ui/typography";
import { useInfiniteScroll } from "@/hooks/use-infinite-scroll";

import { useUnrecordedShiftsData } from "./hooks/use-unrecorded-shifts-data";
import { NoShowTable } from "./unrecorded-shifts/no-show-table";
import { NoShowFilterBar } from "./unrecorded-shifts/NoShowFilterBar";
import { useNoShowExport } from "./unrecorded-shifts/useNoShowExport";

function getTodayIso(): string {
  return format(new Date(), "yyyy-MM-dd");
}

// ── Loading skeleton ───────────────────────────────────────────────────────

function LoadingSkeleton() {
  return (
    <div className="space-y-2 animate-pulse">
      {Array.from({ length: 6 }).map((_, i) => (
        <div
          className="h-14 rounded-xl bg-slate-100 dark:bg-slate-800"
          key={i}
        />
      ))}
    </div>
  );
}

// ── Header ──────────────────────────────────────────────────────────────────

function UnrecordedShiftsHeader({
  absentCount,
  isExporting,
  isLoading,
  onExport,
  t,
}: {
  absentCount: number;
  isExporting: boolean;
  isLoading: boolean;
  onExport: () => void;
  t: (key: string) => string;
}) {
  return (
    <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
      <div>
        <Typography className="font-bold text-sm text-slate-800 dark:text-slate-100" variant="h3">
          {t("unrecorded_shifts.title")}
        </Typography>
        <Typography className="text-xs text-slate-500 dark:text-slate-400 mt-0.5" variant="p">
          {t("unrecorded_shifts.subtitle")}
        </Typography>
      </div>

      <div className="flex items-center gap-3 shrink-0">
        <Button
          className="flex items-center gap-2 px-4 py-2 rounded-xl text-[11px] font-bold uppercase tracking-widest"
          disabled={absentCount === 0 || isLoading || isExporting}
          id="unrecorded-export-btn"
          onClick={onExport}
          variant="outline"
        >
          {isExporting ? <Loader2 className="animate-spin" size={14} /> : <Download size={14} />}
          {t("unrecorded_shifts.export_btn")}
          {absentCount > 0 && !isExporting && (
            <span className="ml-1 px-2 py-0.5 bg-slate-100 dark:bg-slate-800 rounded-full text-[9px]">
              {absentCount}
            </span>
          )}
        </Button>
      </div>
    </div>
  );
}

// ── Main Component ─────────────────────────────────────────────────────────

export function UnrecordedShiftsTab() {
  const { t } = useTranslation();
  const today = getTodayIso();

  // ── Filter state ──────────────────────────────────────────────────────────
  const [selectedDate, setSelectedDate] = useState<string>(today);
  const [selectedDept, setSelectedDept] = useState<string>("__all__");
  const [selectedShiftTypes, setSelectedShiftTypes] = useState<string[]>([]);
  const [employeeIds, setEmployeeIds] = useState<string[]>([]);
  // The operational default is the action queue: employees who are already
  // beyond their configured grace period. HR can opt out to inspect Zone A.
  const [onlyZoneB, setOnlyZoneB] = useState(true);

  const isViewingToday = selectedDate === today;

  // ── Data ──────────────────────────────────────────────────────────────────
  const {
    absentRows,
    departments,
    employeeMap,
    filteredRows,
    hasNextPage,
    isFetchingNextPage,
    isLoading,
    queryKey,
    tracesByKey,
    workspaceTimezone,
    fetchNextPage,
  } = useUnrecordedShiftsData({
    employeeIds,
    selectedDate,
    selectedDept,
    selectedShiftTypes,
    onlyZoneB,
  });

  const loadMoreRef = useInfiniteScroll<HTMLTableRowElement>({
    hasNextPage: hasNextPage ?? false,
    isFetchingNextPage,
    fetchNextPage: () => { void fetchNextPage(); },
  });

  // ── Export ─────────────────────────────────────────────────────────────
  const { exportFn: handleExport, isExporting } = useNoShowExport({
    queryKey,
    fetchNextPage,
    employeeMap,
    selectedDate,
    workspaceTimezone,
    selectedDept,
  });

  const clearFilters = () => {
    setSelectedDept("__all__");
    setSelectedShiftTypes([]);
    setEmployeeIds([]);
    setOnlyZoneB(true);
  };

  // ── Render ─────────────────────────────────────────────────────────────
  return (
    <div className="space-y-6 animate-in fade-in duration-500">
      <UnrecordedShiftsHeader
        absentCount={absentRows.length}
        isExporting={isExporting}
        isLoading={isLoading}
        onExport={() => { void handleExport(); }}
        t={t}
      />

      {/* Date picker + Filters */}
      <NoShowFilterBar
        departments={departments}
        isViewingToday={isViewingToday}
        onClearFilters={clearFilters}
        onDateChange={setSelectedDate}
        onDeptChange={setSelectedDept}
        onlyZoneB={onlyZoneB}
        onOnlyZoneBToggle={() => { setOnlyZoneB((v) => !v); }}
        onShiftTypesChange={setSelectedShiftTypes}
        selectedDate={selectedDate}
        selectedDept={selectedDept}
        selectedShiftTypes={selectedShiftTypes}
        today={today}
      />

      {/* Table */}
      {isLoading ? (
        <LoadingSkeleton />
      ) : (
        <NoShowTable
          filteredRows={filteredRows}
          hasNextPage={hasNextPage ?? false}
          isFetchingNextPage={isFetchingNextPage}
          loadMoreRef={loadMoreRef}
          onlyZoneB={onlyZoneB}
          t={t}
          tracesByKey={tracesByKey}
        />
      )}
    </div>
  );
}
