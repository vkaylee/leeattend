import { createFileRoute } from "@tanstack/react-router";
import ConnectivitySettings from "@/pages/settings/ConnectivitySettings";

export const Route = createFileRoute("/_authenticated/_workspace/workspace/settings/connectivity")({
  component: ConnectivitySettings,
});
