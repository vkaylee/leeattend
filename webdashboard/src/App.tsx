import { BrowserRouter as Router } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { DebugProvider } from "./context/DebugContext";
import { DarkModeProvider } from "./context/DarkModeContext";
import { DebugToolbar } from "./components/debug/DebugToolbar";
import { AppRoutes } from "./AppRoutes";

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

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <DebugProvider>
        <DarkModeProvider>
          <Router>
            <AppRoutes />
          </Router>
        </DarkModeProvider>
        <DebugToolbar />
      </DebugProvider>
    </QueryClientProvider>
  );
}

export default App;
