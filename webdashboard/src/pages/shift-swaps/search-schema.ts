import { z } from "zod";

/**
 * URL search-param contract for `/shift-swaps`.
 */
export const shiftSwapsSearchSchema = z
  .object({
    tab: z
      .enum(["pending_target", "pending_admin", "approved", "rejected", "cancelled"])
      .catch("pending_admin"),
  })
  .catch({ tab: "pending_admin" });

export type ShiftSwapsSearch = z.infer<typeof shiftSwapsSearchSchema>;
