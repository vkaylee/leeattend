---
phase: 3
title: "DeviceListItem Component"
status: pending
effort: 1h
---

# Phase 3: DeviceListItem Component

## Context Links

- Design ref: `/home/elt1541/leeattend/webdashboard/src/pages/DevicesPage.tsx` (card design)
- Types: `/home/elt1541/leeattend/webdashboard/src/lib/api/types.ts` - Device interface
- UI: `/home/elt1541/leeattend/webdashboard/src/components/ui/dropdown-menu.tsx`

## Overview

**Priority:** High
**Status:** Pending

Compact row component for displaying device information nested inside agent accordion cards.

## Key Insights

1. Current DevicesPage uses large cards - need compact row version
2. Enable/disable toggle is primary action
3. Edit/Delete via dropdown menu (less frequent actions)
4. Dim when parent agent is inactive

## Requirements

### Functional
- Display: Name, Brand badge, IP:Port
- Primary action: Enable/Disable toggle button
- Secondary actions: Edit, Delete (dropdown)
- Visual dimming when `parentAgentActive=false`

### Non-Functional
- Compact design for nested view
- Smooth hover effects
- Loading state for toggle action

## Device Interface (from types.ts)

```typescript
interface Device {
  id: string;
  agent_id: string;
  name: string;
  ip: string;
  port: number;
  is_enabled: boolean;
  brand: string;
  last_status: string;
  created_at: string;
  workspace_id: string;
}
```

## Props Interface

```typescript
interface DeviceListItemProps {
  device: Device;
  parentAgentActive?: boolean; // Optional - for nested view
  onUpdate: (id: string, data: Partial<Device>) => Promise<void>;
  onDelete: (id: string) => Promise<void>;
  onEdit?: (device: Device) => void; // Opens edit modal
}
```

## Related Code Files

### Create
- `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/DeviceListItem.tsx`

### Reference
- DevicesPage toggle pattern (lines 167-185)
- Dropdown menu styling (lines 190-212)

## Implementation Steps

1. Create component structure:
   ```typescript
   import { useState } from "react";
   import { useTranslation } from "react-i18next";
   import {
     Cpu,
     Globe,
     Hash,
     MoreVertical,
     Edit2,
     Trash2,
     RefreshCw
   } from "lucide-react";
   import {
     DropdownMenu,
     DropdownMenuContent,
     DropdownMenuItem,
     DropdownMenuTrigger,
   } from "@/components/ui/dropdown-menu";
   import { Device } from "@/lib/api";
   import { cn } from "@/lib/utils";

   interface DeviceListItemProps {
     device: Device;
     parentAgentActive?: boolean;
     onUpdate: (id: string, data: Partial<Device>) => Promise<void>;
     onDelete: (id: string) => Promise<void>;
     onEdit?: (device: Device) => void;
   }
   ```

