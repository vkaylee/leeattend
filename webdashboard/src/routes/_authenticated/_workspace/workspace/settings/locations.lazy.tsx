import { createLazyFileRoute } from '@tanstack/react-router';

import LocationsSettings from "@/pages/settings/locations-settings";

export const Route = createLazyFileRoute('/_authenticated/_workspace/workspace/settings/locations')({
  component: LocationsSettings,
});
