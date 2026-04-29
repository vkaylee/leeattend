import { createLazyFileRoute } from "@tanstack/react-router";

import GeneralSettings from "@/pages/settings/GeneralSettings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/general')({
  component: GeneralSettings,
});
