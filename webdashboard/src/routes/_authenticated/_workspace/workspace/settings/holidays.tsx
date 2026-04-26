import { createFileRoute } from "@tanstack/react-router";
import HolidaysPage from "@/pages/settings/HolidaysPage";

export const Route = createFileRoute("/_authenticated/_workspace/workspace/settings/holidays")({
  component: HolidaysPage,
});
