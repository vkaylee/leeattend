import { createLazyFileRoute } from "@tanstack/react-router";

import ProfilePage from "@/pages/profile-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/profile')({
  component: ProfilePage,
});
