import { createLazyFileRoute } from "@tanstack/react-router";
import WorkerMonitorPage from "@/pages/WorkerMonitorPage";

export const Route = createLazyFileRoute('/_authenticated/_infra/infra/monitor')({
  component: WorkerMonitorPage,
});
