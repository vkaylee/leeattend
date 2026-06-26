import { createLazyFileRoute } from "@tanstack/react-router";

import DashboardPage from "@/pages/dashboard-page";

export const Route = createLazyFileRoute("/_authenticated/_workspace/dashboard")({
  component: DashboardPage,
});
