import * as Sentry from "@sentry/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import React, { Suspense } from "react";

import { ErrorBoundary } from "./components/common/error-boundary";
import { NewVersionBanner } from "./components/common/new-version-banner";
import { DarkModeProvider } from "./context/dark-mode-context";
import { DebugProvider } from "./context/debug-context";
import { useVersionCheck } from "./hooks/use-version-check";
import { createAppRouter } from "./router";



const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 1,
      staleTime: 1000 * 60 * 5, // 5 minutes
      gcTime: 1000 * 60 * 10,   // GC inactive cache after 10 minutes
    },
  },
});

const router = createAppRouter(queryClient);

function AppContent() {
  const { hasNewVersion, dismiss } = useVersionCheck();

  return (
    <>
      {!!hasNewVersion && (
        <NewVersionBanner
          onDismiss={dismiss}
          onRefresh={() => { window.location.reload(); }}
        />
      )}
      <RouterProvider router={router} />
    </>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <DebugProvider>
        <DarkModeProvider>
          { }
          <Sentry.ErrorBoundary fallback={(_error: unknown) => <ErrorBoundary>{null}</ErrorBoundary>}>
            <Suspense fallback={<div className="flex h-screen items-center justify-center">Loading...</div>}>
              <AppContent />
            </Suspense>
          </Sentry.ErrorBoundary>
        </DarkModeProvider>
      </DebugProvider>
    </QueryClientProvider>
  );
}

export default App;
