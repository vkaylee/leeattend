# Phase 3: Frontend ESS Module

**Priority**: High
**Status**: Pending
**Estimated Effort**: 6-8 hours

---

## Context Links

- Pages dir: `webdashboard/src/pages/`
- Components: `webdashboard/src/components/`
- API client: `webdashboard/src/lib/api/`
- Hooks: `webdashboard/src/hooks/`

## Overview

Create employee self-service frontend module with profile, documents, payslips, and attendance views.

## Key Insights

- Follow existing UI patterns (Modal, tables, forms)
- Use existing design system (Tailwind + primary colors)
- i18n support required (vi, en, zh)
- React Query for data fetching (existing pattern)

## Requirements

### Functional
- My Profile page with edit capability
- Emergency contacts section (add/remove)
- Document upload/download UI
- Payslips list with download
- Attendance history view
- Leave balance display

### Non-Functional
- Mobile-responsive design
- Lazy loading for documents list
- Toast notifications for actions
- Loading states for all data

## Architecture

```
webdashboard/src/
├── pages/ess/
│   ├── MyProfilePage.tsx
│   ├── MyDocumentsPage.tsx
│   ├── MyPayslipsPage.tsx
│   ├── MyAttendancePage.tsx
│   └── MyLeavePage.tsx
├── components/ess/
│   ├── ProfileForm.tsx
│   ├── EmergencyContactCard.tsx
│   ├── DocumentUploader.tsx
│   ├── PayslipCard.tsx
│   └── AttendanceCalendar.tsx
├── hooks/
│   └── use-ess.ts
└── lib/api/
    └── ess.ts
```

## Implementation Steps

### 1. Create API Client
```typescript
// lib/api/ess.ts
export const getMyProfile = () => apiFetch('/api/ess/profile');
export const updateMyProfile = (data) => apiFetch('/api/ess/profile', { method: 'PUT', body: data });
export const getMyDocuments = () => apiFetch('/api/ess/documents');
export const uploadDocument = (formData) => apiFetch('/api/ess/documents', { method: 'POST', body: formData });
export const getMyPayslips = () => apiFetch('/api/ess/payslips');
export const getMyAttendance = (params) => apiFetch('/api/ess/attendance?' + new URLSearchParams(params));
```

### 2. Create React Query Hooks
```typescript
// hooks/use-ess.ts
export function useMyProfile() { ... }
export function useMyDocuments() { ... }
export function useMyPayslips() { ... }
export function useMyAttendance(dateRange) { ... }
```

### 3. Create Pages

#### MyProfilePage
- Personal info form (phone, address, DOB)
- Emergency contacts section
- Bank details section (masked display)
- Edit mode toggle

#### MyDocumentsPage
- Upload dropzone
- Documents list with type badges
- Download/delete actions
- Expiry warnings

#### MyPayslipsPage
- Monthly payslips list
- Period filter (year selector)
- Download PDF button
- Earnings/deductions summary

#### MyAttendancePage
- Calendar view with daily records
- Date range filter
- Punch in/out times
- Hours worked summary

#### MyLeavePage
- Leave balance cards (by type)
- Request history table
- "New Request" button (opens modal)
- Pending requests highlight

### 4. Add Routes
```typescript
// In router config
{ path: '/ess/profile', element: <MyProfilePage /> }
{ path: '/ess/documents', element: <MyDocumentsPage /> }
{ path: '/ess/payslips', element: <MyPayslipsPage /> }
{ path: '/ess/attendance', element: <MyAttendancePage /> }
{ path: '/ess/leave', element: <MyLeavePage /> }
```

### 5. Add Sidebar Navigation
- New "My Workspace" section for employees
- Icons: User, FileText, DollarSign, Clock, Calendar

## Todo List

- [ ] Create lib/api/ess.ts
- [ ] Create hooks/use-ess.ts
- [ ] Create pages/ess/MyProfilePage.tsx
- [ ] Create components/ess/ProfileForm.tsx
- [ ] Create components/ess/EmergencyContactCard.tsx
- [ ] Create pages/ess/MyDocumentsPage.tsx
- [ ] Create components/ess/DocumentUploader.tsx
- [ ] Create pages/ess/MyPayslipsPage.tsx
- [ ] Create pages/ess/MyAttendancePage.tsx
- [ ] Create pages/ess/MyLeavePage.tsx
- [ ] Add routes to router config
- [ ] Add sidebar menu items
- [ ] Add i18n keys (vi, en, zh)

## Success Criteria

- All pages render correctly
- Forms validate input
- File upload shows progress
- Data loads with proper loading states
- Mobile responsive layout

## Security Considerations

- No sensitive data (bank full number) displayed unmasked
- Confirm before delete actions
- Session timeout handling

## Related Code Files

- Create: `webdashboard/src/pages/ess/*.tsx`
- Create: `webdashboard/src/components/ess/*.tsx`
- Create: `webdashboard/src/hooks/use-ess.ts`
- Create: `webdashboard/src/lib/api/ess.ts`
- Modify: `webdashboard/src/routes.tsx`
- Modify: `webdashboard/src/components/Sidebar.tsx`
- Modify: `webdashboard/src/i18n/locales/*.json`

## Next Steps

After frontend pages ready, proceed to Phase 4: Document Management
