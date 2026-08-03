import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import AssignmentRuleEditorPage from "@/pages/assignment-rule-editor-page";


function AssignmentRuleEditorPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <AssignmentRuleEditorPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_workspace/shifts/rules/assignment-rules/$ruleId/edit')({
  component: AssignmentRuleEditorPageWithErrorBoundary,
});
