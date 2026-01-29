# Phase 03: Frontend Entry Form

## Context Links
- [Phase 02 - Backend API](/home/elt1541/leeattend/plans/260128-2113-ot-paper-form-digitization/phase-02-backend-api.md)
- [Existing ClaimOtDialog](/home/elt1541/leeattend/webdashboard/src/components/ess/ClaimOtDialog.tsx)
- [EmployeeSearch Component](/home/elt1541/leeattend/webdashboard/src/components/common/EmployeeSearch.tsx)
- [OtApprovalsPage](/home/elt1541/leeattend/webdashboard/src/pages/approvals/OtApprovalsPage.tsx)

## Overview
- **Priority**: P1
- **Status**: Pending
- **Effort**: 4 hours

Create dialog component for department heads to enter paper OT claims.

## Key Insights

From existing code:
- `ClaimOtDialog.tsx` is good template (same form fields, different flow)
- `EmployeeSearch.tsx` provides employee autocomplete with debounce
- Dialog pattern uses Radix UI `Dialog` component
- Form validation done inline with state
- React Query mutation for API calls

Key differences from ClaimOtDialog:
- Employee selector (not pre-selected)
- No max validation against system detected (paper form = trust)
- Photo upload optional
- Different API endpoint

## Requirements

### Functional
- Employee dropdown (searchable, filtered by workspace)
- Date picker (defaults to today, allow past dates)
- Minutes input (number, required, > 0)
- Reason textarea (required)
- Photo upload (optional, image preview)
- Submit button calls new API endpoint
- Success closes dialog, shows toast
- Error shows inline message

### Non-Functional
- Match existing UI patterns (Radix Dialog, Tailwind styles)
- Mobile responsive (form stacks vertically)
- Loading states during submission

## Component Design

### File Structure

```
webdashboard/src/
├── components/
│   └── approvals/
│       └── EnterPaperOtDialog.tsx  # New component
├── hooks/
│   └── use-attendance.ts           # Add new hook
├── lib/api/
│   └── attendance.ts               # Add API function
│   └── types.ts                    # Add new types
└── pages/approvals/
    └── OtApprovalsPage.tsx         # Add entry button
```

### EnterPaperOtDialog Component

