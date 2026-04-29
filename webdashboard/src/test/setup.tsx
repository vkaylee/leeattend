import "@testing-library/jest-dom";
import { vi } from "vitest";

import viLocale from "../i18n/locales/vi.json";

// Helper to resolve nested keys
const resolveKey = (key: string, obj: any, vars?: Record<string, any>): string => {
  let result = key.split('.').reduce((o, i) => o?.[i], obj);
  if (!result && key.startsWith('common.')) {
    const fallbackKey = key.substring(7);
    result = fallbackKey.split('.').reduce((o, i) => o?.[i], obj);
  }
  if (!result) {
    console.warn(`Missing key in test mock: ${key}`);
    return key;
  }
  // Handle i18next-style {{variable}} interpolation
  if (vars && typeof result === 'string') {
    return result.replace(/\{\{(\w+)\}\}/g, (_, varName) => vars[varName] ?? `{{${varName}}}`);
  }
  return result;
};

// Mock react-i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string, vars?: Record<string, any>) => resolveKey(key, viLocale, vars),
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
  readyState: number = 0;
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

global.EventSource = MockEventSource as any;

// Mock window.confirm which is missing in happy-dom
if (typeof window !== "undefined" && !window.confirm) {
  window.confirm = vi.fn();
}

// Mock Combobox to be a simple select for existing tests to work

vi.mock("@/components/ui/Combobox", () => ({

  Combobox: ({ 

    options, 

    value, 

    onChange, 

    id, 

    "data-testid": testId 

  }: any) => (

    <select aria-label="Role"

      id={id}

      data-testid={testId}

      value={value}

      onChange={(e) => onChange(e.target.value)}

    >

      <option value="">Select...</option>

      {options.map((opt: any) => (

        <option 

          key={opt.value} 

          value={opt.value}

          data-testid={testId ? `${testId}-option-${opt.value}` : `combo-option-${opt.value}`}

        >

          {opt.label}

        </option>

      ))}

    </select>

  ),

}));
