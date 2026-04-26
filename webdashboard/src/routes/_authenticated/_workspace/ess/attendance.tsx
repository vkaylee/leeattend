import { createFileRoute } from "@tanstack/react-router";
import MyAttendancePage from "@/pages/ess/MyAttendancePage";

export const Route = createFileRoute("/_authenticated/_workspace/ess/attendance")({
  component: MyAttendancePage,
});
