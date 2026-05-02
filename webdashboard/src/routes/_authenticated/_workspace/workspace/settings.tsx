import { createFileRoute } from "@tanstack/react-router";

import SettingsLayout from "@/pages/settings/settings-layout";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings"
)({
  component: SettingsLayout,
});
