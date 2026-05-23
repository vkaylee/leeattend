import fs from 'fs';
import path from 'path';
import { describe, it, expect } from 'vitest';

const localesDir = path.resolve(__dirname, '../locales');

function loadMergedLocales(lng: string): Record<string, unknown> {
  const lngDir = path.join(localesDir, lng);
  const files = fs.readdirSync(lngDir);
  const merged: Record<string, unknown> = {};
  
  files.forEach(file => {
    if (file.endsWith('.json')) {
      const content = fs.readFileSync(path.join(lngDir, file), 'utf-8');
      const data = JSON.parse(content) as Record<string, unknown>;
      Object.assign(merged, data);
    }
  });
  
  return merged;
}

const en = loadMergedLocales('en');
const vi = loadMergedLocales('vi');
const zh = loadMergedLocales('zh');

 
type JsonObject = Record<string, unknown>;

function getAllKeys(object: JsonObject, prefix = ''): string[] {
  return Object.keys(object).reduce((keys: string[], key: string) => {
    const value = object[key];
    const newPrefix = prefix ? `${prefix}.${key}` : key;

    if (value && typeof value === 'object' && !Array.isArray(value)) {
      return [...keys, ...getAllKeys(value as JsonObject, newPrefix)];
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
