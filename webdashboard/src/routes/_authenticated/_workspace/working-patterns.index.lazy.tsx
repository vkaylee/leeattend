import { createLazyFileRoute } from "@tanstack/react-router";

import WorkingPatternsPage from "@/pages/working-patterns-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/working-patterns/')({
  component: WorkingPatternsPage,
});
