import { createLazyFileRoute } from "@tanstack/react-router";

import { ErrorBoundary } from "@/components/common/error-boundary";
import ShiftEditorPage from "@/pages/shifts/shift-editor-page";


function ShiftEditorPageWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <ShiftEditorPage />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_organization/shifts/$shiftId/edit')({
  component: ShiftEditorPageWithErrorBoundary,
});
