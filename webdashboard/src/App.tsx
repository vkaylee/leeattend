import { RouterProvider } from "@tanstack/react-router";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { DebugProvider } from "./context/DebugContext";
import { DarkModeProvider } from "./context/DarkModeContext";
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

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <DebugProvider>
        <DarkModeProvider>
          <RouterProvider router={router} />
        </DarkModeProvider>
      </DebugProvider>
    </QueryClientProvider>
  );
}

export default App;
