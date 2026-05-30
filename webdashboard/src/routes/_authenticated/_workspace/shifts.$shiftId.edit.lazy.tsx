import { createLazyFileRoute } from "@tanstack/react-router";

import ShiftEditorPage from "@/pages/shifts/shift-editor-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/shifts/$shiftId/edit')({
  component: ShiftEditorPage,
});
