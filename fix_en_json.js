const fs = require('fs');
const path = require('path');

const enPath = path.join(__dirname, 'webdashboard/src/i18n/locales/en.json');
let en = JSON.parse(fs.readFileSync(enPath, 'utf8'));

const keysToMove = [
  'agents',
  'payroll',
  'settings',
  'leave_requests',
  'members',
  'employees',
  'departments',
  'attendance_confirm',
  'attendance_logs',
  'attendance_import',
  'devices',
  'onboarding',
  'profile',
  'reports',
  'patterns',
  'rules',
  'shifts',
  'days',
  'debug_tools',
  'time_tracking',
  'ess',
  'leave_balances'
];

if (!en.common) en.common = {};

keysToMove.forEach(key => {
  if (en[key] && typeof en[key] === 'object') {
    // If common[key] exists and is a string, move it to common[key].title or something?
    // Looking at the code, it seems common.agents is expected to be an object.
    if (typeof en.common[key] === 'string') {
       // Keep the string as 'label' or something? 
       // Actually, let's see what the code expects.
       // It uses t("common.agents.edit_modal.title")
       // So en.common.agents must be an object.
       const label = en.common[key];
       en.common[key] = en[key];
       en.common[key].label = label;
    } else {
       en.common[key] = en[key];
    }
    // We should keep it at top level too for now, to avoid breaking other things
    // delete en[key]; 
  }
});

fs.writeFileSync(enPath, JSON.stringify(en, null, 2), 'utf8');
console.log('Fixed en.json structure by moving keys under common');
