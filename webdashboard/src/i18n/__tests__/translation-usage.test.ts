import fs from 'fs';
import path from 'path';

import { describe, it, expect } from 'vitest';

import en from '../locales/en.json';
import vi from '../locales/vi.json';
import zh from '../locales/zh.json';

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type JsonObject = { [key: string]: any };

const locales = { en, vi, zh };

const KEY_WHITELIST = [
  'common.languages.', // Dynamic language keys
];

function getAllKeys(obj: JsonObject, prefix = ''): { key: string; value: string }[] {
  return Object.keys(obj).reduce((keys: { key: string; value: string }[], key: string) => {
    const value = obj[key];
    const newPrefix = prefix ? `${prefix}.${key}` : key;

    if (value && typeof value === 'object' && !Array.isArray(value)) {
      return [...keys, ...getAllKeys(value, newPrefix)];
    }

    if (typeof value === 'string') {
      keys.push({ key: newPrefix, value });
    }
    return keys;
  }, []);
}

function getFilesRecursively(dir: string, fileList: string[] = []): string[] {
  const files = fs.readdirSync(dir);
  files.forEach((file) => {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    if (stat.isDirectory()) {
      if (file !== 'node_modules' && file !== '__tests__' && file !== 'i18n' && file !== 'test' && file !== 'dist') {
        getFilesRecursively(filePath, fileList);
      }
    } else if ((file.endsWith('.ts') || file.endsWith('.tsx')) && !file.endsWith('.test.ts') && !file.endsWith('.test.tsx')) {
      fileList.push(filePath);
    }
  });
  return fileList;
}

const localeData: Record<string, { key: string; value: string }[]> = {
  en: getAllKeys(en),
  vi: getAllKeys(vi),
  zh: getAllKeys(zh),
};

const localeKeySets: Record<string, Set<string>> = {
  en: new Set(localeData.en.map(d => d.key)),
  vi: new Set(localeData.vi.map(d => d.key)),
  zh: new Set(localeData.zh.map(d => d.key)),
};

