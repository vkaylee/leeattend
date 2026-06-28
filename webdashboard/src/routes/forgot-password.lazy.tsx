import { createLazyFileRoute } from "@tanstack/react-router";

import ForgotPasswordPage from "@/pages/forgot-password-page";

export const Route = createLazyFileRoute("/forgot-password")({
  component: ForgotPasswordPage,
});
