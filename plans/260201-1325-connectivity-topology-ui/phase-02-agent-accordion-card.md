---
phase: 2
title: "AgentAccordionCard Component"
status: pending
effort: 2h
---

# Phase 2: AgentAccordionCard Component

## Context Links

- Design ref: `/home/elt1541/leeattend/webdashboard/src/pages/AgentsPage.tsx` (table row design)
- Edit modal: `/home/elt1541/leeattend/webdashboard/src/components/features/agents/AgentEditModal.tsx`
- Types: `/home/elt1541/leeattend/webdashboard/src/lib/api/types.ts`

## Overview

**Priority:** High
**Status:** Pending

Expandable card component that displays agent information with nested devices when expanded.

## Key Insights

1. Current AgentsPage uses table rows - convert to card pattern
2. AgentEditModal already exists - reuse for edit action
3. Actions: Edit, Toggle Trust, Delete (from AgentsPage)
4. Visual dimming for inactive/offline agents
5. Chevron icon indicates expand state

## Requirements

### Functional
- Display agent: Name, ID, Status (Trusted/Pending), Last Seen
- Expand/collapse with smooth animation
- Show nested devices when expanded
- Actions: Edit (opens modal), Toggle Trust, Delete
- Visual indicator for device count

### Non-Functional
- Smooth accordion animation (max-height transition)
- Follow existing card styling from DevicesPage
- Responsive design

## Architecture

```
AgentAccordionCard
├── Card header (always visible)
│   ├── Expand chevron
│   ├── Agent icon
│   ├── Agent info (name, id, status badges)
│   ├── Last seen timestamp
│   ├── Device count badge
│   └── Action buttons (hover reveal)
└── Collapsible content
    ├── Divider
    └── DeviceListItem[]
```

## Props Interface

```typescript
interface AgentAccordionCardProps {
  agent: AuthorizedAgent;
  devices: Device[];
  isExpanded: boolean;
  onToggleExpand: () => void;
  // Agent actions
  onEdit: (agent: AuthorizedAgent) => void;
  onToggleTrust: (agent: AuthorizedAgent) => Promise<void>;
  onDelete: (agent: AuthorizedAgent) => Promise<void>;
  // Device actions (passed to children)
  deviceActions: {
    updateDevice: (id: string, data: Partial<Device>) => Promise<void>;
    deleteDevice: (id: string) => Promise<void>;
  };
}
```

## Related Code Files

### Create
- `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/AgentAccordionCard.tsx`

### Reference
- DevicesPage card styling: `bg-white p-8 rounded-[2.5rem] border border-slate-100`
- AgentsPage row: trust toggle, edit, delete buttons
- AgentEditModal: reuse for edit functionality

## Implementation Steps

1. Create component with TypeScript interface:
   ```typescript
   import { useState } from "react";
   import { useTranslation } from "react-i18next";
   import {
     ChevronDown,
     ChevronRight,
     Server,
     Edit2,
     Trash2,
     ShieldCheck,
     ShieldAlert,
     RefreshCw,
     HardDrive
   } from "lucide-react";
   import { AuthorizedAgent, Device } from "@/lib/api";
   import { cn } from "@/lib/utils";
   import DeviceListItem from "./DeviceListItem";
   ```

