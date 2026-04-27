import { createLazyFileRoute } from "@tanstack/react-router";
import ConnectivitySettings from "@/pages/settings/ConnectivitySettings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/connectivity')({
  component: ConnectivitySettings,
});
