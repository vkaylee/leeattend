const fs = require('fs');
const path = require('path');

const enPath = path.join(__dirname, 'webdashboard/src/i18n/locales/en.json');
const viPath = path.join(__dirname, 'webdashboard/src/i18n/locales/vi.json');
const zhPath = path.join(__dirname, 'webdashboard/src/i18n/locales/zh.json');

// Restore en.json first (remove my previous fix impact)
// Actually I'll just read it and fix it in memory
let en = JSON.parse(fs.readFileSync(enPath, 'utf8'));

// If I moved things under common, move them back or ensure they are strings where expected
const topLevelKeys = [
  'agents', 'payroll', 'settings', 'leave_requests', 'members', 
  'employees', 'departments', 'attendance_confirm', 'attendance_logs', 
  'attendance_import', 'devices', 'onboarding', 'profile', 'reports', 
  'patterns', 'rules', 'shifts', 'days', 'debug_tools', 'time_tracking', 
  'ess', 'leave_balances'
];

topLevelKeys.forEach(key => {
  if (en.common && en.common[key] && typeof en.common[key] === 'object' && en.common[key].label) {
    en.common[key] = en.common[key].label; // Restore string label
  }
});
fs.writeFileSync(enPath, JSON.stringify(en, null, 2), 'utf8');

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

function smartReconstruct(source, values, prefix = '') {
  let result = {};
  for (const key in source) {
    const fullKey = prefix ? `${prefix}.${key}` : key;
    const sourceValue = source[key];
    
    if (sourceValue && typeof sourceValue === 'object' && !Array.isArray(sourceValue)) {
      result[key] = smartReconstruct(sourceValue, values, fullKey);
    } else {
      // Smart lookup
      let val = values[fullKey];
      if (!val) {
        // Try without common. prefix if present
        if (fullKey.startsWith('common.')) {
          val = values[fullKey.substring(7)];
        } else {
          // Try with common. prefix
          val = values['common.' + fullKey];
        }
      }
      result[key] = val || sourceValue;
    }
  }
  return result;
}

function processFile(filePath) {
  const target = JSON.parse(fs.readFileSync(filePath, 'utf8'));
  const targetValues = getAllValues(target);
  const result = smartReconstruct(en, targetValues);
  fs.writeFileSync(filePath, JSON.stringify(result, null, 2), 'utf8');
  console.log(`Smart Synchronized ${path.basename(filePath)}`);
}

processFile(viPath);
processFile(zhPath);
