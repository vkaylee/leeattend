import { useMemo } from "react";

import type { WorkingPatternDetail, Shift  } from "@/lib/api/types";

import {
  checkRestHours,
  checkShiftOverlap,
  checkWeeklyHours,
  computeWeeklyTotal,
  type ComplianceIssue,
  type ShiftTimeBlock,
} from "@/lib/utils/shift-compliance-utils";
import { maybeNull } from "@/types/api-overrides";

/** Group preview details by schedule_day, filtering out entries without a shift_id. */
function groupByDay(details: WorkingPatternDetail[]): Map<number, WorkingPatternDetail[]> {
  const byDay = new Map<number, WorkingPatternDetail[]>();
  for (const d of details) {
    if (!d.shift_id) continue;
    const list = byDay.get(d.schedule_day) ?? [];
    list.push(d);
    byDay.set(d.schedule_day, list);
  }
  return byDay;
}

/** Convert a single pattern detail to a ShiftTimeBlock, or null if its shift has no intervals. */
function toShiftBlock(d: WorkingPatternDetail, shifts: Shift[], day: number): ShiftTimeBlock | null {
  const shift = shifts.find((s) => s.id === d.shift_id);
  const iv = shift?.intervals?.[0];
  if (!iv) return null;
  if (!iv.start_time || !iv.end_time) return null;
  return {
    shiftId: d.shift_id ?? "",
    // shift may not be found in the lookup; guard with maybeNull
    shiftName: maybeNull(shift)?.name ?? "",
    start: iv.start_time,
    end: iv.end_time,
    day,
  };
}

/** Build day blocks from grouped details and check intra-day overlaps. */
function buildDayBlocks(
  previewDetails: WorkingPatternDetail[],
  shifts: Shift[],
): { allBlocks: ShiftTimeBlock[]; issues: ComplianceIssue[] } {
  const issues: ComplianceIssue[] = [];
  const allBlocks: ShiftTimeBlock[] = [];

  for (const [day, dayDetails] of groupByDay(previewDetails)) {
    const dayBlocks = dayDetails
      .map((d) => toShiftBlock(d, shifts, day))
      .filter((b): b is ShiftTimeBlock => b !== null);

    allBlocks.push(...dayBlocks);

    if (dayBlocks.length > 1) {
      issues.push(...checkShiftOverlap(dayBlocks));
    }
  }

  return { allBlocks, issues };
}

/**
 * Hook: compute client-side compliance issues from pattern preview details.
 * Extracted from BulkAssignModal to enable isolated testing.
 */
export function usePatternCompliance(
  previewDetails: WorkingPatternDetail[] | undefined,
  shifts: Shift[] | undefined,
): ComplianceIssue[] {
  return useMemo<ComplianceIssue[]>(() => {
    if (!previewDetails?.length || !shifts?.length) return [];

    const { allBlocks, issues } = buildDayBlocks(previewDetails, shifts);

    // Check rest hours (all blocks sorted by start across the cycle)
    if (allBlocks.length > 1) {
      issues.push(...checkRestHours(allBlocks));
    }

    // Check weekly hours
    const weeklyTotal = computeWeeklyTotal(
      previewDetails
        .filter((d) => d.shift_id)
        .map((d) => ({
          scheduleDay: d.schedule_day,
          shiftId: d.shift_id ?? "",
        })),
      shifts.map((s) => ({
        id: s.id,
        name: s.name,
        intervals: s.intervals,
      })),
    );
    const hoursIssue = checkWeeklyHours(weeklyTotal);
    if (hoursIssue) issues.push(hoursIssue);

    return issues;
  }, [previewDetails, shifts]);
}
