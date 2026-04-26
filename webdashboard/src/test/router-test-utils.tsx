/**
 * Test helper: wrap a component in a minimal TanStack Router context.
 * Replaces MemoryRouter / BrowserRouter from react-router-dom in unit tests.
 *
 * NOTE: In most unit tests, just use vi.mock("@tanstack/react-router", ...) instead.
 * This file is for integration tests that need the actual router context.
 */
export {};
