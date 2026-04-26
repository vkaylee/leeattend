import { createFileRoute } from "@tanstack/react-router";
import WorkerMonitorPage from "@/pages/WorkerMonitorPage";

export const Route = createFileRoute("/_authenticated/_infra/infra/monitor")({
  component: WorkerMonitorPage,
});
