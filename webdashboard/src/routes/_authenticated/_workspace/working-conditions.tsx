import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

const workingConditionsSearchSchema = z.object({
  q: z.string().optional(),
});

export const Route = createFileRoute("/_authenticated/_workspace/working-conditions")({
  validateSearch: (search) => workingConditionsSearchSchema.parse(search),
});
