import { createFileRoute } from "@tanstack/react-router";

import AppShell from "@/components/layout/app-shell";

function InfraAppShell() {
  return <AppShell variant="infra" />;
}

export const Route = createFileRoute("/_authenticated/_infra")({
  component: InfraAppShell,
});
