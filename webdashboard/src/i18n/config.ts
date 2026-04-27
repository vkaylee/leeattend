import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

const localeLoaders: Record<string, () => Promise<{ default: Record<string, unknown> }>> = {
  en: () => import('./locales/en.json'),
  vi: () => import('./locales/vi.json'),
  zh: () => import('./locales/zh.json'),
};

const loadedLanguages = new Set<string>();

async function loadLocale(lng: string) {
  if (loadedLanguages.has(lng) || !localeLoaders[lng]) return;
  loadedLanguages.add(lng);
  const mod = await localeLoaders[lng]();
  i18n.addResourceBundle(lng, 'translation', mod.default || mod, true, true);
}

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    fallbackLng: 'vi',
    partialBundledLanguages: true,
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
    },
  });

const detected = i18n.language?.split('-')[0] || 'vi';
loadLocale(detected);

i18n.on('languageChanged', (lng) => {
  loadLocale(lng.split('-')[0]);
});

export default i18n;
