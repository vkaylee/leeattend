import { createLazyFileRoute } from "@tanstack/react-router";

import AssignmentRuleEditorPage from "@/pages/assignment-rule-editor-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/shifts/rules/assignment-rules/new')({
  component: AssignmentRuleEditorPage,
});
