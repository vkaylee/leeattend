import { createLazyFileRoute } from "@tanstack/react-router";

import OrgChartPage from "@/pages/org-chart-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/org-chart')({
  component: OrgChartPage,
});
