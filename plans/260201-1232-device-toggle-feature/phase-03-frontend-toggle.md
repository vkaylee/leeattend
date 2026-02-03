# Phase 3: Frontend Toggle Button

## Context Links
- [Scout Report](/home/elt1541/leeattend/plans/reports/scout-260201-1225-device-toggle-codebase.md)
- [Plan Overview](./plan.md)
- [Phase 2: Agent](./phase-02-agent-configpush.md)
- [AgentsPage Toggle Pattern](/home/elt1541/leeattend/webdashboard/src/pages/AgentsPage.tsx) (line 53-59)

## Overview

| Property | Value |
|----------|-------|
| Priority | P1 |
| Status | Pending |
| Effort | 1.5 hours |
| Risk | Low |

Add inline toggle button to DevicesPage for quick enable/disable without opening edit modal.

## Key Insights

1. **Existing Pattern:** AgentsPage uses button-based toggle with confirmation dialog (not Switch component)
2. **Current State:** DevicesPage shows `is_enabled` as a badge, but toggle requires edit modal
3. **API Support:** `updateDevice` hook already supports `is_enabled` updates
4. **Role Check:** Need Owner/Admin role validation (via backend)

## Requirements

### Functional
- Inline toggle button next to status badge
- Confirmation dialog before toggle
- Loading state during request
- Success/error toast notifications
- Optimistic update with rollback on error

### Non-Functional
- Follow existing UI patterns (AgentsPage toggle style)
- Accessible (keyboard navigation)
- Mobile responsive

## Related Code Files

### Modify
1. `/home/elt1541/leeattend/webdashboard/src/pages/DevicesPage.tsx`
   - Add `handleToggleDevice` function
   - Add toggle button next to status badge
   - Add loading state for toggle action

### Reference
1. `/home/elt1541/leeattend/webdashboard/src/pages/AgentsPage.tsx` (line 53-59)
   - Toggle pattern with confirmation dialog

## Implementation Steps

### Step 1: Add Toggle Handler Function

Add after line 71 (after `handleDelete`):

```typescript
const [togglingId, setTogglingId] = useState<string | null>(null);

async function handleToggleDevice(device: Device) {
  const action = device.is_enabled ? "disable" : "enable";
  const confirmKey = device.is_enabled ? "devices.confirm_disable" : "devices.confirm_enable";

  if (!confirm(t(confirmKey, { name: device.name }))) return;

  setTogglingId(device.id);
  try {
    await updateDevice(device.id, { is_enabled: !device.is_enabled });
    toast.success(t(device.is_enabled ? "devices.toast.disabled" : "devices.toast.enabled", { name: device.name }));
  } catch (e) {
    console.error(e);
    toast.error(t("devices.toast.toggle_error"));
  } finally {
    setTogglingId(null);
  }
}
```

### Step 2: Add Toggle Button UI

Replace status badge section (line 143-148) with:

```tsx
<div className="flex items-center justify-end gap-4 md:justify-start">
  {/* Status Badge */}
  <span className={cn("px-4 py-1.5 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
    device.is_enabled ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-slate-100 text-slate-500 border-slate-200 opacity-60"
  )}>
    {device.is_enabled ? t("devices.table.active").toUpperCase() : t("devices.table.disabled").toUpperCase()}
  </span>

  {/* Toggle Button */}
  <button
    onClick={() => handleToggleDevice(device)}
    disabled={togglingId === device.id}
    className={cn(
      "px-4 py-1.5 rounded-xl text-[9px] font-bold uppercase tracking-widest transition-all border shadow-sm active:scale-95",
      togglingId === device.id && "opacity-50 cursor-not-allowed",
      device.is_enabled
        ? "text-rose-600 hover:bg-rose-50 border-rose-100 bg-white"
        : "bg-emerald-600 text-white hover:bg-emerald-700 border-transparent shadow-emerald-100"
    )}
  >
    {togglingId === device.id ? (
      <RefreshCw size={12} className="animate-spin" />
    ) : device.is_enabled ? (
      t("common.disable").toUpperCase()
    ) : (
      t("common.enable").toUpperCase()
    )}
  </button>
</div>
```

### Step 3: Add Import for RefreshCw Icon

Check if `RefreshCw` is already imported. If not, add to imports (line 6):

```typescript
import {
  Plus,
  Trash2,
  Server,
  Cpu,
  Globe,
  Hash,
  ShieldCheck,
  ShieldAlert,
  ArrowLeft,
  MoreVertical,
  Settings,
  RefreshCw,  // Add if missing
  Edit2,
  X,
  HardDrive,
  Cpu as BrandIcon
} from "lucide-react";
```

### Step 4: Add Translation Keys

Add to translation files:

**English (`en.json`):**
```json
{
  "devices": {
    "confirm_disable": "Are you sure you want to disable {{name}}? The agent will stop polling this device.",
    "confirm_enable": "Are you sure you want to enable {{name}}? The agent will start polling this device.",
    "toast": {
      "disabled": "{{name}} has been disabled",
      "enabled": "{{name}} has been enabled",
      "toggle_error": "Failed to update device status"
    }
  },
  "common": {
    "disable": "Disable",
    "enable": "Enable"
  }
}
```

**Vietnamese (`vi.json`):**
```json
{
  "devices": {
    "confirm_disable": "Xac nhan tat {{name}}? Agent se ngung dong bo thiet bi nay.",
    "confirm_enable": "Xac nhan bat {{name}}? Agent se bat dau dong bo thiet bi nay.",
    "toast": {
      "disabled": "Da tat {{name}}",
      "enabled": "Da bat {{name}}",
      "toggle_error": "Khong the cap nhat trang thai thiet bi"
    }
  },
  "common": {
    "disable": "Tat",
    "enable": "Bat"
  }
}
```

### Step 5: Add Device Type Import

Ensure Device type is available:

```typescript
import { Device } from "@/lib/api";  // or wherever Device type is defined
```

## Todo List

- [ ] Add `togglingId` state variable
- [ ] Add `handleToggleDevice` function
- [ ] Add toggle button next to status badge
- [ ] Verify `RefreshCw` icon import
- [ ] Add translation keys (en.json, vi.json)
- [ ] Run `npm run dev` to verify no errors
- [ ] Test toggle functionality in browser
- [ ] Verify loading state during request
- [ ] Verify toast notifications

## Success Criteria

- [ ] Toggle button visible in device list
- [ ] Confirmation dialog before toggle
- [ ] Loading spinner during API call
- [ ] Toast notification on success
- [ ] Toast notification on error
- [ ] Device list updates after toggle
- [ ] Button disabled during toggle

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Missing translation keys | Medium | Low | Add keys before testing |
| API error handling | Low | Medium | Try-catch with toast.error |
| Race condition | Low | Low | Disable button during toggle |

## UI/UX Considerations

- Button changes color based on current state (red for disable, green for enable)
- Loading spinner replaces button text during request
- Button disabled during toggle to prevent double-clicks
- Confirmation dialog prevents accidental toggles
- Toast provides feedback on success/error

## Security Considerations

- Backend validates workspace membership via MemberContext
- Only Owner/Admin roles should be able to toggle (handled by backend RBAC)
- No additional frontend validation needed

## Next Steps

After completion, proceed to [Phase 4: Testing](./phase-04-testing.md)
