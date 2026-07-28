import { createFileRoute } from "@tanstack/react-router";

import { shiftSwapsSearchSchema } from "@/pages/shift-swaps/search-schema";

export const Route = createFileRoute("/_authenticated/_workspace/shift-swaps")({
  validateSearch: (search) => shiftSwapsSearchSchema.parse(search),
});
