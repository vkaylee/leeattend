import { z } from "zod";

/**
 * URL search-param contract for `/shifts`.
 *
 * Owned by the route file (`routes/_authenticated/_organization/shifts.index.tsx`)
 * via `validateSearch`. Every field has `.catch()` — see ADR-006.
 */
export const shiftsSearchSchema = z
  .object({
    tab: z.enum(["weekly-roster", "assignments", "directory"]).catch("weekly-roster"),
    view: z.enum(["week", "month"]).catch("week"),
    employeeIds: z.string().optional().catch(undefined),
  })
  .catch({ tab: "weekly-roster", view: "week" });

export type ShiftsSearch = z.infer<typeof shiftsSearchSchema>;
