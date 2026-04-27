import { createLazyFileRoute } from "@tanstack/react-router";
import MyProfilePage from "@/pages/ess/MyProfilePage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/ess/profile')({
  component: MyProfilePage,
});
