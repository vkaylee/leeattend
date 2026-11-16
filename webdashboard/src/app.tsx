import * as Sentry from "@sentry/react";
import { MutationCache, QueryCache, QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { Suspense, useEffect } from "react";

import { GlobalLoader } from "./components/common/global-loader";
import { NewVersionBanner } from "./components/common/new-version-banner";
import { RouteErrorFallback } from "./components/common/route-error-fallback";
import { toast } from "./components/ui/toast-imperative";
import { DarkModeProvider } from "./context/dark-mode-context";
import { useVersionCheck } from "./hooks/use-version-check";
import i18n from "./i18n/config";
import { parseApiError } from "./lib/api/error-handler";
import { loadRuntimeFeatureFlags } from "./lib/feature-flags";
import { logError } from "./lib/logger";
import { createAppRouter } from "./router";



function showErrorToast(error: unknown) {
  logError(error, { component: "QueryClient" });
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

function getErrorStatus(error: unknown): number | undefined {
  return error && typeof error === "object" && "status" in error && typeof error.status === "number" ? error.status : undefined;
}

function hasSuppressedStatus(error: unknown, suppress: unknown): boolean {
  return Array.isArray(suppress) && suppress.includes(getErrorStatus(error));
}

function shouldSuppressQueryError(error: unknown, query: { meta?: unknown; state: { data: unknown } }): boolean {
  const meta = query.meta;
  if (meta && typeof meta === "object" && "suppressErrorToast" in meta) {
    const suppress = meta.suppressErrorToast;
    if (suppress === true || hasSuppressedStatus(error, suppress)) return true;
  }
  return query.state.data !== undefined;
}

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 1000 * 60 * 5, // 5 minutes
      gcTime: 1000 * 60 * 10,   // GC inactive cache after 10 minutes
      refetchOnWindowFocus: false, // dashboard use case: window focus should not trigger refetch storms
    },
  },
  queryCache: new QueryCache({
    onError: (_error, query) => {
      if (!shouldSuppressQueryError(_error, query)) showErrorToast(_error);
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
  const { hasNewVersion, dismiss, refresh } = useVersionCheck();

  useEffect(() => {
    void loadRuntimeFeatureFlags();
  }, []);

  return (
    <>
      {hasNewVersion ? (
        <NewVersionBanner
          onDismiss={dismiss}
          onRefresh={refresh}
        />
      ) : null}
      <RouterProvider router={router} />
    </>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
        <DarkModeProvider>
          <Sentry.ErrorBoundary fallback={(error: unknown) => (
            <RouteErrorFallback error={error instanceof Error ? error : new Error(i18n.t("errors.unknown"))} />
          )}>
            <Suspense fallback={<GlobalLoader />}>
              <AppContent />
            </Suspense>
          </Sentry.ErrorBoundary>
        </DarkModeProvider>
    </QueryClientProvider>
  );
}

export default App;
