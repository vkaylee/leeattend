import { createFileRoute } from "@tanstack/react-router";
import ShiftsPage from "@/pages/ShiftsPage";

export const Route = createFileRoute("/_authenticated/_workspace/shifts")({
  component: ShiftsPage,
});
