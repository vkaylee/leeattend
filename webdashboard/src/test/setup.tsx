import "@testing-library/jest-dom";
import { vi } from "vitest";

// Mock react-i18next
vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => {
      // Return Vietnamese strings for existing tests to pass
      const translations: Record<string, string> = {
        'workspace_settings.title': 'Cài đặt Workspace',
        'workspace_settings.save_config': 'Lưu cấu hình',
        'workspace_settings.invite_member': 'Gửi lời mời',
        'workspace_settings.invite_email_placeholder': 'dong-nghiep@congty.com',
        'workspace_settings.invite_role_label': 'Vai trò',
        'workspace_settings.member_manage_roles': 'Quản lý quyền',
        'role_management.title': 'Quản lý quyền hạn',
        'common.one_minute': '1 phút',
        'common.fifteen_minutes': '15 phút',
        'common.thirty_minutes': '30 phút',
        'common.sixty_minutes': '60 phút',
        'Email': 'Email',
        'Vai trò': 'Vai trò',
      };
      return translations[key] || key;
    },
    i18n: {
      changeLanguage: () => Promise.resolve(),
      language: 'vi',
    },
  }),
  initReactI18next: {
    type: '3rdParty',
    init: () => {},
  },
}));

// Mock Combobox to be a simple select for existing tests to work
vi.mock("@/components/ui/Combobox", () => ({
  Combobox: ({ 
    options, 
    value, 
    onChange, 
    id, 
    "data-testid": testId 
  }: any) => (
    <select
      id={id}
      data-testid={testId}
      value={value}
      onChange={(e) => onChange(e.target.value)}
    >
      {options.map((opt: any) => (
        <option key={opt.value} value={opt.value}>
          {opt.label}
        </option>
      ))}
    </select>
  ),
}));
