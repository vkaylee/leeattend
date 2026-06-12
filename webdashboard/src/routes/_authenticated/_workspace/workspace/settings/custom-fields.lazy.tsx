import { createLazyFileRoute } from "@tanstack/react-router";

import CustomFieldsPage from "@/pages/settings/custom-fields-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/custom-fields')({
  component: CustomFieldsPage,
});
