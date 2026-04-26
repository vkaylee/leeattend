import { createFileRoute, Outlet } from "@tanstack/react-router";
import InfraLayout from "@/components/layout/InfraLayout";

export const Route = createFileRoute("/_authenticated/_infra")({
  component: InfraLayout,
});
