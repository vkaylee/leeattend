import { createLazyFileRoute } from "@tanstack/react-router";

import AttendanceLogsPage from "@/pages/AttendanceLogsPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/attendance-logs')({
  component: AttendanceLogsPage,
});
