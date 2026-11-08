import { createFileRoute } from "@tanstack/react-router";

import { leaveSettingsSearchSchema } from "@/pages/settings/search-schema";

export const Route = createFileRoute("/_authenticated/_workspace/settings/leave")({
  validateSearch: (search) => leaveSettingsSearchSchema.parse(search),
});
