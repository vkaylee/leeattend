import { createLazyFileRoute } from "@tanstack/react-router";

import TimeSettings from "@/pages/settings/TimeSettings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/time')({
  component: TimeSettings,
});
