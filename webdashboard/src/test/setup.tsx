import "@testing-library/jest-dom";
import { beforeEach, vi } from "vitest";

import attendance from "../i18n/locales/vi/attendance.json";
import common from "../i18n/locales/vi/common.json";
import departments from "../i18n/locales/vi/departments.json";
import employees from "../i18n/locales/vi/employees.json";
import leave from "../i18n/locales/vi/leave.json";
import payroll from "../i18n/locales/vi/payroll.json";
import settings from "../i18n/locales/vi/settings.json";
import shift_swaps from "../i18n/locales/vi/shift_swaps.json";
import system_roles from "../i18n/locales/vi/system_roles.json";

const viLocale = {
  ...common,
  ...payroll,
  ...shift_swaps,
  ...employees,
  ...departments,
  ...attendance,
  ...leave,
  ...settings,
  ...system_roles,
};

// Ignore Happy DOM AbortError caused by pending async tasks during teardown
process.on('unhandledRejection', (reason) => {
  if (reason instanceof Error && reason.name === 'AbortError') {
    return;
  }
  // We don't re-throw here to prevent failing the entire test suite for other unhandled rejections,
  // but we should log them. Vitest will still report them.
});

// Helper to resolve nested keys
const resolveKey = (key: string, object: Record<string, unknown>, variables?: Record<string, unknown>): string => {
  let result = key.split('.').reduce((o: unknown, index) => {
    if (o === null || o === undefined) return undefined;
    return (o as Record<string, unknown>)[index];
  }, object) as string | undefined;
  if (!result && key.startsWith('common.')) {
    const fallbackKey = key.substring(7);
    result = fallbackKey.split('.').reduce((o: unknown, index) => {
      if (o === null || o === undefined) return undefined;
      return (o as Record<string, unknown>)[index];
    }, object) as string | undefined;
  }
  if (!result) {
    console.warn(`Missing key in test mock: ${key}`);
    return key;
  }
  // Handle i18next-style {{variable}} interpolation
  if (variables && typeof result === 'string') {
    return result.replace(/\{\{(\w+)\}\}/g, (_, variableName) => {
      const val = variables[variableName as keyof typeof variables];
      if (typeof val === 'string' || typeof val === 'number') {
        return String(val);
      }
      return `{{${variableName}}}`;
    });
  }
  return result;
};

// Mock react-i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, variables?: Record<string, unknown>) => resolveKey(key, viLocale, variables),
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'vi',
    },
  }),
  initReactI18next: {
    type: '3rdParty',
    init: () => { /* empty */ },
  },
}));

// Mock ResizeObserver for Radix UI
global.ResizeObserver = class ResizeObserver {
  observe() { /* empty */ }
  unobserve() { /* empty */ }
  disconnect() { /* empty */ }
};

// Mock fetch to prevent Happy DOM from attempting real HTTP requests and aborting them on teardown
global.fetch = vi.fn(() =>
  Promise.resolve(
    new Response(null, {
      status: 200,
      headers: new Headers(),
    }),
  ),
);

// Mock EventSource for SSE. Implements the EventSource interface so the
// instance-side usage is type-safe; the partial `as EventSource` on the class
// value bridges the constructor signature the runtime doesn't exercise.
class MockEventSource implements EventSource {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSED = 2;
  readonly CONNECTING = 0;
  readonly OPEN = 1;
  readonly CLOSED = 2;

  url: string;
  readyState = 0;
  withCredentials = false;
  onopen: ((this: EventSource, ev: Event) => any) | null = null;
  onmessage: ((this: EventSource, ev: MessageEvent) => any) | null = null;
  onerror: ((this: EventSource, ev: Event) => any) | null = null;

  constructor(url: string) {
    this.url = url;
  }

  addEventListener = vi.fn();
  removeEventListener = vi.fn();
  close = vi.fn();
  dispatchEvent = vi.fn();
}

global.EventSource = MockEventSource as typeof EventSource;

// Mock window.confirm which is missing in happy-dom
if (typeof window !== "undefined") {
  window.confirm = vi.fn();
}

// Mock Combobox to be a simple select for existing tests to work

vi.mock("@/components/ui/combobox", () => ({
  Combobox: ({ 
    options, 
    value, 
    onChange, 
    id, 
    "data-testid": testId,
    emptyText
  }: { 
    options: { value: string; label: string }[]; 
    value: string; 
    onChange: (value_: string) => void; 
    id?: string; 
    "data-testid"?: string;
    emptyText?: string;
  }) => (
    <select aria-label="Role"
      data-testid={testId}
      id={id}
      onChange={(e) => { onChange(e.target.value); }}
      value={value}
    >
      <option value="">Select...</option>
      {options.length === 0 && emptyText ? (
        <option value="">{emptyText}</option>
      ) : null}
      {options.map((opt) => (
        <option 
          data-testid={testId ? `${testId}-option-${opt.value}` : `combo-option-${opt.value}`} 
          key={opt.value}
          value={opt.value}
        >
          {opt.label}
        </option>
      ))}
    </select>
  ),
}));

