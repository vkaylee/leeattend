import { createLazyFileRoute } from "@tanstack/react-router";

import SmtpSettingsPage from "@/pages/system-admin/smtp-settings-page";

export const Route = createLazyFileRoute("/_authenticated/_infra/system/smtp")({
  component: SmtpSettingsPage,
});
