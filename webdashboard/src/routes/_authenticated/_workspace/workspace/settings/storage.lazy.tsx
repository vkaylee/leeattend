import { createLazyFileRoute } from "@tanstack/react-router";

import { StorageSettings } from "@/pages/settings/storage-settings";

export const Route = createLazyFileRoute(
  "/_authenticated/_workspace/workspace/settings/storage"
)({
  component: StorageSettings,
});
