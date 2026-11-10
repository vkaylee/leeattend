import { createFileRoute } from "@tanstack/react-router";

import { payrollSettingsSearchSchema } from "@/pages/settings/search-schema";

export const Route = createFileRoute("/_authenticated/_organization/settings/payroll")({
  validateSearch: (search) => payrollSettingsSearchSchema.parse(search),
});
