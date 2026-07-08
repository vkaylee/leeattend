import * as Sentry from "@sentry/react";
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import React, { Suspense } from "react";
import { toast } from "sonner";

import { ErrorBoundary } from "./components/common/error-boundary";
import { GlobalLoader } from "./components/common/global-loader";
import { NewVersionBanner } from "./components/common/new-version-banner";
import { DarkModeProvider } from "./context/dark-mode-context";
import { DebugProvider } from "./context/debug-context";
import { useVersionCheck } from "./hooks/use-version-check";
import { parseApiError } from "./lib/api/error-handler";
import i18n from "./i18n/config";
import { createAppRouter } from "./router";



function showErrorToast(error: unknown) {
  const parsed = parseApiError(error);
  const message = parsed.messageKey
    ? i18n.t(parsed.messageKey)
    : error instanceof Error
      ? error.message
      : i18n.t("errors.unknown");
  if (parsed.severity === "warning") {
    toast.warning(message);
  } else {
    toast.error(message);
  }
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 1000 * 60 * 5, // 5 minutes
      gcTime: 1000 * 60 * 10,   // GC inactive cache after 10 minutes
    },
  },
  queryCache: new QueryCache({
    onError: (_error, query) => {
      // Suppress toast for background refetch (query already has data)
      if (query.state.data === undefined) {
        showErrorToast(_error);
      }
    },
  }),
  mutationCache: new MutationCache({
    onError: (_error) => {
      showErrorToast(_error);
    },
  }),
});

const router = createAppRouter(queryClient);

function AppContent() {
  const { hasNewVersion, dismiss } = useVersionCheck();

  return (
    <>
      {hasNewVersion ? (
        <NewVersionBanner
          onDismiss={dismiss}
          onRefresh={() => { window.location.reload(); }}
        />
      ) : null}
      <RouterProvider router={router} />
    </>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <DebugProvider>
        <DarkModeProvider>
          <Sentry.ErrorBoundary fallback={(_error: unknown) => <ErrorBoundary>{null}</ErrorBoundary>}>
            <Suspense fallback={<GlobalLoader />}>
              <AppContent />
            </Suspense>
          </Sentry.ErrorBoundary>
        </DarkModeProvider>
      </DebugProvider>
    </QueryClientProvider>
  );
}

export default App;
