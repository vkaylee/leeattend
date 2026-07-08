// Helper functions & shared types for working pattern forms.
// Split out from use-working-pattern-form.ts to keep the hook file under max-lines.

import type {
  WorkingPatternDetail,
  WorkingPatternDetailRequest,
} from "@/lib/api/types";

export function initializeGridFromDetails(len: number, details: WorkingPatternDetail[]): string[][] {
  const grid = Array.from({ length: len }, () => [] as string[]);
  details.forEach((detail) => {
    if (detail.shift_id && detail.schedule_day >= 0 && detail.schedule_day < len) {
      grid[detail.schedule_day].push(detail.shift_id);
    }
  });
  return grid;
}

export function convertGridToDetails(grid: string[][]): WorkingPatternDetailRequest[] {
  return grid.flatMap((shiftIds, scheduleDay) => {
    if (shiftIds.length === 0) {
      return [{ schedule_day: scheduleDay, shift_id: null as string | null }];
    }
    return shiftIds.map((shiftId) => ({
      schedule_day: scheduleDay,
      shift_id: shiftId,
    }));
  });
}

export function isCycleLengthInvalid(patternType: string, len: number): boolean {
  return patternType === "rotation" && (len < 1 || len > 366);
}

export function shouldInit(isEditMode: boolean, pattern: unknown, loading: boolean, init: boolean): boolean {
  return isEditMode && Boolean(pattern) && !loading && !init;
}

export function getFirstShiftId(shiftsList: { id: string }[]): string | undefined {
  return shiftsList[0]?.id;
}

export type { CreateWorkingPatternRequest, UpdateWorkingPatternRequest } from "@/lib/api/types";
