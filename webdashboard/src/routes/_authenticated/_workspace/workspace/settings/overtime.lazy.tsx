import { createLazyFileRoute } from "@tanstack/react-router";

import OvertimeSettings from "@/pages/settings/overtime-settings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/overtime')({
  component: OvertimeSettings,
});
