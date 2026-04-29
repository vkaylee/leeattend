import { createLazyFileRoute } from "@tanstack/react-router";

import LeaveSettings from "@/pages/settings/LeaveSettings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/leave')({
  component: LeaveSettings,
});
