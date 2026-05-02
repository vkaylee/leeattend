import { createLazyFileRoute } from "@tanstack/react-router";

import MembersPage from "@/pages/members-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/members')({
  component: MembersPage,
});
