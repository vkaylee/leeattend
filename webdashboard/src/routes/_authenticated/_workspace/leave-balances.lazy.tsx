import { createLazyFileRoute } from "@tanstack/react-router";

import LeaveBalancesPage from "@/pages/LeaveBalancesPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/leave-balances')({
  component: LeaveBalancesPage,
});
