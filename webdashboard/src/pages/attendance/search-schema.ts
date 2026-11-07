import { z } from "zod";

// ── /attendance-logs ────────────────────────────────────────────────

/**
 * URL search-param contract for `/attendance-logs`.
 *
 * Owned by the route file (`routes/_authenticated/_workspace/attendance-logs.tsx`)
 * via `validateSearch`. Every field has `.catch()` — see ADR-006.
 */
export const attendanceLogsSearchSchema = z
  .object({
    tab: z.enum(["recorded", "unknown", "unrecorded", "matrix"]).catch("recorded"),
    employee: z.string().optional().catch(undefined),
    mode: z
      .enum(["this_month", "last_month", "custom", "specific_month"])
      .catch("this_month"),
    from: z.string().optional().catch(undefined),
    to: z.string().optional().catch(undefined),
  })
  .catch({ tab: "recorded", mode: "this_month" });

export type AttendanceLogsSearch = z.infer<typeof attendanceLogsSearchSchema>;

// ── /attendance-adjustments ──────────────────────────────────────────

/**
 * URL search-param contract for `/attendance-adjustments`.
 */
export const adjustmentSearchSchema = z
  .object({
    tab: z.enum(["adjustments", "behalf_assignments"]).catch("adjustments"),
    status: z.enum(["all", "pending", "approved", "rejected"]).catch("all"),
    employee: z.string().optional().catch(undefined),
  })
  .catch({ tab: "adjustments", status: "all" });

export type AdjustmentSearch = z.infer<typeof adjustmentSearchSchema>;
