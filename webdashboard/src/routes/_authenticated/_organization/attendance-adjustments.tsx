import { createFileRoute } from "@tanstack/react-router";

import { adjustmentSearchSchema } from "@/pages/attendance/search-schema";

export const Route = createFileRoute("/_authenticated/_organization/attendance-adjustments")({
  validateSearch: (search) => adjustmentSearchSchema.parse(search),
});
