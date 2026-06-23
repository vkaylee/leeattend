import { createFileRoute } from "@tanstack/react-router";

import { z } from "zod";

const rulesSearchSchema = z.object({
  tab: z.enum(["patterns", "rules"]).optional().catch("patterns"),
});

export const Route = createFileRoute("/_authenticated/_workspace/shifts/rules/")({
  validateSearch: (search) => rulesSearchSchema.parse(search),
});
