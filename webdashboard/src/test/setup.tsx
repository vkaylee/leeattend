import "@testing-library/jest-dom";
import { vi } from "vitest";

import viLocale from "../i18n/locales/vi.json";

// Helper to resolve nested keys
const resolveKey = (key: string, object: Record<string, unknown>, variables?: Record<string, unknown>): string => {
  let result = key.split('.').reduce((o: unknown, index) => (o as Record<string, unknown>)?.[index], object) as string | undefined;
  if (!result && key.startsWith('common.')) {
    const fallbackKey = key.substring(7);
    result = fallbackKey.split('.').reduce((o: unknown, index) => (o as Record<string, unknown>)?.[index], object) as string | undefined;
  }
  if (!result) {
    console.warn(`Missing key in test mock: ${key ?? ""}`);
    return key;
  }
  // Handle i18next-style {{variable}} interpolation
  if (variables && typeof result === 'string') {
    return result.replace(/\{\{(\w+)\}\}/g, (_, variableName) => String(variables[variableName] ?? `{{${variableName ?? ""}}}`));
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
    init: () => {},
  },
}));

// Mock ResizeObserver for Radix UI
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
};

// Mock EventSource for SSE
class MockEventSource {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSED = 2;
  readonly CONNECTING = 0;
  readonly OPEN = 1;
  readonly CLOSED = 2;

  url: string;
  readyState = 0;
  onopen = null;
  onmessage = null;
  onerror = null;

  constructor(url: string) {
    this.url = url;
  }

  addEventListener = vi.fn();
  removeEventListener = vi.fn();
  close = vi.fn();
  dispatchEvent = vi.fn();
}

global.EventSource = MockEventSource as unknown as typeof EventSource;

// Mock window.confirm which is missing in happy-dom
if (typeof window !== "undefined" && !window.confirm) {
  window.confirm = vi.fn();
}

// Mock Combobox to be a simple select for existing tests to work

vi.mock("@/components/ui/combobox", () => ({
  Combobox: ({ 
    options, 
    value, 
    onChange, 
    id, 
    "data-testid": testId 
  }: { 
    options: { value: string; label: string }[]; 
    value: string; 
    onChange: (value_: string) => void; 
    id?: string; 
    "data-testid"?: string 
  }) => (
    <select aria-label="Role"
      data-testid={testId}
      id={id}
      onChange={(e) => { onChange(e.target.value); }}
      value={value}
    >
      <option value="">Select...</option>
      {options.map((opt) => (
        <option 
          data-testid={testId ? `${testId ?? ""}-option-${opt.value ?? ""}` : `combo-option-${opt.value ?? ""}`} 
          key={opt.value}
          value={opt.value}
        >
          {opt.label}
        </option>
      ))}
    </select>
  ),
}));
