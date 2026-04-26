import { createFileRoute } from "@tanstack/react-router";
import LeaveRequestsPage from "@/pages/LeaveRequestsPage";

export const Route = createFileRoute("/_authenticated/_workspace/leave-requests")({
  component: LeaveRequestsPage,
});
