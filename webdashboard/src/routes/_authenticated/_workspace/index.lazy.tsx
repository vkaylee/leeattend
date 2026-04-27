import { createLazyFileRoute } from "@tanstack/react-router";
import DashboardPage from "@/pages/DashboardPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/')({
  component: DashboardPage,
});
