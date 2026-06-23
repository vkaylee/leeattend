import { createLazyFileRoute } from "@tanstack/react-router";

import ShiftsRulesPage from "@/pages/shifts-rules-page";

export const Route = createLazyFileRoute("/_authenticated/_workspace/shifts/rules/")({
  component: ShiftsRulesPage,
});
