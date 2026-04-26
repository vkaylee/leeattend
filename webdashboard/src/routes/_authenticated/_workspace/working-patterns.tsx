import { createFileRoute } from "@tanstack/react-router";
import WorkingPatternsPage from "@/pages/WorkingPatternsPage";

export const Route = createFileRoute("/_authenticated/_workspace/working-patterns")({
  component: WorkingPatternsPage,
});
