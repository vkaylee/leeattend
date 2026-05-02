import { createLazyFileRoute } from "@tanstack/react-router";

import MyAttendancePage from "@/pages/ess/my-attendance-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/ess/attendance')({
  component: MyAttendancePage,
});
