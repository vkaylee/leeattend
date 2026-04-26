import { createFileRoute } from "@tanstack/react-router";
import AttendanceAdjustmentsPage from "@/pages/AttendanceAdjustmentsPage";

export const Route = createFileRoute("/_authenticated/_workspace/attendance-adjustments")({
  component: AttendanceAdjustmentsPage,
});
