import { createLazyFileRoute } from "@tanstack/react-router";

import ConditionTypesPage from "@/pages/settings/condition-types-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/condition-types')({
  component: ConditionTypesPage,
});
