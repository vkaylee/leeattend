import { initReactI18next } from 'react-i18next';

// eslint-disable-next-line import-x/no-named-as-default
import i18n, { BackendModule } from 'i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

export const namespaces = [
  'common',
  'payroll',
  'shift_swaps',
  'employees',
  'departments',
  'attendance',
  'leave',
  'settings',
  'system_roles',
  'custom_fields',
];

export const lazyLoadBackend: BackendModule = {
  type: 'backend',
  init: () => { /* empty */ },
  read: (language, namespace, callback) => {
    const lng = language.split('-')[0];
    const supportedLngs = ['en', 'vi', 'zh'];
    if (!supportedLngs.includes(lng)) {
      callback(new Error(`Language ${lng} not found`), null);
      return;
    }
    import(`./locales/${lng}/${namespace}.json`)
      .then((module_: { default?: Record<string, unknown> } | Record<string, unknown>) => {
        const data = 'default' in module_ ? module_.default ?? module_ : module_;
        callback(null, data);
      })
      .catch((error: unknown) => {
        callback(error instanceof Error ? error : new Error(String(error)), null);
      });
  },
};

// eslint-disable-next-line import-x/no-named-as-default-member
void i18n
  .use(lazyLoadBackend)
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    fallbackLng: 'vi',
    ns: namespaces,
    defaultNS: 'common',
    fallbackNS: namespaces,
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
    },
  });

type GenericTFunction = (
  key: string | string[],
  optionsOrFallbackValue?: unknown,
  options?: Record<string, unknown>
) => string;

// eslint-disable-next-line import-x/no-named-as-default-member
// i18next's public TFunction type does not structurally overlap the simplified
// GenericTFunction signature used here, so this sanctioned `as unknown as`
// bridges the two (coding-standard §10.1). Not a drift mask — kept deliberate.
const originalT = i18n.t.bind(i18n) as unknown as GenericTFunction;

const mergeNsOption = (opts: unknown, ns: string): Record<string, unknown> | undefined => {
  if (opts && typeof opts === 'object') {
    const record = opts as Record<string, unknown>;
    return record.ns ? record : { ...record, ns };
  }
  return undefined;
};

const customT = (
  key: string | string[],
  optionsOrFallbackValue?: unknown,
  options?: Record<string, unknown>
): string => {
  if (typeof key !== 'string' || !key.includes('.')) {
    return originalT(key, optionsOrFallbackValue, options);
  }

  let ns = key.split('.')[0];
  if (["system_admin", "audit_logs", "rolesPage"].includes(ns)) {
    ns = "system_roles";
  }
  if (!namespaces.includes(ns)) {
    return originalT(key, optionsOrFallbackValue, options);
  }

  const opt1 = mergeNsOption(optionsOrFallbackValue, ns);
  if (opt1) {
    return originalT(key, opt1, options);
  }

  const opt2 = mergeNsOption(options, ns);
  if (opt2) {
    return originalT(key, optionsOrFallbackValue, opt2);
  }

  if (optionsOrFallbackValue === undefined) {
    return originalT(key, { ns });
  }

  return originalT(key, optionsOrFallbackValue, options);
};

interface CustomI18n {
  t: GenericTFunction;
}

// Sanctioned bridge: assigns the custom t wrapper onto the i18next instance
// (coding-standard §10.1). Not a drift mask — kept deliberate.
(i18n as unknown as CustomI18n).t = customT;

export default i18n;
