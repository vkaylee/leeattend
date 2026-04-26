import { createFileRoute } from "@tanstack/react-router";
import LeaveBalancesPage from "@/pages/LeaveBalancesPage";

export const Route = createFileRoute("/_authenticated/_workspace/leave-balances")({
  component: LeaveBalancesPage,
});
