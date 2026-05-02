import { createLazyFileRoute } from "@tanstack/react-router";

import LeaveBalancesPage from "@/pages/leave-balances-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/leave-balances')({
  component: LeaveBalancesPage,
});
