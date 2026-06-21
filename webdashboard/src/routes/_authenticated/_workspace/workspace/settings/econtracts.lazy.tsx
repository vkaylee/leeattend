import { createLazyFileRoute } from "@tanstack/react-router";

import EContractsSettingsPage from "@/pages/settings/econtracts-settings-page";

export const Route = createLazyFileRoute(
  "/_authenticated/_workspace/workspace/settings/econtracts"
)({
  component: EContractsSettingsPage,
});
