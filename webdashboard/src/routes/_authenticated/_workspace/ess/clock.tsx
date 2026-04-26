import { createFileRoute } from "@tanstack/react-router";
import MyClockPage from "@/pages/ess/MyClockPage";

export const Route = createFileRoute("/_authenticated/_workspace/ess/clock")({
  component: MyClockPage,
});
