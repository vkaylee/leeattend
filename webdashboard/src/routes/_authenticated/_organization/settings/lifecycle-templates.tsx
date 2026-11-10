import { createFileRoute } from "@tanstack/react-router";

import { lifecycleTemplatesSearchSchema } from "@/pages/settings/search-schema";

export const Route = createFileRoute("/_authenticated/_organization/settings/lifecycle-templates")({
  validateSearch: (search) => lifecycleTemplatesSearchSchema.parse(search),
});
