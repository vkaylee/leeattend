import { createFileRoute } from "@tanstack/react-router";
import PeopleSettings from "@/pages/settings/PeopleSettings";

export const Route = createFileRoute("/_authenticated/_workspace/workspace/settings/people")({
  component: PeopleSettings,
});
