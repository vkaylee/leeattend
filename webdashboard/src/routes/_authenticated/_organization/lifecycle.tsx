import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

const lifecycleSearchSchema = z.object({
  q: z.string().optional(),
});

export const Route = createFileRoute("/_authenticated/_organization/lifecycle")({
  validateSearch: (search) => lifecycleSearchSchema.parse(search),
});
