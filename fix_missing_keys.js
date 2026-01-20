const fs = require('fs');
const path = require('path');

const enPath = path.join(__dirname, 'webdashboard/src/i18n/locales/en.json');
let en = JSON.parse(fs.readFileSync(enPath, 'utf8'));

function getAllKeys(obj, prefix = '') {
  let keys = new Set();
  for (const key in obj) {
    const value = obj[key];
    const newPrefix = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      keys.add(newPrefix);
      getAllKeys(value, newPrefix).forEach(k => keys.add(k));
    } else {
      keys.add(newPrefix);
    }
  }
  return keys;
}

const currentKeys = getAllKeys(en);

function getFilesRecursively(dir, fileList = []) {
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

const srcDir = path.join(__dirname, 'webdashboard/src');
const files = getFilesRecursively(srcDir);
const tRegex = /\bt\s*\(\s*['"]([^'"]+)['"]/g;

let missing = [];
files.forEach((file) => {
  const content = fs.readFileSync(file, 'utf-8');
  let match;
  while ((match = tRegex.exec(content)) !== null) {
    const key = match[1];
    if (key.includes('${') || key.includes('+')) continue;
    if (key.startsWith('common.languages.')) continue;
    if (!currentKeys.has(key)) {
      missing.push(key);
    }
  }
});

missing = Array.from(new Set(missing));
console.log(`Found ${missing.length} missing keys in en.json`);

function setDeep(obj, path, value) {
  const parts = path.split('.');
  let current = obj;
  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i];
    if (!current[part] || typeof current[part] !== 'object') {
      current[part] = {};
    }
    current = current[part];
  }
  const last = parts[parts.length - 1];
  if (!current[last]) {
    current[last] = value;
  }
}

missing.forEach(key => {
  // If we are adding common.something, and something exists at top level, use its value
  let fallbackValue = key;
  if (key.startsWith('common.')) {
    const altKey = key.substring(7);
    // Find value of altKey in en
    const parts = altKey.split('.');
    let val = en;
    for (const p of parts) {
      val = val?.[p];
      if (!val) break;
    }
    if (typeof val === 'string') fallbackValue = val;
  } else {
    // Try with common. prefix
    const altKey = 'common.' + key;
    const parts = altKey.split('.');
    let val = en;
    for (const p of parts) {
      val = val?.[p];
      if (!val) break;
    }
    if (typeof val === 'string') fallbackValue = val;
  }
  
  setDeep(en, key, fallbackValue);
});

fs.writeFileSync(enPath, JSON.stringify(en, null, 2), 'utf8');
console.log('Updated en.json with missing keys');
