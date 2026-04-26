import { createFileRoute } from "@tanstack/react-router";
import MyProfilePage from "@/pages/ess/MyProfilePage";

export const Route = createFileRoute("/_authenticated/_workspace/ess/profile")({
  component: MyProfilePage,
});
