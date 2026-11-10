import { createFileRoute } from "@tanstack/react-router";

import { shiftsSearchSchema } from "@/pages/shifts/search-schema";

export const Route = createFileRoute("/_authenticated/_organization/shifts/")({
  validateSearch: (search) => shiftsSearchSchema.parse(search),
});
