import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

const contractsSearchSchema = z.object({
  q: z.string().optional(),
});

export const Route = createFileRoute("/_authenticated/_workspace/econtracts")({
  validateSearch: (search) => contractsSearchSchema.parse(search),
});
