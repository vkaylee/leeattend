import { createLazyFileRoute } from "@tanstack/react-router";

import LeaveSettings from "@/pages/settings/leave-settings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/leave')({
  component: LeaveSettings,
});
