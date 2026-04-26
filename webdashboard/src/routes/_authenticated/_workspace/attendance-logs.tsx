import { createFileRoute } from "@tanstack/react-router";
import AttendanceLogsPage from "@/pages/AttendanceLogsPage";

export const Route = createFileRoute("/_authenticated/_workspace/attendance-logs")({
  component: AttendanceLogsPage,
});
