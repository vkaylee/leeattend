import { createLazyFileRoute } from "@tanstack/react-router";

import MyClockPage from "@/pages/ess/my-clock-page";

export const Route = createLazyFileRoute('/_authenticated/_workspace/ess/clock')({
  component: MyClockPage,
});
