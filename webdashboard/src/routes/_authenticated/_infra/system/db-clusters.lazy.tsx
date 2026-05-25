import { createLazyFileRoute } from "@tanstack/react-router";

import DbClustersPage from "@/pages/system-admin/db-clusters-page";

export const Route = createLazyFileRoute("/_authenticated/_infra/system/db-clusters")({
  component: DbClustersPage,
});