2. Implement component:
   ```typescript
   export default function DeviceListItem({
     device,
     parentAgentActive = true,
     onUpdate,
     onDelete,
     onEdit
   }: DeviceListItemProps) {
     const { t } = useTranslation();
     const [isToggling, setIsToggling] = useState(false);

     async function handleToggle() {
       if (!confirm(t(device.is_enabled ? "devices.confirm_disable" : "devices.confirm_enable", { name: device.name }))) return;

       setIsToggling(true);
       try {
         await onUpdate(device.id, { is_enabled: !device.is_enabled });
       } finally {
         setIsToggling(false);
       }
     }

     async function handleDelete() {
       if (!confirm(t("devices.toast.delete_confirm"))) return;
       await onDelete(device.id);
     }

     const isDimmed = !parentAgentActive || !device.is_enabled;

     return (
       <div className={cn(
         "flex items-center gap-6 p-4 rounded-2xl transition-all group",
         "bg-slate-50/50 hover:bg-slate-100/50 border border-transparent hover:border-slate-200",
         isDimmed && "opacity-50"
       )}>
         {/* Device icon */}
         <div className={cn(
           "w-10 h-10 rounded-xl flex items-center justify-center",
           "bg-white border border-slate-100 shadow-inner",
           "group-hover:bg-primary-50 group-hover:text-primary-600 transition-colors"
         )}>
           <Cpu size={18} className="text-slate-400 group-hover:text-primary-600" />
         </div>

         {/* Device info */}
         <div className="flex-1 min-w-0">
           <div className="flex items-center gap-2">
             <span className="font-bold text-slate-700 text-sm uppercase tracking-tight truncate">
               {device.name}
             </span>
             <span className="text-[8px] bg-slate-200 px-1.5 py-0.5 rounded font-bold text-slate-500 uppercase tracking-widest shrink-0">
               {device.brand}
             </span>
           </div>
         </div>

         {/* Connection info */}
         <div className="flex items-center gap-3 shrink-0">
           <div className="flex items-center gap-2 text-slate-500 bg-white px-3 py-1.5 rounded-lg border border-slate-100">
             <Globe size={12} className="text-slate-300" />
             <span className="text-[11px] font-bold font-mono tracking-tighter">{device.ip}</span>
           </div>
           <div className="flex items-center gap-1 text-slate-400 font-mono font-bold text-[11px]">
             <Hash size={12} /> {device.port}
           </div>
         </div>

         {/* Status badge */}
         <span className={cn(
           "px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm shrink-0",
           device.is_enabled
             ? "bg-emerald-50 text-emerald-600 border-emerald-100"
             : "bg-slate-100 text-slate-500 border-slate-200"
         )}>
           {device.is_enabled ? t("devices.table.active") : t("devices.table.disabled")}
         </span>

         {/* Toggle button */}
         <button
           onClick={handleToggle}
           disabled={isToggling}
           className={cn(
             "px-3 py-1.5 rounded-xl text-[9px] font-bold uppercase tracking-widest transition-all border shadow-sm",
             isToggling && "opacity-50 cursor-not-allowed",
             device.is_enabled
               ? "text-rose-600 hover:bg-rose-50 border-rose-100 bg-white"
               : "bg-emerald-600 text-white hover:bg-emerald-700 border-transparent"
           )}
         >
           {isToggling ? (
             <RefreshCw size={12} className="animate-spin" />
           ) : device.is_enabled ? (
             t("common.disable")
           ) : (
             t("common.enable")
           )}
         </button>

         {/* Actions dropdown */}
         <DropdownMenu>
           <DropdownMenuTrigger asChild>
             <button className="p-2 text-slate-300 hover:text-primary-600 rounded-lg transition-colors opacity-0 group-hover:opacity-100">
               <MoreVertical size={16} />
             </button>
           </DropdownMenuTrigger>
           <DropdownMenuContent align="end" className="w-48 p-2 rounded-xl">
             {onEdit && (
               <DropdownMenuItem
                 className="gap-3 p-3 rounded-lg font-bold text-xs uppercase"
                 onSelect={() => onEdit(device)}
               >
                 <Edit2 size={14} className="text-slate-500" />
                 {t("devices.table.edit")}
               </DropdownMenuItem>
             )}
             <DropdownMenuItem
               onSelect={handleDelete}
               className="gap-3 p-3 rounded-lg font-bold text-xs uppercase text-rose-600 focus:text-rose-600"
             >
               <Trash2 size={14} />
               {t("devices.table.delete")}
             </DropdownMenuItem>
           </DropdownMenuContent>
         </DropdownMenu>
       </div>
     );
   }
   ```

## Todo List

- [ ] Create DeviceListItem component
- [ ] Implement compact row layout
- [ ] Add enable/disable toggle with loading state
- [ ] Add dropdown menu for edit/delete
- [ ] Handle parent agent inactive dimming
- [ ] Handle device disabled dimming
- [ ] Test confirmation dialogs

## Success Criteria

- [ ] Device info displayed correctly
- [ ] Enable/disable toggle works with loading state
- [ ] Edit/delete actions work via dropdown
- [ ] Visual dimming when parent agent inactive
- [ ] Responsive layout

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Toggle race condition | Disable button during API call |
| Accidental delete | Require confirmation dialog |

## Security Considerations

- Confirmation dialogs for destructive actions
- No new API calls

## Next Steps

After Phase 3, proceed to Phase 4: ConnectivitySettings integration.
