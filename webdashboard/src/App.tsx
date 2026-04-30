import * as Sentry from "@sentry/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";

import { ErrorBoundary } from "./components/common/ErrorBoundary";
import { NewVersionBanner } from "./components/common/NewVersionBanner";
import { DarkModeProvider } from "./context/DarkModeContext";
import { DebugProvider } from "./context/DebugContext";
import { useVersionCheck } from "./hooks/use-version-check";
import { createAppRouter } from "./router";

console.log("🚀 [LeeAttend] Bundle initialized at:", new Date().toISOString());

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
      {hasNewVersion && (
        <NewVersionBanner
          onRefresh={() => window.location.reload()}
          onDismiss={dismiss}
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
            <AppContent />
          </Sentry.ErrorBoundary>
        </DarkModeProvider>
      </DebugProvider>
    </QueryClientProvider>
  );
}

export default App;
