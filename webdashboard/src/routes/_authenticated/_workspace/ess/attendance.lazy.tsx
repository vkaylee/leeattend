import { createLazyFileRoute } from "@tanstack/react-router";
import MyAttendancePage from "@/pages/ess/MyAttendancePage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/ess/attendance')({
  component: MyAttendancePage,
});
