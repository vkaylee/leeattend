import { Outlet, createFileRoute } from "@tanstack/react-router";
import React from "react";

export const Route = createFileRoute('/_authenticated/_workspace/shifts/rules')({
  component: () => <Outlet />,
});
