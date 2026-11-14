import { UserX } from "lucide-react";
import { useCallback, useMemo } from "react";

import InfiniteScrollSentinel from "@/components/common/infinite-scroll-sentinel";
import ResponsiveTable from "@/components/ui/responsive-table";
import { Typography } from "@/components/ui/typography";
import { useHorizontalDragScroll } from "@/hooks/use-horizontal-drag-scroll";
import { ScheduleTrace } from "@/lib/api";
import { cn } from "@/lib/utils";

import type { ZonedNoShowRow } from "../../hooks/use-no-show-rows";

import { NoShowDesktopRow } from "./no-show-desktop-row";
import { NoShowMobileCard } from "./no-show-mobile-card";

interface NoShowTableProps {
  filteredRows: ZonedNoShowRow[];
  hasNextPage: boolean;
  isFetchingNextPage: boolean;
  loadMoreRef: React.RefObject<HTMLTableRowElement | null>;
  onlyZoneB: boolean;
  t: (k: string, o?: Record<string, unknown>) => string;
  tracesByKey: Map<string, ScheduleTrace[]>;
}

function EmptyStateRow({ empty, hint }: { empty: string; hint: string }) {
  return (
    <div className="flex flex-col items-center justify-center gap-3 py-20 text-center">
      <div className="w-16 h-16 rounded-2xl bg-emerald-50 dark:bg-emerald-950 flex items-center justify-center shadow-inner">
        <UserX className="text-emerald-500 dark:text-emerald-400" size={28} />
      </div>
      <Typography className="font-bold text-sm text-slate-700 dark:text-slate-200" variant="p">{empty}</Typography>
      <Typography className="text-xs text-slate-400 dark:text-slate-500 max-w-xs" variant="p">{hint}</Typography>
    </div>
  );
}

/**
 * Renders the no-show report table with zone-aware row styling,
 * employee details, shift schedule, and infinite-scroll loading sentinel.
 */
export function NoShowTable({
  filteredRows,
  hasNextPage,
  isFetchingNextPage,
  loadMoreRef,
  tracesByKey,
  onlyZoneB,
  t,
}: NoShowTableProps) {
  const {
    onClickCapture: handleHorizontalDragClick,
    setRef: setHorizontalDragScrollRef,
  } = useHorizontalDragScroll<HTMLDivElement>();
  const columns = useMemo(() => ({
    zone: t("unrecorded_shifts.zone_header"),
    employee_code: t("unrecorded_shifts.columns.employee_code"),
    employee_name: t("unrecorded_shifts.columns.employee_name"),
    department: t("unrecorded_shifts.columns.department"),
    shift_type: t("unrecorded_shifts.columns.shift_type"),
    shift_in: t("unrecorded_shifts.columns.shift_in"),
    shift_out: t("unrecorded_shifts.columns.shift_out"),
    punch_in: t("unrecorded_shifts.columns.punch_in", { defaultValue: "Punch-in" }),
    punch_out: t("unrecorded_shifts.columns.punch_out", { defaultValue: "Punch-out" }),
  }), [t]);

  const renderCard = useCallback((row: ZonedNoShowRow) => (
    <NoShowMobileCard columns={columns} row={row} t={t} tracesByKey={tracesByKey} />
  ), [columns, t, tracesByKey]);

  const renderDesktopTable = useCallback(() => (
    <div
      className="overflow-x-auto overscroll-contain"
      onClickCapture={handleHorizontalDragClick}
      ref={setHorizontalDragScrollRef}
    >
      <table className="w-full min-w-[1200px] text-sm">
        <thead>
          <tr className="bg-slate-50 dark:bg-slate-800/60 border-b border-slate-200 dark:border-slate-700">
            {Object.entries(columns).map(([k, label]) => (
              <th
                className={cn(
                  "px-5 py-3 text-[10px] font-bold uppercase tracking-widest text-slate-500 dark:text-slate-400 whitespace-nowrap",
                  (k === "shift_in" || k === "shift_out") ? "text-center" : "text-left",
                )}
                key={k}
              >
                {label}
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
          {filteredRows.map((row, idx) => (
            <NoShowDesktopRow
              columns={columns}
              idx={idx}
              key={`${row.employment_id}-${row.date}`}
              row={row}
              t={t}
              tracesByKey={tracesByKey}
            />
          ))}
          {hasNextPage ? (
            <InfiniteScrollSentinel
              colSpan={9}
              isFetchingNextPage={isFetchingNextPage}
              sentinelRef={loadMoreRef}
            />
          ) : null}
        </tbody>
      </table>
    </div>
  ), [filteredRows, columns, t, tracesByKey, hasNextPage, isFetchingNextPage, loadMoreRef, handleHorizontalDragClick, setHorizontalDragScrollRef]);

  if (filteredRows.length === 0) {
    return (
      <EmptyStateRow
        empty={
          onlyZoneB
            ? t("unrecorded_shifts.empty_zone_b")
            : t("unrecorded_shifts.empty")
        }
        hint={
          onlyZoneB
            ? t("unrecorded_shifts.empty_zone_b_hint")
            : t("unrecorded_shifts.empty_hint")
        }
      />
    );
  }

  return (
    <div className="rounded-2xl border border-slate-200 dark:border-slate-700 overflow-hidden shadow-sm">
      <ResponsiveTable<ZonedNoShowRow>
        data={filteredRows}
        getCardKey={(row) => `${row.employment_id}-${row.date}`}
        isLoading={false}
        mobileFooter={
          hasNextPage ? (
            <InfiniteScrollSentinel
              colSpan={9}
              isFetchingNextPage={isFetchingNextPage}
              sentinelRef={loadMoreRef}
            />
          ) : undefined
        }
        renderCard={renderCard}
        renderDesktopTable={renderDesktopTable}
      />
    </div>
  );
}
