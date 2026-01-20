const fs = require('fs');
const path = require('path');

const enPath = path.join(__dirname, 'webdashboard/src/i18n/locales/en.json');
const viPath = path.join(__dirname, 'webdashboard/src/i18n/locales/vi.json');
const zhPath = path.join(__dirname, 'webdashboard/src/i18n/locales/zh.json');

const en = JSON.parse(fs.readFileSync(enPath, 'utf8'));

function getAllValues(obj, prefix = '') {
  let values = {};
  for (const key in obj) {
    const value = obj[key];
    const newPrefix = prefix ? `${prefix}.${key}` : key;
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      Object.assign(values, getAllValues(value, newPrefix));
    } else {
      values[newPrefix] = value;
    }
  }
  return values;
}

function reconstruct(source, values, prefix = '') {
  let result = {};
  for (const key in source) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    const sourceValue = source[key];
    
    if (sourceValue && typeof sourceValue === 'object' && !Array.isArray(sourceValue)) {
      result[key] = reconstruct(sourceValue, values, fullKey);
    } else {
      result[key] = values[fullKey] || sourceValue;
    }
  }
  return result;
}

function processFile(filePath) {
  const target = JSON.parse(fs.readFileSync(filePath, 'utf8'));
  const targetValues = getAllValues(target);
  const result = reconstruct(en, targetValues);
  fs.writeFileSync(filePath, JSON.stringify(result, null, 2), 'utf8');
  console.log(`Synchronized ${path.basename(filePath)} with en.json structure`);
}

processFile(viPath);
processFile(zhPath);
