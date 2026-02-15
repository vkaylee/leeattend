const fs = require('fs');
const path = require('path');

const enPath = path.join(__dirname, 'webdashboard/src/i18n/locales/en.json');
const viPath = path.join(__dirname, 'webdashboard/src/i18n/locales/vi.json');

const en = JSON.parse(fs.readFileSync(enPath, 'utf8'));
const vi = JSON.parse(fs.readFileSync(viPath, 'utf8'));

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

const viValues = getAllValues(vi);

function merge(source, values) {
  let result = {};
  for (const key in source) {
    const value = source[key];
    if (value && typeof value === 'object' && !Array.isArray(value)) {
      result[key] = merge(value, values);
    } else {
      // Find value in flat map
      // Check if it exists in viValues
      // We need to match the key path
      // But wait, viValues has keys like 'common.dashboard'
      // If we are at top level 'common' and key is 'dashboard', prefix is ''
      // Actually we need to pass prefix to merge
    }
  }
}

// Easier approach: flat merge
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

const newVi = reconstruct(en, viValues);

fs.writeFileSync(viPath, JSON.stringify(newVi, null, 2), 'utf8');
console.log('Merged vi.json with en.json structure');
