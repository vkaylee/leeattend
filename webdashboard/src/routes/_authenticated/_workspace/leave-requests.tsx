import { createFileRoute } from "@tanstack/react-router";

import { leaveRequestsSearchSchema } from "@/components/features/leave/search-schema";

export const Route = createFileRoute("/_authenticated/_workspace/leave-requests")({
  validateSearch: (search) => leaveRequestsSearchSchema.parse(search),
});
