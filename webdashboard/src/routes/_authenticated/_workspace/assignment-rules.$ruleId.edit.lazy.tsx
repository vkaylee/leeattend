import { createLazyFileRoute } from "@tanstack/react-router";

import AssignmentRuleEditorPage from "@/pages/assignment-rule-editor-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/assignment-rules/$ruleId/edit')({
  component: AssignmentRuleEditorPage,
});
