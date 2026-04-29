import { createLazyFileRoute } from "@tanstack/react-router";

import LeaveRequestsPage from "@/pages/LeaveRequestsPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/leave-requests')({
  component: LeaveRequestsPage,
});
