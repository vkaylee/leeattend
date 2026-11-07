import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

const assignmentsSearchSchema = z.object({
  q: z.string().optional(),
});

export const Route = createFileRoute("/_authenticated/_workspace/assignments")({
  validateSearch: (search) => assignmentsSearchSchema.parse(search),
});
