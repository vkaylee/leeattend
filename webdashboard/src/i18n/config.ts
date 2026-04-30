// eslint-disable-next-line import-x/no-named-as-default
import i18n, { BackendModule } from 'i18next';
import LanguageDetector from 'i18next-browser-languagedetector';
import { initReactI18next } from 'react-i18next';

const localeLoaders: Record<string, () => Promise<{ default: Record<string, unknown> }>> = {
  en: () => import('./locales/en.json'),
  vi: () => import('./locales/vi.json'),
  zh: () => import('./locales/zh.json'),
};

export const lazyLoadBackend: BackendModule = {
  type: 'backend',
  init: () => {},
  read: (language, _namespace, callback) => {
    const lng = language.split('-')[0];
    if (localeLoaders[lng]) {
      localeLoaders[lng]()
        .then((mod) => callback(null, mod.default || mod))
        .catch((err) => callback(err, null));
    } else {
      callback(new Error(`Language ${lng} not found`), null);
    }
  },
};

// eslint-disable-next-line import-x/no-named-as-default-member
i18n
  .use(lazyLoadBackend)
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    fallbackLng: 'vi',
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
    },
  });

export default i18n;
