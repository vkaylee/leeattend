/**
 * Client state infrastructure — see `.agent/adr/ADR-006-client-state-model.md`.
 *
 * Layered state model:
 *   Server data              → React Query
 *   Shareable client state   → URL search params via TanStack Router + Zod
 *   Private UI state         → sessionStorage via useSessionState
 *   Cross-session user prefs → localStorage via usePersistentState
 */
export { useSearchPatch } from "./use-search-patch";
export { useSessionState, usePersistentState } from "./use-session-state";
