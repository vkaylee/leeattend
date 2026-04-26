import { createFileRoute } from "@tanstack/react-router";
import AssignmentRulesPage from "@/pages/AssignmentRulesPage";

export const Route = createFileRoute("/_authenticated/_workspace/assignment-rules")({
  component: AssignmentRulesPage,
});
