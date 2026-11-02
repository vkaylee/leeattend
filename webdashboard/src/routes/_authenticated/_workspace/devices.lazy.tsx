import { createLazyFileRoute } from "@tanstack/react-router";

import DevicesSettings from "@/pages/settings/devices-settings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/devices')({
  component: DevicesSettings,
});