2. Implement card header:
   ```typescript
   <div
     className={cn(
       "bg-white rounded-[2.5rem] border border-slate-100 overflow-hidden transition-all",
       "hover:shadow-lg hover:shadow-primary-100/20",
       !agent.is_active && "opacity-50 grayscale"
     )}
   >
     {/* Header row - clickable to expand */}
     <div
       onClick={onToggleExpand}
       className="flex items-center gap-6 p-8 cursor-pointer group"
     >
       {/* Chevron */}
       <div className="text-slate-400 group-hover:text-primary-600 transition-colors">
         {isExpanded ? <ChevronDown size={20} /> : <ChevronRight size={20} />}
       </div>

       {/* Agent icon */}
       <div className={cn(
         "w-14 h-14 rounded-2xl flex items-center justify-center shadow-inner border transition-all",
         agent.is_active
           ? "bg-primary-50 text-primary-600 border-primary-100/50 group-hover:scale-110"
           : "bg-slate-100 text-slate-500 border-slate-200"
       )}>
         <Server size={24} />
       </div>

       {/* Agent info */}
       <div className="flex-1">
         <div className="font-bold text-slate-700 text-sm uppercase tracking-tight">
           {agent.name}
         </div>
         <div className="text-[10px] text-slate-500 font-mono font-bold uppercase opacity-70">
           ID: {agent.id}
         </div>
       </div>

       {/* Status badges */}
       <div className="flex items-center gap-2">
         <span className={cn(
           "px-3 py-1 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm",
           agent.is_trusted
             ? "bg-emerald-50 text-emerald-600 border-emerald-100"
             : "bg-amber-50 text-amber-600 border-amber-100"
         )}>
           {agent.is_trusted ? t("common.agents.status.trusted") : t("common.agents.status.pending")}
         </span>
       </div>

       {/* Device count badge */}
       <div className="flex items-center gap-1.5 bg-slate-100 px-3 py-1.5 rounded-lg">
         <HardDrive size={14} className="text-slate-400" />
         <span className="text-xs font-bold text-slate-600">{devices.length}</span>
       </div>

       {/* Last seen */}
       <div className="flex items-center gap-2 text-slate-500 text-[10px] font-mono font-bold uppercase bg-slate-50 px-3 py-1.5 rounded-lg border border-slate-100">
         <RefreshCw size={12} />
         {agent.last_seen_at ? new Date(agent.last_seen_at).toLocaleString() : t("common.never")}
       </div>

       {/* Action buttons - show on hover */}
       <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-all" onClick={e => e.stopPropagation()}>
         <button onClick={() => onEdit(agent)} className="p-3 bg-white text-slate-500 hover:text-primary-600 rounded-xl shadow-md border border-slate-100">
           <Edit2 size={18} />
         </button>
         <button onClick={() => onToggleTrust(agent)} className={cn(
           "px-4 py-2 rounded-xl text-[9px] font-bold uppercase tracking-widest border shadow-sm",
           agent.is_trusted
             ? "text-rose-600 hover:bg-rose-50 border-rose-100 bg-white"
             : "bg-emerald-600 text-white border-transparent"
         )}>
           {agent.is_trusted ? t("common.revoke") : t("common.approve")}
         </button>
         <button onClick={() => onDelete(agent)} className="p-3 bg-white text-slate-500 hover:text-rose-600 rounded-xl shadow-md border border-slate-100">
           <Trash2 size={18} />
         </button>
       </div>
     </div>

     {/* Expandable content */}
     <div className={cn(
       "overflow-hidden transition-all duration-300",
       isExpanded ? "max-h-[2000px] opacity-100" : "max-h-0 opacity-0"
     )}>
       {/* Content */}
     </div>
   </div>
   ```

3. Implement expandable content:
   ```typescript
   <div className={cn(
     "overflow-hidden transition-all duration-300",
     isExpanded ? "max-h-[2000px] opacity-100" : "max-h-0 opacity-0"
   )}>
     <div className="border-t border-slate-100 mx-8" />
     <div className="p-8 pt-6 space-y-3">
       {devices.length === 0 ? (
         <div className="text-center py-8 text-slate-400 text-xs font-bold uppercase tracking-widest">
           {t("devices.no_devices_for_agent")}
         </div>
       ) : (
         devices.map(device => (
           <DeviceListItem
             key={device.id}
             device={device}
             parentAgentActive={agent.is_active}
             {...deviceActions}
           />
         ))
       )}
     </div>
   </div>
   ```

4. Handle visual dimming for inactive agent:
   - Apply `opacity-50 grayscale` to entire card
   - Pass `parentAgentActive={false}` to DeviceListItem

## Todo List

- [ ] Create AgentAccordionCard component
- [ ] Implement card header with all info
- [ ] Add expand/collapse animation
- [ ] Add action buttons with hover reveal
- [ ] Render nested DeviceListItem when expanded
- [ ] Handle inactive agent visual dimming
- [ ] Add empty state for agents with no devices
- [ ] Wire up edit modal integration

## Success Criteria

- [ ] Card expands/collapses smoothly
- [ ] All agent info displayed correctly
- [ ] Actions (edit, trust, delete) work
- [ ] Nested devices render when expanded
- [ ] Inactive agents visually dimmed

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Animation jank | Use CSS transitions, avoid JS-based animation |
| Click propagation | stopPropagation on action buttons |

## Security Considerations

- Confirm dialogs for destructive actions (delete, revoke trust)
- No new API surface

## Next Steps

After Phase 2, proceed to Phase 3: DeviceListItem implementation.
