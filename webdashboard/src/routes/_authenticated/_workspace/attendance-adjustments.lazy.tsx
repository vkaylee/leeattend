import { createLazyFileRoute } from "@tanstack/react-router";

import AttendanceAdjustmentsPage from "@/pages/attendance-adjustments-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/attendance-adjustments')({
  component: AttendanceAdjustmentsPage,
});
