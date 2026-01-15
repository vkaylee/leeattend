import "@testing-library/jest-dom";
import { vi } from "vitest";
import viLocale from "../i18n/locales/vi.json";

// Helper to resolve nested keys
const resolveKey = (key: string, obj: any): string => {
  const result = key.split('.').reduce((o, i) => o?.[i], obj);
  if (!result) console.warn(`Missing key in test mock: ${key}`);
  return result || key;
};

// Mock react-i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => resolveKey(key, viLocale),
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

// Mock Combobox to be a simple select for existing tests to work
vi.mock("@/components/ui/Combobox", () => ({
  Combobox: ({ 
    options, 
    value, 
    onChange, 
    id, 
    "data-testid": testId 
  }: any) => (
    <select
      id={id}
      data-testid={testId}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    >
      {options.map((opt: any) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  ),
}));