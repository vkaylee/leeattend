import { createLazyFileRoute } from "@tanstack/react-router";
import ShiftsPage from "@/pages/ShiftsPage";

export const Route = createLazyFileRoute('/_authenticated/_workspace/shifts')({
  component: ShiftsPage,
});
