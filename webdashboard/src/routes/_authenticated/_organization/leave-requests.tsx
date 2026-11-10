import { createFileRoute } from "@tanstack/react-router";

import { leaveRequestsSearchSchema } from "@/components/features/leave/search-schema";

export const Route = createFileRoute("/_authenticated/_organization/leave-requests")({
  validateSearch: (search) => leaveRequestsSearchSchema.parse(search),
});
