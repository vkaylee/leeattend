import { createFileRoute } from "@tanstack/react-router";

import { profileSearchSchema } from "@/pages/profile/search-schema";

export const Route = createFileRoute("/_authenticated/_organization/profile")({
  validateSearch: (search) => profileSearchSchema.parse(search),
});
