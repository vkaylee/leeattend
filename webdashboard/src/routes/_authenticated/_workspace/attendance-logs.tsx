import { createFileRoute } from "@tanstack/react-router";

import { attendanceLogsSearchSchema } from "@/pages/attendance/search-schema";

export const Route = createFileRoute("/_authenticated/_workspace/attendance-logs")({
  validateSearch: (search) => attendanceLogsSearchSchema.parse(search),
});
