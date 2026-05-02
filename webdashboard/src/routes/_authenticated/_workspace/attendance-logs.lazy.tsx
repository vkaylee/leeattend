import { createLazyFileRoute } from "@tanstack/react-router";

import AttendanceLogsPage from "@/pages/attendance-logs-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/attendance-logs')({
  component: AttendanceLogsPage,
});
