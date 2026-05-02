import { createFileRoute } from "@tanstack/react-router";

import InfraLayout from "@/components/layout/infra-layout";

export const Route = createFileRoute("/_authenticated/_infra")({
  component: InfraLayout,
});
