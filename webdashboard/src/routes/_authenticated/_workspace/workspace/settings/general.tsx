import { createFileRoute } from "@tanstack/react-router";
import GeneralSettings from "@/pages/settings/GeneralSettings";

export const Route = createFileRoute("/_authenticated/_workspace/workspace/settings/general")({
  component: GeneralSettings,
});
