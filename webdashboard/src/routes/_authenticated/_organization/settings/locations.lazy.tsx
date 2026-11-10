import { createLazyFileRoute } from '@tanstack/react-router';

import { ErrorBoundary } from "@/components/common/error-boundary";
import LocationsSettings from "@/pages/settings/locations-settings";


function LocationsSettingsWithErrorBoundary() {
  return (
    <ErrorBoundary variant="full">
      <LocationsSettings />
    </ErrorBoundary>
  );
}

export const Route = createLazyFileRoute('/_authenticated/_organization/settings/locations')({
  component: LocationsSettingsWithErrorBoundary,
});
