const fs = require('fs');

function set(obj, path, value) {
    const keys = path.split('.');
    let current = obj;
    for (let i = 0; i < keys.length - 1; i++) {
        if (!current[keys[i]]) current[keys[i]] = {};
        current = current[keys[i]];
    }
    current[keys[keys.length - 1]] = value;
}

const locales = ['en', 'vi', 'zh'];
const keys = [
  'employees.policies.add_condition',
  'employees.policies.type_label',
  'employees.policies.conditions_title',
  'employees.policies.conditions_desc',
  'common.delete_confirm',
  'employees.policies.delete_warning',
  'employees.policies.no_conditions',
  'common.present',
  'rules.confirm_delete_msg',
  'settings.categories.leave_desc',
  'system_roles.messages.assign_success'
];

for (const locale of locales) {
    const file = './src/i18n/locales/' + locale + '.json';
    const data = JSON.parse(fs.readFileSync(file, 'utf8'));
    for (const key of keys) {
        set(data, key, key);
    }
    fs.writeFileSync(file, JSON.stringify(data, null, 2) + '\n');
}
