import { createFileRoute, redirect } from "@tanstack/react-router";

import { fetchMyPermissions } from "@/lib/api";
import { attendanceLogsSearchSchema } from "@/pages/attendance/search-schema";

export const Route = createFileRoute("/_authenticated/_organization/attendance-logs")({
  beforeLoad: async () => {
    try {
      const { permissions } = await fetchMyPermissions();
      if (!permissions.includes("attendance:logs:read")) throw redirect({ to: "/" });
    } catch (error) {
      if (error && typeof error === "object" && "isRedirect" in error) throw error;
      throw redirect({ to: "/" });
    }
  },
  validateSearch: (search) => attendanceLogsSearchSchema.parse(search),
});
