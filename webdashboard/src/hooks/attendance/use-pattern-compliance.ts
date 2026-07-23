import { useMemo } from "react";

import type { WorkingPatternDetail } from "@/lib/api/types";
import type { Shift } from "@/lib/api/types";
import {
  checkRestHours,
  checkShiftOverlap,
  checkWeeklyHours,
  computeWeeklyTotal,
  type ComplianceIssue,
  type ShiftTimeBlock,
} from "@/lib/utils/shift-compliance-utils";

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

    const issues: ComplianceIssue[] = [];

    // Group by schedule_day
    const byDay = new Map<number, WorkingPatternDetail[]>();
    for (const d of previewDetails) {
      if (!d.shift_id) continue;
      const list = byDay.get(d.schedule_day) || [];
      list.push(d);
      byDay.set(d.schedule_day, list);
    }

    // Build shift time blocks per day
    const allBlocks: ShiftTimeBlock[] = [];
    for (const [day, dayDetails] of byDay) {
      const dayBlocks: ShiftTimeBlock[] = [];
      for (const d of dayDetails) {
        const shift = shifts.find((s) => s.id === d.shift_id);
        const iv = shift?.intervals?.[0];
        if (!iv?.start_time || !iv?.end_time) continue;
        dayBlocks.push({
          shiftId: d.shift_id ?? "",
          shiftName: shift?.name ?? "",
          start: iv.start_time,
          end: iv.end_time,
          day,
        });
        allBlocks.push({
          shiftId: d.shift_id ?? "",
          shiftName: shift?.name ?? "",
          start: iv.start_time,
          end: iv.end_time,
          day,
        });
      }

      // Check overlap within this day
      if (dayBlocks.length > 1) {
        const overlaps = checkShiftOverlap(dayBlocks);
        issues.push(...overlaps);
      }
    }

    // Check rest hours (all blocks sorted by start across the cycle)
    if (allBlocks.length > 1) {
      const restViolations = checkRestHours(allBlocks);
      issues.push(...restViolations);
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
