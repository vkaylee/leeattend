import { describe, it, expect } from 'vitest';
import fs from 'fs';
import path from 'path';
import en from '../locales/en.json';
import vi from '../locales/vi.json';
import zh from '../locales/zh.json';

type JsonObject = { [key: string]: any };

const locales = { en, vi, zh };
const localeKeys: Record<string, Set<string>> = {
  en: new Set(getAllKeys(en)),
  vi: new Set(getAllKeys(vi)),
  zh: new Set(getAllKeys(zh)),
};

const KEY_WHITELIST = [
  'common.languages.', // Dynamic language keys
];

function getAllKeys(obj: JsonObject, prefix = ''): string[] {
  return Object.keys(obj).reduce((keys: string[], key: string) => {
    const value = obj[key];
    const newPrefix = prefix ? `${prefix}.${key}` : key;

    if (value && typeof value === 'object' && !Array.isArray(value)) {
      // For objects, the prefix itself might be used as a key if it maps to a string in locale files
      // and we want to allow nested paths
      return [newPrefix, ...keys, ...getAllKeys(value, newPrefix)];
    }

    return [...keys, newPrefix];
  }, []);
}

function getFilesRecursively(dir: string, fileList: string[] = []): string[] {
  const files = fs.readdirSync(dir);
  files.forEach((file) => {
    const filePath = path.join(dir, file);
    const stat = fs.statSync(filePath);
    if (stat.isDirectory()) {
      if (file !== 'node_modules' && file !== '__tests__' && file !== 'i18n' && file !== 'test') {
        getFilesRecursively(filePath, fileList);
      }
    } else if ((file.endsWith('.ts') || file.endsWith('.tsx')) && !file.endsWith('.test.ts') && !file.endsWith('.test.tsx')) {
      fileList.push(filePath);
    }
  });
  return fileList;
}

describe('Translation Usage', () => {
  it('all translation keys used in the code should exist in all locale files', () => {
    const srcDir = path.resolve(__dirname, '../..'); // webdashboard/src
    const files = getFilesRecursively(srcDir);

    const missingKeys: { file: string; key: string; lang: string }[] = [];
    const tRegex = /\bt\s*\(\s*['"]([^'"]+)['"]/g;

    files.forEach((file) => {
      const content = fs.readFileSync(file, 'utf-8');
      let match;

      while ((match = tRegex.exec(content)) !== null) {
        const key = match[1];
        if (key.includes('${') || key.includes('+')) continue;
        
        // Check whitelist
        if (KEY_WHITELIST.some(w => key.startsWith(w))) continue;

        Object.keys(localeKeys).forEach(lang => {
          if (!localeKeys[lang].has(key)) {
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
        console.error(`  ${fileLang}: ${Array.from(new Set(keys)).join(', ')}`);
      });
    }

    expect(missingKeys.length, `Found ${missingKeys.length} missing translation keys across all languages.`).toBe(0);
  });

  it('should not have hardcoded strings in JSX (experimental)', () => {
    const srcDir = path.resolve(__dirname, '../..'); // webdashboard/src
    const files = getFilesRecursively(srcDir);

    const hardcodedStrings: { file: string; line: number; text: string }[] = [];
    
    // Regex for text between tags: >Some Text<
    // We ignore strings that are only whitespace, numbers, or special characters
    const textNodeRegex = />([^<>{}\n]+)</g;
    
    // Regex for attributes: placeholder="Some Text" or title="Some Text"
    const attrRegex = /\b(placeholder|title|label)\s*=\s*['"]([^'"]+)['"]/g;

    files.forEach((file) => {
      // Skip certain files that naturally have strings
      if (file.includes('lib/api') || file.includes('lib/utils')) return;

      const content = fs.readFileSync(file, 'utf-8');
      const lines = content.split('\n');
      
      lines.forEach((line, index) => {
        // Skip imports, comments, and lines with t()
        if (line.trim().startsWith('import') || line.trim().startsWith('//') || line.includes('t(')) return;

        let match;
        while ((match = textNodeRegex.exec(line)) !== null) {
          const text = match[1].trim();
          if (text.length > 1 && /[a-zA-Z]/.test(text)) {
            hardcodedStrings.push({ file, line: index + 1, text });
          }
        }

        while ((match = attrRegex.exec(line)) !== null) {
          const text = match[2].trim();
          if (text.length > 1 && /[a-zA-Z]/.test(text)) {
            hardcodedStrings.push({ file, line: index + 1, text });
          }
        }
      });
    });

    if (hardcodedStrings.length > 0) {
      console.warn('Potential hardcoded strings found:');
      const relative = (p: string) => path.relative(path.resolve(__dirname, '../../..'), p);
      hardcodedStrings.slice(0, 50).forEach(({ file, line, text }) => {
        console.warn(`  ${relative(file)}:${line}: "${text}"`);
      });
      if (hardcodedStrings.length > 50) console.warn(`  ... and ${hardcodedStrings.length - 50} more.`);
    }

    // We keep this as a warning for now since it might have false positives
    // expect(hardcodedStrings.length).toBe(0);
  });

  it('all locale files should have the exact same set of keys', () => {
    const allKeys = new Set([
      ...localeKeys.en,
      ...localeKeys.vi,
      ...localeKeys.zh,
    ]);

    const mismatches: { lang: string; missing: string[]; extra: string[] }[] = [];

    Object.keys(locales).forEach(lang => {
      const currentKeys = localeKeys[lang];
      const missing = Array.from(allKeys).filter(k => !currentKeys.has(k));
      
      if (missing.length > 0) {
        mismatches.push({ lang, missing, extra: [] });
      }
    });

    if (mismatches.length > 0) {
      console.error('Locale files are not synchronized:');
      mismatches.forEach(({ lang, missing }) => {
        if (missing.length > 0) {
          console.error(`  [${lang}] is missing keys present in other languages:`);
          missing.sort().slice(0, 20).forEach(k => console.error(`    - ${k}`));
          if (missing.length > 20) console.error(`    ... and ${missing.length - 20} more.`);
        }
      });
    }

    expect(mismatches.length, 'Locale files should be 100% synchronized').toBe(0);
  });
});
