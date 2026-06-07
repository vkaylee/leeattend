import { createLazyFileRoute } from "@tanstack/react-router";

import AssignmentRulesPage from "@/pages/assignment-rules-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/assignment-rules/')({
  component: AssignmentRulesPage,
});
