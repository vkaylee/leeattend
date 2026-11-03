import { Outlet } from "@tanstack/react-router";

/** Authenticated routes already establish access; organization context is backend-owned. */
export default function WorkspaceGuard() {
	return <Outlet />;
}
