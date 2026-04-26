import { createFileRoute } from "@tanstack/react-router";
import ReportsPage from "@/pages/ReportsPage";

export const Route = createFileRoute("/_authenticated/_workspace/reports")({
  component: ReportsPage,
});