// Bridge localStorage auth-token WRITES to the in-memory token store so that
// existing tests (which set access_token / active_workspace_id / active_user_id
// via localStorage.setItem) keep working unchanged after the token-store
// migration (see lib/api/token-store.ts).
//
// Why patch Storage.prototype: many tests call localStorage.setItem directly in
// their setup, and many components read getCurrentWorkspaceId() etc. at render
// time. The bridge translates the legacy localStorage write path into the new
// memory store synchronously, with zero per-test edits required.
//
// Keys bridged: access_token, active_workspace_id, active_user_id
// NOT bridged by design: refresh_token (now HttpOnly-only — tests must NOT set
// it), i18nextLng, debug_*, theme, and other non-auth localStorage keys.
import {
  setAccessToken as storeSetAccessToken,
  setCurrentWorkspaceId as storeSetWorkspaceId,
  setCurrentUserId as storeSetUserId,
  clearMemorySession,
} from "../lib/api/token-store";

const BRIDGED_KEYS = new Set(["access_token", "active_workspace_id", "active_user_id"]);

function bridgeWrite(key: string | null, value: string | null) {
  if (key === null || !BRIDGED_KEYS.has(key)) return;
  switch (key) {
    case "access_token": storeSetAccessToken(value); break;
    case "active_workspace_id": storeSetWorkspaceId(value); break;
    case "active_user_id": storeSetUserId(value); break;
  }
}

// Patch Storage.prototype so every setItem/removeItem/clear on localStorage is
// bridged. Performed once at setup-file load (runs once per test file).
// The original methods are .bind()'d to the Storage prototype so the
// unbound-method lint rule is satisfied (it flags bare method references
// captured in closures).
const boundSetItem = Storage.prototype.setItem.bind(localStorage);
const boundRemoveItem = Storage.prototype.removeItem.bind(localStorage);
Storage.prototype.setItem = (key: string, value: string): void => {
  boundSetItem(key, value);
  bridgeWrite(key, value);
};
Storage.prototype.removeItem = (key: string): void => {
  boundRemoveItem(key);
  bridgeWrite(key, null);
};

beforeEach(() => {
  // Reset the in-memory singleton between tests so state never leaks.
  clearMemorySession();
  // Clear the localStorage mirrors too (kept in sync with the bridge).
  BRIDGED_KEYS.forEach((key) => { localStorage.removeItem(key); });
});

// Mock useLanguage (depends on useProfile → API) so that components consuming
// useDateTimeFormatter (which now reads i18n language) don't crash during
// render. Self-contained: uses fake profile data without touching use-profile
// module so individual test files can still mock use-profile themselves.
vi.mock("@/hooks/use-language", () => ({
  useLanguage: () => ({
    currentLanguage: "vi",
    currentOption: { code: "vi", flag: "🇻🇳", name: "Vietnamese", nativeName: "Tiếng Việt" },
    languages: [
      { code: "vi", flag: "🇻🇳", name: "Vietnamese", nativeName: "Tiếng Việt" },
      { code: "en", flag: "🇺🇸", name: "English", nativeName: "English" },
      { code: "zh", flag: "🇨🇳", name: "Chinese", nativeName: "中文" },
    ],
    changeLanguage: () => Promise.resolve(),
    userPreference: undefined,
  }),
  SUPPORTED_LANGUAGES: ["vi", "en", "zh"],
}));

vi.mock("@/hooks/use-header-actions", () => ({
  useHeaderActions: () => ({ actions: null, setActions: vi.fn() }),
  HeaderActionsProvider: ({ children }: { children: React.ReactNode }) => children,
}));

vi.mock("@tanstack/react-router", () => ({
  useNavigate: () => vi.fn(),
  useParams: () => ({}),
  useSearch: () => ({}),
  useBlocker: () => ({
    proceed: vi.fn(),
    reset: vi.fn(),
    status: "idle",
  }),
  createLazyFileRoute: () => (options: unknown) => ({ options }),
  createFileRoute: () => (options: unknown) => ({ options }),
  Outlet: () => null,
  Link: ({ to, children, ...props }: React.AnchorHTMLAttributes<HTMLAnchorElement> & { to?: string }) => <a href={to} {...props}>{children}</a>,
}));

