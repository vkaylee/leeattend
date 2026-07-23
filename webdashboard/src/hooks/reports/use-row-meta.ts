import type { AttendanceAdjustmentRequest, DailyAttendanceReport } from "@/lib/api";

interface DailyReportTimelineEntry {
  entry_type: string;
  punch_in_time: string | null;
  punch_out_time: string | null;
}

export interface UseRowMetaProps {
  adjustmentMap: Partial<Record<string, AttendanceAdjustmentRequest>>;
  confirmingRow: string | null;
  expandedDebugRows: Set<string>;
  reprocessingRow: string | null;
}

export interface RowMeta {
  dayKey: string;
  existingAdjustment: AttendanceAdjustmentRequest | undefined;
  hasMultipleSessions: boolean;
  isConfirming: boolean;
  isExpanded: boolean;
  isReprocessing: boolean;
  segmentKey: string;
  timeEntries: DailyReportTimelineEntry[];
}

export function useRowMeta(row: DailyAttendanceReport, props: UseRowMetaProps): RowMeta {
  const segmentKey = `${row.employee_id}-${row.date}-${row.segment_id ?? row.sort_order ?? '0'}`;
  const dayKey = `${row.employment_id}-${row.date}`;
  const dayKeyEmployee = `${row.employee_id}-${row.date}`;
  const isReprocessing = props.reprocessingRow === dayKeyEmployee;
  const isConfirming = props.confirmingRow === dayKeyEmployee;
  const isExpanded = props.expandedDebugRows.has(segmentKey);
  const metadata = row.generation_metadata as { time_entries?: DailyReportTimelineEntry[] } | undefined;
  const timeEntries = metadata?.time_entries ?? [];
  const workEntries = timeEntries.filter((e) => e.entry_type === "work");
  const hasMultipleSessions = (row.is_break_auto_deduct === true) ? workEntries.length > 2 : workEntries.length > 1;
  return { segmentKey, dayKey, isReprocessing, isConfirming, isExpanded, existingAdjustment: props.adjustmentMap[dayKey], timeEntries, hasMultipleSessions };
}
