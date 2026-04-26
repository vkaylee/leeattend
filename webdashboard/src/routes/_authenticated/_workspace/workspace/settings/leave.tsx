import { createFileRoute } from "@tanstack/react-router";
import LeaveSettings from "@/pages/settings/LeaveSettings";

export const Route = createFileRoute("/_authenticated/_workspace/workspace/settings/leave")({
  component: LeaveSettings,
});
