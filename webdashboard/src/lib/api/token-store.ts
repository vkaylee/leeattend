// In-memory token store.
//
// WHY: access_token historically lived in localStorage, which is readable by any
// JS on the page — including malicious scripts injected via XSS. An attacker
// who gets to run JS once steals the bearer + refresh tokens FOREVER.
//
// FIX: access_token now lives ONLY in module-level memory. It is cleared on
// page unload / tab close, so the attack window collapses from "forever" to
// "while the user has the tab open".
//
// refresh_token is NOT stored here at all. It is persisted by the backend as
// an HttpOnly cookie (see apiserver/src/handlers/auth.rs `create_auth_cookies`).
// HttpOnly cookies are invisible to JS — immune to XSS token theft. The refresh
// endpoint also falls back to reading the cookie when the body is empty.
//

const AUTH_SESSION_HINT_KEY = "leeattend.authenticated";
let accessToken: string | null = null;
let currentUserId: string | null = null;

/** Persist only session presence, never credentials, across page reloads. */
export function markAuthSession(): void {
  localStorage.setItem(AUTH_SESSION_HINT_KEY, "1");
}

export function hasAuthSessionHint(): boolean {
  return localStorage.getItem(AUTH_SESSION_HINT_KEY) === "1";
}

export function clearAuthSessionHint(): void {
  localStorage.removeItem(AUTH_SESSION_HINT_KEY);
}
/** auth token change listeners (e.g. websocket reconnect on token rotation) */
type TokenListener = (token: string | null) => void;
const listeners = new Set<TokenListener>();

export function getAccessToken(): string | null {
  return accessToken;
}

export function setAccessToken(token: string | null): void {
  // Coerce undefined → null so a malformed API response can never leave the
  // store in an `undefined` state (which would fail `getAccessToken() === null`
  // assertions downstream).
  accessToken = token ?? null;
  listeners.forEach((fn) => { fn(accessToken); });
}


export function getCurrentUserId(): string | null {
  return currentUserId;
}

export function setCurrentUserId(id: string | null): void {
  currentUserId = id;
}

/** Subscribe to access-token changes. Returns an unsubscribe function. */
export function onAccessTokenChange(fn: TokenListener): () => void {
  listeners.add(fn);
  return () => { listeners.delete(fn); };
}

/**
 * Wipe every in-memory credential. Called on explicit logout and on
 * 401-failure where the refresh token itself is dead.
 *
 * NOTE: this does NOT touch cookies. Because HttpOnly cookies are invisible to
 * JS, they can only be cleared by the backend `/auth/logout` endpoint (which
 * sets Max-Age=0) — that call happens in `clearAuthSession`, not here.
 */
export function clearMemorySession(): void {
	accessToken = null;
	currentUserId = null;
	clearAuthSessionHint();
	listeners.forEach((fn) => { fn(null); });
}
