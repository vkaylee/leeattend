import { createLazyFileRoute } from "@tanstack/react-router";

import ConnectivitySettings from "@/pages/settings/connectivity-settings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/connectivity')({
  component: ConnectivitySettings,
});
