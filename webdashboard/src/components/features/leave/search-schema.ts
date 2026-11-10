import { z } from "zod";

/**
 * URL search-param contract for `/leave-requests`.
 *
 * Owned by the route file (`routes/_authenticated/_organization/leave-requests.tsx`)
 * via `validateSearch`. Every field has `.catch()` — see ADR-006.
 */
export const leaveRequestsSearchSchema = z
  .object({
    tab: z.enum(["requests", "behalf_assignments"]).catch("requests"),
    status: z
      .enum(["all", "pending", "approved", "rejected", "cancelled", "pre_approved"])
      .catch("all"),
    employee: z.string().optional().catch(undefined),
  })
  .catch({ tab: "requests", status: "all" });

export type LeaveRequestsSearch = z.infer<typeof leaveRequestsSearchSchema>;
