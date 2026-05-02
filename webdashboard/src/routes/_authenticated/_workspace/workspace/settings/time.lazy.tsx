import { createLazyFileRoute } from "@tanstack/react-router";

import TimeSettings from "@/pages/settings/time-settings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/time')({
  component: TimeSettings,
});