```tsx
// webdashboard/src/components/approvals/EnterPaperOtDialog.tsx

import { useState } from "react";
import * as Dialog from "@radix-ui/react-dialog";
import { X, Upload, FileText, Calendar } from "lucide-react";
import { useTranslation } from "react-i18next";
import { EmployeeSearch } from "@/components/common/EmployeeSearch";
import { useEnterPaperOt } from "@/hooks/use-attendance";
import { getCurrentWorkspaceId } from "@/lib/api/helpers";
import { format } from "date-fns";

interface Props {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSuccess?: () => void;
}

export function EnterPaperOtDialog({ open, onOpenChange, onSuccess }: Props) {
  const { t } = useTranslation();
  const wsId = getCurrentWorkspaceId() || "";
  const { enterPaperOt, isSubmitting } = useEnterPaperOt();

  // Form state
  const [employeeId, setEmployeeId] = useState<string>("");
  const [date, setDate] = useState(format(new Date(), "yyyy-MM-dd"));
  const [minutes, setMinutes] = useState(0);
  const [reason, setReason] = useState("");
  const [photoUrl, setPhotoUrl] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const resetForm = () => {
    setEmployeeId("");
    setDate(format(new Date(), "yyyy-MM-dd"));
    setMinutes(0);
    setReason("");
    setPhotoUrl(null);
    setError(null);
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    // Validation
    if (!employeeId) {
      setError(t("approvals.paper_ot.error_no_employee") || "Select an employee");
      return;
    }
    if (minutes <= 0) {
      setError(t("approvals.paper_ot.error_minutes") || "Minutes must be > 0");
      return;
    }
    if (!reason.trim()) {
      setError(t("approvals.paper_ot.error_reason") || "Reason is required");
      return;
    }

    try {
      await enterPaperOt({
        employeeId,
        date,
        claimed_minutes: minutes,
        reason,
        paper_form_photo_url: photoUrl || undefined,
      });
      resetForm();
      onOpenChange(false);
      onSuccess?.();
    } catch (e: any) {
      setError(e.message || "Failed to submit");
    }
  };

  return (
    <Dialog.Root open={open} onOpenChange={(o) => { if (!o) resetForm(); onOpenChange(o); }}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 bg-black/40 z-50 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-lg -translate-x-1/2 -translate-y-1/2 border border-slate-100 bg-white p-6 shadow-xl rounded-[2rem]">
          <Dialog.Title className="text-xl font-bold text-slate-800 uppercase tracking-tight flex items-center gap-2">
            <FileText className="w-5 h-5" />
            {t("approvals.paper_ot.title") || "Enter Paper OT Form"}
          </Dialog.Title>
          <Dialog.Description className="text-sm text-slate-500 mt-1">
            {t("approvals.paper_ot.description") || "Enter overtime claim from signed paper form"}
          </Dialog.Description>

          <button onClick={() => onOpenChange(false)} className="absolute right-4 top-4 p-2 rounded-full hover:bg-slate-100">
            <X className="h-4 w-4" />
          </button>

          <form onSubmit={handleSubmit} className="mt-6 space-y-5">
            {/* Employee Selector */}
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em]">
                {t("common.employee")}
              </label>
              <EmployeeSearch
                workspaceId={wsId}
                value={employeeId}
                onChange={setEmployeeId}
                placeholder={t("approvals.paper_ot.select_employee") || "Search employee..."}
              />
            </div>

            {/* Date Picker */}
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em]">
                {t("common.date")}
              </label>
              <div className="relative">
                <Calendar className="absolute left-3 top-3 h-4 w-4 text-slate-400" />
                <input
                  type="date"
                  value={date}
                  onChange={(e) => setDate(e.target.value)}
                  max={format(new Date(), "yyyy-MM-dd")}
                  className="w-full pl-10 h-10 rounded-xl border border-slate-200 bg-slate-50 px-3 text-sm focus:ring-2 focus:ring-primary-600"
                />
              </div>
            </div>

            {/* Minutes Input */}
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em]">
                {t("approvals.paper_ot.minutes") || "OT Minutes"}
              </label>
              <input
                type="number"
                value={minutes || ""}
                onChange={(e) => setMinutes(parseInt(e.target.value) || 0)}
                min={1}
                className="w-full h-10 rounded-xl border border-slate-200 bg-slate-50 px-3 text-sm font-mono focus:ring-2 focus:ring-primary-600"
                placeholder="120"
              />
            </div>

            {/* Reason Textarea */}
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em]">
                {t("common.reason")}
              </label>
              <textarea
                value={reason}
                onChange={(e) => setReason(e.target.value)}
                rows={3}
                className="w-full rounded-xl border border-slate-200 bg-slate-50 px-3 py-2 text-sm resize-none focus:ring-2 focus:ring-primary-600"
                placeholder={t("approvals.paper_ot.reason_placeholder") || "Paper form #1234 - Production line"}
              />
            </div>

            {/* Photo Upload (Optional) */}
            <div className="space-y-2">
              <label className="text-[10px] font-bold text-slate-500 uppercase tracking-[0.2em]">
                {t("approvals.paper_ot.photo") || "Paper Form Photo (Optional)"}
              </label>
              <div className="border-2 border-dashed border-slate-200 rounded-xl p-4 text-center hover:border-primary-400 transition-colors cursor-pointer">
                <Upload className="w-8 h-8 text-slate-400 mx-auto mb-2" />
                <p className="text-xs text-slate-500">{t("common.click_to_upload") || "Click to upload"}</p>
              </div>
            </div>

            {/* Error Message */}
            {error && (
              <div className="text-xs font-bold text-rose-500 uppercase tracking-wide bg-rose-50 p-3 rounded-lg">
                {error}
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex justify-end gap-3 pt-2">
              <button
                type="button"
                onClick={() => onOpenChange(false)}
                className="px-4 py-2 rounded-xl text-xs font-bold uppercase tracking-wider text-slate-600 hover:bg-slate-100"
              >
                {t("common.cancel")}
              </button>
              <button
                type="submit"
                disabled={isSubmitting}
                className="px-6 py-2 rounded-xl bg-primary-600 text-white text-xs font-bold uppercase tracking-wider shadow-lg hover:bg-primary-700 disabled:opacity-50"
              >
                {isSubmitting ? t("common.saving") : t("common.submit")}
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
}
```

### API Function

```typescript
// Add to webdashboard/src/lib/api/attendance.ts

export async function enterPaperOt(
  wsId: string,
  employeeId: string,
  data: {
    date: string;
    claimed_minutes: number;
    reason: string;
    allow_exceed_system_detected?: boolean;
    paper_form_photo_url?: string;
  }
): Promise<AttendanceDailySummary> {
  const response = await apiFetch(
    `${API_BASE_URL}/workspaces/${wsId}/attendance/enter-paper-ot/${employeeId}`,
    {
      method: "POST",
      body: JSON.stringify(data),
    }
  );
  if (!response.ok) {
    const err = await response.text();
    throw new Error(err || "Failed to enter paper OT");
  }
  return response.json();
}
```

### React Query Hook

