import { z } from "zod";

/**
 * URL search-param contract for `/profile`.
 */
export const profileSearchSchema = z
  .object({
    tab: z.enum(["profile", "personalization", "security"]).catch("profile"),
  })
  .catch({ tab: "profile" });

export type ProfileSearch = z.infer<typeof profileSearchSchema>;
