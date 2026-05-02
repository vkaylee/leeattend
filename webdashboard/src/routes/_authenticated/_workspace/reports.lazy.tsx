import { createLazyFileRoute } from "@tanstack/react-router";

import ReportsPage from "@/pages/reports-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/reports')({
  component: ReportsPage,
});
