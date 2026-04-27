import { createLazyFileRoute } from "@tanstack/react-router";
import AttendanceAdjustmentsPage from "@/pages/AttendanceAdjustmentsPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/attendance-adjustments')({
  component: AttendanceAdjustmentsPage,
});
