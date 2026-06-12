import { createLazyFileRoute } from "@tanstack/react-router";

import LifecycleTemplatesPage from "@/pages/settings/lifecycle-templates-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/lifecycle-templates')({
  component: LifecycleTemplatesPage,
});
