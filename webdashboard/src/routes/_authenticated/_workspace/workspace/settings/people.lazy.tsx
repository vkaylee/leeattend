import { createLazyFileRoute } from "@tanstack/react-router";
import PeopleSettings from "@/pages/settings/PeopleSettings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/people')({
  component: PeopleSettings,
});
