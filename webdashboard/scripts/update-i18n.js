import fs from 'fs';
import path from 'path';

const localesDir = path.resolve(process.cwd(), 'src/i18n/locales');
const languages = ['en', 'vi', 'zh'];

const attendanceMissingKeys = {
  "ot_approvals": {
    "subtitle": "Manage overtime approvals",
    "title": "OT Approvals",
    "actions": {
      "approve": "Approve",
      "reject": "Reject"
    },
    "badges": {
      "paper": "Paper",
      "self": "Self"
    },
    "empty_state": {
      "description": "No pending OT requests",
      "title": "All caught up!"
    },
    "table": {
      "actions": "Actions",
      "claimed": "Claimed",
      "date": "Date",
      "employee": "Employee",
      "reason": "Reason",
      "source": "Source",
      "system": "System"
    },
    "tabs": {
      "post_ot": "Post-OT",
      "pre_ot": "Pre-OT"
    }
  }
};

const employeesMissingKeys = {
  "ess": {
    "ot": {
      "no_assignments": "No assignments",
      "no_assignments_desc": "No overtime assignments.",
      "no_quota": "No quota",
      "quota_subtitle": "OT Quota",
      "quota_title": "OT Quota",
      "request": "Request OT",
      "request_success": "Success",
      "request_title": "Request OT",
      "subtitle": "Manage OT",
      "title": "Overtime"
    }
  }
};

const settingsMissingKeys = {
  "settings": {
    "categories": {
      "overtime": "Overtime",
      "overtime_desc": "Overtime settings"
    },
    "overtime": {
      "delete_config_desc": "Delete?",
      "delete_workflow_desc": "Delete?",
      "quotas_desc": "Quotas",
      "tabs": {
        "quotas": "Quotas",
        "workflows": "Workflows"
      },
      "workflows_desc": "Workflows"
    }
  }
};

const commonMissingKeys = {
  "submitting": "Submitting...",
  "key": "Key",
  "scope": "Scope",
  "value": "Value"
};

function deepMerge(target, source) {
  for (const key of Object.keys(source)) {
    if (source[key] instanceof Object && !Array.isArray(source[key]) && key in target) {
      Object.assign(source[key], deepMerge(target[key], source[key]));
    }
  }
  Object.assign(target || {}, source);
  return target;
}

for (const lang of languages) {
  // 1. Update attendance.json
  const attPath = path.join(localesDir, lang, 'attendance.json');
  if (fs.existsSync(attPath)) {
    const data = JSON.parse(fs.readFileSync(attPath, 'utf8'));
    deepMerge(data, attendanceMissingKeys);
    fs.writeFileSync(attPath, JSON.stringify(data, null, 2) + '\n');
  }

  // 2. Update employees.json
  const empPath = path.join(localesDir, lang, 'employees.json');
  if (fs.existsSync(empPath)) {
    const data = JSON.parse(fs.readFileSync(empPath, 'utf8'));
    deepMerge(data, employeesMissingKeys);
    fs.writeFileSync(empPath, JSON.stringify(data, null, 2) + '\n');
  }

  // 3. Update settings.json
  const setPath = path.join(localesDir, lang, 'settings.json');
  if (fs.existsSync(setPath)) {
    const data = JSON.parse(fs.readFileSync(setPath, 'utf8'));
    deepMerge(data, settingsMissingKeys);
    fs.writeFileSync(setPath, JSON.stringify(data, null, 2) + '\n');
  }

  // 4. Update common.json (clean up the wrongly added top-level ones)
  const comPath = path.join(localesDir, lang, 'common.json');
  if (fs.existsSync(comPath)) {
    const data = JSON.parse(fs.readFileSync(comPath, 'utf8'));
    delete data.ot_approvals;
    delete data.ess;
    delete data.settings;
    if (!data.common) data.common = {};
    deepMerge(data.common, commonMissingKeys);
    fs.writeFileSync(comPath, JSON.stringify(data, null, 2) + '\n');
  }
  
  console.log(`Updated files for ${lang}`);
}
