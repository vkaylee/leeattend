import { createLazyFileRoute } from "@tanstack/react-router";

import EContractsPage from "@/pages/econtracts-page";

export const Route = createLazyFileRoute("/_authenticated/_workspace/econtracts")({
  component: EContractsPage,
});