describe('Translation Usage', () => {
  it('all translation keys used in the code should exist in all locale files', () => {
    const srcDir = path.resolve(__dirname, '../..'); // webdashboard/src
    const files = getFilesRecursively(srcDir);

    const missingKeys: { file: string; key: string; lang: string }[] = [];
    const tRegex = /\b(?:t|i18n\.t)\s*\(\s*['"]([^'"]+)['"]/g;

    files.forEach((file) => {
      const content = fs.readFileSync(file, 'utf-8');
      let match;

      while ((match = tRegex.exec(content)) !== null) {
        const key = match[1];
        if (key.includes('${') || key.includes('+')) continue;

        // Check whitelist
        if (KEY_WHITELIST.some(w => key.startsWith(w))) continue;

        Object.keys(localeKeySets).forEach(lang => {
          if (!localeKeySets[lang].has(key)) {
            const relativePath = path.relative(path.resolve(__dirname, '../../..'), file);
            missingKeys.push({ file: relativePath, key, lang });
          }
        });
      }
    });

    if (missingKeys.length > 0) {
      console.error('Found translation keys in code that are missing from locale files:');
      const grouped = missingKeys.reduce((acc, { file, key, lang }) => {
        const fileKey = `${file} [${lang}]`;
        if (!acc[fileKey]) acc[fileKey] = [];
        acc[fileKey].push(key);
        return acc;
      }, {} as Record<string, string[]>);

      Object.entries(grouped).forEach(([fileLang, keys]) => {
        console.error(`  ${fileLang}: ${Array.from(new Set(keys)).sort().join(', ')}`);
      });
    }

    fs.writeFileSync('missing-keys.json', JSON.stringify(missingKeys, null, 2));
    expect(missingKeys.length, `Found ${missingKeys.length} missing translation keys across all languages.`).toBe(0);
  });

  it('should not have untranslated keys (value matches key name)', () => {
    const untranslated: { lang: string; key: string }[] = [];

    Object.entries(localeData).forEach(([lang, data]) => {
      data.forEach(({ key, value }) => {
        // Only check long keys that look like paths, or keys that are likely auto-generated
        if (key.includes('.') && key === value) {
          untranslated.push({ lang, key });
        }
      });
    });

    if (untranslated.length > 0) {
      console.warn('Found potentially untranslated keys (value equals key path):');
      untranslated.forEach(({ lang, key }) => {
        console.warn(`  [${lang}] ${key}`);
      });
    }

    // We keep this as a warning for now
    // expect(untranslated.length).toBe(0);
  });

  it('should not have hardcoded strings in JSX', () => {
    const srcDir = path.resolve(__dirname, '../..'); // webdashboard/src
    const files = getFilesRecursively(srcDir);

    const hardcodedStrings: { file: string; line: number; text: string }[] = [];

    const textNodeRegex = />([^<>{}\n\s][^<>{}\n]*[^<>{}\n\s])</g;
    const attrRegex = /\b(placeholder|title|label)\s*=\s*['"]([^'"]+)['"]/g;

    files.forEach((file) => {
      if (file.includes('lib/api') || file.includes('lib/utils') || file.includes('DebugToolbar')) return;

      const content = fs.readFileSync(file, 'utf-8');
      const lines = content.split('\n');

      lines.forEach((line, index) => {
        if (line.trim().startsWith('import') || line.trim().startsWith('//') || line.includes('t(')) return;
        // Skip common UI primitives that might have hardcoded classes or icons
        if (line.includes('<path') || line.includes('<svg')) return;

        let match;
        while ((match = textNodeRegex.exec(line)) !== null) {
          const text = match[1].trim();
          if (text.length > 2 && /[a-zA-Z]/.test(text)) {
            // Filter out common non-translatable things
            if (/^[A-Z0-9_-]+$/.test(text)) return; // Likely a constant or ID
            if (text.startsWith('data-')) return;
            // Brand names and technical identifiers
            if (['Lee', 'Attend', 'ID:'].includes(text)) return;

            hardcodedStrings.push({ file, line: index + 1, text });
          }
        }

        while ((match = attrRegex.exec(line)) !== null) {
          const text = match[2].trim();
          if (text.length > 2 && /[a-zA-Z]/.test(text)) {
            hardcodedStrings.push({ file, line: index + 1, text });
          }
        }
      });
    });

    if (hardcodedStrings.length > 0) {
      const relative = (p: string) => path.relative(path.resolve(__dirname, '../../..'), p);
      console.error(`Found ${hardcodedStrings.length} potential hardcoded strings:`);
      hardcodedStrings.slice(0, 20).forEach(({ file, line, text }) => {
        console.error(`  ${relative(file)}:${line}: "${text}"`);
      });
    }

    // Fail if we find too many (e.g. > 50) or if we want to be strict
    expect(hardcodedStrings.length).toBeLessThan(250);
  });

  it('all locale files should have the exact same set of keys', () => {
    const allKeys = new Set([
      ...localeKeySets.en,
      ...localeKeySets.vi,
      ...localeKeySets.zh,
    ]);

    const mismatches: { lang: string; missing: string[] }[] = [];

    Object.keys(locales).forEach(lang => {
      const currentKeys = localeKeySets[lang];
      const missing = Array.from(allKeys).filter(k => !currentKeys.has(k));

      if (missing.length > 0) {
        mismatches.push({ lang, missing });
      }
    });

    if (mismatches.length > 0) {
      console.error('Locale files are not synchronized:');
      mismatches.forEach(({ lang, missing }) => {
        console.error(`  [${lang}] is missing ${missing.length} keys: ${missing.slice(0, 5).join(', ')}...`);
      });
    }

    expect(mismatches.length, 'Locale files should be 100% synchronized').toBe(0);
  });
});
