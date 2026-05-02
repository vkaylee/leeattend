import { createLazyFileRoute } from "@tanstack/react-router";

import LeaveRequestsPage from "@/pages/leave-requests-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/leave-requests')({
  component: LeaveRequestsPage,
});
