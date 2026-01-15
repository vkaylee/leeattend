### # Implementation Plan: Multi-Language Support (i18n)

### ## Approach
- **Why this solution**: Implements "Approach B: Full-stack Integrated i18n". This ensures language preferences are saved in the database, allowing for a consistent experience across different devices and enabling future server-side localized communications (emails, notifications).
- **Alternatives considered**: Client-side only storage (LocalStorage) was rejected to avoid settings "resetting" when users switch browsers or clear cache.

### ## Steps

#### 1. Database & Backend Implementation (20 min)
- **Migration**: Create a new migration file to add `language` columns.
  - `workspaces`: `default_language` (VARCHAR(10) DEFAULT 'vi')
  - `users`: `preferred_language` (VARCHAR(10) NULL)
- **Rust Models**: Update `Workspace` and `User` structs in `apiserver/src/models.rs` to include the new fields.
- **Handlers**: Update workspace and user update handlers to accept and persist the language preference.

#### 2. Frontend Infrastructure Setup (15 min)
- **Dependencies**:
  ```bash
  cd webdashboard && npm install i18next react-i18next i18next-browser-languagedetector
  ```
- **Locales**: Create `webdashboard/src/i18n/locales/` with `en.json`, `vi.json`, and `zh.json`.
- **Config**: Initialize `i18next` in `webdashboard/src/i18n/config.ts` and import it in `main.tsx`.

#### 3. Core Logic & Sync (20 min)
- **Sync Hook**: Create or update a hook to synchronize the `i18next` language with the authenticated user's `preferred_language` or the workspace's `default_language`.
- **API Update**: Update `webdashboard/src/lib/api.ts` types and functions to support the new language fields.

#### 4. UI Integration (25 min)
- **Workspace Settings**: Add a "Ngôn ngữ mặc định" (Default Language) Combobox in `WorkspaceSettingsPage.tsx`.
- **Header/Profile**: Add a quick language switcher in the Header or User Profile menu for personal override.
- **Translation**: Begin wrapping core UI strings (Sidebar, Dashboard titles) with the `t()` function.

#### 5. Testing & Verification (15 min)
- **Verification**: 
  - Change workspace default -> verify UI updates for all (if no user override).
  - Change user preference -> verify it overrides workspace default.
  - Refresh page -> verify preference persists.
- **UI Audit**: Check for layout shifts or text overflow in English/Chinese vs Vietnamese.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Backend / DB Migration | 20 min |
| Infrastructure Setup | 15 min |
| Core Logic & Sync | 20 min |
| UI Integration | 25 min |
| Testing & Verification | 15 min |
| **Total** | **~1.5 hours** |

### ## Rollback Plan
1. Revert database migrations using `sqlx migrate revert`.
2. Revert code changes via `git checkout`.
3. Remove installed npm packages if necessary.

### ## Security Checklist
- [ ] Validate language codes (e.g., whitelist 'en', 'vi', 'zh') on the backend to prevent invalid data.
- [ ] Ensure RBAC checks for updating workspace-wide language settings.

---
**File created:** `plans/multi-language-support-20260115.md`
