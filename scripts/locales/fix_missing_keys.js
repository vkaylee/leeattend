const fs = require('fs');
const path = require('path');

const localesDir = 'webdashboard/src/i18n/locales';
const files = ['vi.json', 'en.json', 'zh.json'];

const missingKeys = {
  'common': {
    'month_short': {
      'vi': 'Tháng',
      'en': 'Mo',
      'zh': '月'
    }
  },
  'payroll.settings': {
    'policy_desc': {
      'vi': 'Quản lý đơn vị công chuẩn và sự thay đổi theo thời gian.',
      'en': 'Manage standard work units and changes over time.',
      'zh': '管理标准工作单位及其随时间的变化。'
    },
    'add_version': {
      'vi': 'Tạo phiên bản chính sách mới',
      'en': 'Create new policy version',
      'zh': '创建新政策版本'
    },
    'date_warning': {
      'vi': 'Lưu ý: Ngày có hiệu lực quyết định các kỳ lương bị ảnh hưởng. Các thay đổi sẽ tự động áp dụng từ ngày này.',
      'en': 'Note: Effective date determines the payroll periods affected. Changes apply automatically from this date.',
      'zh': '注意：生效日期决定受影响的薪资周期。更改将从此日期起自动应用。'
    },
    'status': {
      'vi': 'Trạng thái',
      'en': 'Status',
      'zh': '状态'
    },
    'virtual': {
      'vi': 'Mặc định hệ thống',
      'en': 'System Default',
      'zh': '系统默认'
    }
  },
  'employees': {
    'hire_date': {
      'vi': 'Ngày vào làm',
      'en': 'Hire Date',
      'zh': '入职日期'
    }
  },
  'days': {
    'monday_short': { 'vi': 'T2', 'en': 'Mon', 'zh': '一' },
    'tuesday_short': { 'vi': 'T3', 'en': 'Tue', 'zh': '二' },
    'wednesday_short': { 'vi': 'T4', 'en': 'Wed', 'zh': '三' },
    'thursday_short': { 'vi': 'T5', 'en': 'Thu', 'zh': '四' },
    'friday_short': { 'vi': 'T6', 'en': 'Fri', 'zh': '五' },
    'saturday_short': { 'vi': 'T7', 'en': 'Sat', 'zh': '六' },
    'sunday_short': { 'vi': 'CN', 'en': 'Sun', 'zh': '日' }
  }
};

files.forEach(file => {
  const filePath = path.join(localesDir, file);
  const lang = file.split('.')[0];
  const content = JSON.parse(fs.readFileSync(filePath, 'utf8'));

  Object.entries(missingKeys).forEach(([section, keys]) => {
    const parts = section.split('.');
    let target = content;
    parts.forEach(part => {
      if (!target[part]) target[part] = {};
      target = target[part];
    });

    Object.entries(keys).forEach(([key, values]) => {
      if (!target[key]) {
        target[key] = values[lang];
      }
    });
  });

  fs.writeFileSync(filePath, JSON.stringify(content, null, 2) + '\n');
  console.log(`Updated ${file}`);
});