```typescript
// Add to webdashboard/src/hooks/use-attendance.ts

export function useEnterPaperOt() {
  const queryClient = useQueryClient();
  const wsId = getCurrentWorkspaceId();

  const mutation = useMutation({
    mutationFn: (data: {
      employeeId: string;
      date: string;
      claimed_minutes: number;
      reason: string;
      paper_form_photo_url?: string;
    }) => {
      if (!wsId) throw new Error("No workspace");
      return enterPaperOt(wsId, data.employeeId, {
        date: data.date,
        claimed_minutes: data.claimed_minutes,
        reason: data.reason,
        paper_form_photo_url: data.paper_form_photo_url,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: attendanceKeys.all });
    },
  });

  return {
    enterPaperOt: mutation.mutateAsync,
    isSubmitting: mutation.isPending,
  };
}
```

### TypeScript Types

```typescript
// Add to webdashboard/src/lib/api/types.ts

// Update AttendanceDailySummary interface
export interface AttendanceDailySummary {
  id: string;
  workspace_id: string;
  employee_id: string;
  date: string;
  actual_work_minutes: number;
  system_detected_ot_minutes: number;
  claimed_ot_minutes: number;
  approved_ot_minutes: number;
  ot_status: OtStatus;
  ot_reason?: string;
  manager_note?: string;
  // New fields for paper form
  claim_entered_by_user_id?: string;
  claim_entered_at?: string;
  paper_form_photo_url?: string;
  entered_via?: "self_service" | "paper_form";
  created_at: string;
  updated_at: string;
}
```

## Related Code Files

### To Modify
- `/home/elt1541/leeattend/webdashboard/src/lib/api/attendance.ts` - Add API function
- `/home/elt1541/leeattend/webdashboard/src/lib/api/types.ts` - Update types
- `/home/elt1541/leeattend/webdashboard/src/hooks/use-attendance.ts` - Add hook
- `/home/elt1541/leeattend/webdashboard/src/pages/approvals/OtApprovalsPage.tsx` - Add button

### To Create
- `/home/elt1541/leeattend/webdashboard/src/components/approvals/EnterPaperOtDialog.tsx`

## Implementation Steps

1. [ ] Update `AttendanceDailySummary` type in `types.ts`
2. [ ] Add `enterPaperOt` function in `attendance.ts`
3. [ ] Add `useEnterPaperOt` hook in `use-attendance.ts`
4. [ ] Create `EnterPaperOtDialog.tsx` component
5. [ ] Add i18n keys to `en.json` and `vi.json`
6. [ ] Add "Enter Paper Form" button to `OtApprovalsPage.tsx`
7. [ ] Test form submission

## i18n Keys to Add

```json
// en.json
{
  "approvals": {
    "paper_ot": {
      "title": "Enter Paper OT Form",
      "description": "Enter overtime claim from signed paper form",
      "select_employee": "Search employee...",
      "minutes": "OT Minutes",
      "reason_placeholder": "Paper form #1234 - Production line",
      "photo": "Paper Form Photo (Optional)",
      "error_no_employee": "Please select an employee",
      "error_minutes": "Minutes must be greater than 0",
      "error_reason": "Reason is required",
      "btn_enter": "Enter Paper Form"
    }
  }
}
```

```json
// vi.json
{
  "approvals": {
    "paper_ot": {
      "title": "Nhap Don OT Giay",
      "description": "Nhap yeu cau tang ca tu don giay da ky",
      "select_employee": "Tim nhan vien...",
      "minutes": "Phut Tang Ca",
      "reason_placeholder": "Don #1234 - Day chuyen san xuat",
      "photo": "Anh Don Giay (Tuy Chon)",
      "error_no_employee": "Vui long chon nhan vien",
      "error_minutes": "So phut phai lon hon 0",
      "error_reason": "Ly do la bat buoc",
      "btn_enter": "Nhap Don Giay"
    }
  }
}
```

## Todo List

- [ ] Create dialog component
- [ ] Add API function
- [ ] Add React Query hook
- [ ] Update TypeScript types
- [ ] Add i18n translations
- [ ] Integrate with OtApprovalsPage
- [ ] Test form flow

## Success Criteria

- [ ] Dialog opens from OtApprovalsPage
- [ ] Employee search works
- [ ] Form validates required fields
- [ ] Submit creates claim with status `pending_manager`
- [ ] Dialog closes on success
- [ ] New claim appears in list

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Employee search slow | Low | Debounce already implemented |
| Photo upload fails | Low | Photo is optional |
| Form validation bypass | Medium | Server-side validation as backup |

## Security Considerations

- Employee ID from dropdown (not user input)
- Date restricted to past/today (no future OT)
- CSRF protection via existing auth token

## Next Steps

After this phase:
- Phase 04 integrates display changes to OtApprovalsPage
- Phase 05 tests full workflow
