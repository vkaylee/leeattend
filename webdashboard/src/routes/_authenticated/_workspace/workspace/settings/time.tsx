import { createFileRoute } from "@tanstack/react-router";
import TimeSettings from "@/pages/settings/TimeSettings";

export const Route = createFileRoute("/_authenticated/_workspace/workspace/settings/time")({
  component: TimeSettings,
});
