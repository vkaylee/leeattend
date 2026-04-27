import { createFileRoute } from "@tanstack/react-router";
import MembersPage from "@/pages/MembersPage";

export const Route = createFileRoute(
  "/_authenticated/_workspace/workspace/settings/members"
)({
  component: MembersPage,
});
