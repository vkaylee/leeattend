import { describe, it, expect } from 'vitest';
import vi from '../locales/vi.json';
import en from '../locales/en.json';
import zh from '../locales/zh.json';

type JsonObject = { [key: string]: any };

function getAllKeys(obj: JsonObject, prefix = ''): string[] {
  return Object.keys(obj).reduce((keys: string[], key: string) => {
    const value = obj[key];
    const newPrefix = prefix ? `${prefix}.${key}` : key;

    if (value && typeof value === 'object' && !Array.isArray(value)) {
      return [...keys, ...getAllKeys(value, newPrefix)];
    }

    return [...keys, newPrefix];
  }, []);
}

describe('i18n Locales Synchronization', () => {
  const locales = {
    vi,
    en,
    zh
  };

  const localeKeys = {
    vi: getAllKeys(vi),
    en: getAllKeys(en),
    zh: getAllKeys(zh)
  };

  const allLanguages = Object.keys(locales) as (keyof typeof locales)[];

  allLanguages.forEach((sourceLang) => {
    allLanguages.forEach((targetLang) => {
      if (sourceLang === targetLang) return;

      it(`should have all keys from ${sourceLang} present in ${targetLang}`, () => {
        const sourceKeys = localeKeys[sourceLang];
        const targetKeys = localeKeys[targetLang];

        const missingKeys = sourceKeys.filter(key => !targetKeys.includes(key));

        if (missingKeys.length > 0) {
          console.error(`Missing keys in ${targetLang} (present in ${sourceLang}):`, missingKeys);
        }

        expect(missingKeys, `Missing keys in ${targetLang} that exist in ${sourceLang}`).toEqual([]);
      });
    });
  });

  it('all locale files should have the same number of keys', () => {
    const counts = {
      vi: localeKeys.vi.length,
      en: localeKeys.en.length,
      zh: localeKeys.zh.length
    };

    expect(counts.en, 'English should match Vietnamese key count').toBe(counts.vi);
    expect(counts.zh, 'Chinese should match Vietnamese key count').toBe(counts.vi);
  });
});
