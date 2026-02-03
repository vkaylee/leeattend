---
phase: 1
title: "ConnectivityTopology Component"
status: pending
effort: 1.5h
---

# Phase 1: ConnectivityTopology Component

## Context Links

- Current: `/home/elt1541/leeattend/webdashboard/src/pages/settings/ConnectivitySettings.tsx`
- Hook: `/home/elt1541/leeattend/webdashboard/src/hooks/use-agents.ts`
- Hook: `/home/elt1541/leeattend/webdashboard/src/hooks/use-devices.ts`
- Types: `/home/elt1541/leeattend/webdashboard/src/lib/api/types.ts`

## Overview

**Priority:** High
**Status:** Pending

Main container component that orchestrates the topology view by fetching data and grouping devices under their parent agents.

## Key Insights

1. **Client-side mapping (Option A):** No backend changes needed
2. Device has `agent_id` field linking to agent
3. Both hooks use React Query - can fetch in parallel
4. Devices without valid `agent_id` go to "Unassigned" section

## Requirements

### Functional
- Fetch all agents (active by default, toggle for inactive)
- Fetch all devices
- Group devices by `agent_id` using `useMemo`
- Track expanded state for each agent card
- Pass grouped data to child components

### Non-Functional
- Smooth loading states with skeleton
- Under 200 lines (modular design)
- Follow existing animation patterns

## Architecture

```
ConnectivityTopology
├── Header (title, subtitle, add agent button)
├── Loading skeleton OR
├── Empty state OR
├── Agent cards
│   ├── AgentAccordionCard (agent + devices)
│   ├── AgentAccordionCard (agent + devices)
│   └── ...
└── Unassigned devices section
    └── DeviceListItem[]
```

### Data Flow

```typescript
// Group devices by agent_id
const { groupedDevices, unassignedDevices } = useMemo(() => {
  const grouped: Record<string, Device[]> = {};
  const unassigned: Device[] = [];

  for (const device of devices) {
    const agentExists = agents.some(a => a.id === device.agent_id);
    if (agentExists) {
      if (!grouped[device.agent_id]) grouped[device.agent_id] = [];
      grouped[device.agent_id].push(device);
    } else {
      unassigned.push(device);
    }
  }

  return { groupedDevices: grouped, unassignedDevices: unassigned };
}, [agents, devices]);
```

## Related Code Files

### Create
- `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/ConnectivityTopology.tsx`

### Import From
- `@/hooks/use-agents` - `useAuthorizedAgents`
- `@/hooks/use-devices` - `useDevices`
- `@/lib/utils` - `cn`
- `lucide-react` - icons
- `react-i18next` - translations

## Implementation Steps

1. Create directory structure:
   ```
   src/components/features/connectivity/
   ├── ConnectivityTopology.tsx
   ├── AgentAccordionCard.tsx
   └── DeviceListItem.tsx
   ```

2. Implement ConnectivityTopology:
   ```typescript
   import { useState, useMemo } from "react";
   import { useTranslation } from "react-i18next";
   import { Plus, Share2, Server, HardDrive } from "lucide-react";
   import { useAuthorizedAgents } from "@/hooks/use-agents";
   import { useDevices } from "@/hooks/use-devices";
   import { cn } from "@/lib/utils";
   import AgentAccordionCard from "./AgentAccordionCard";
   import DeviceListItem from "./DeviceListItem";

   export default function ConnectivityTopology() {
     const { t } = useTranslation();
     const { agents, isLoading: agentsLoading, includeInactive, setIncludeInactive, ...agentActions } = useAuthorizedAgents();
     const { devices, isLoading: devicesLoading, ...deviceActions } = useDevices();

     const [expandedAgents, setExpandedAgents] = useState<Set<string>>(new Set());

     // Group devices by agent_id
     const { groupedDevices, unassignedDevices } = useMemo(() => { ... }, [agents, devices]);

     const toggleExpanded = (agentId: string) => {
       setExpandedAgents(prev => {
         const next = new Set(prev);
         if (next.has(agentId)) next.delete(agentId);
         else next.add(agentId);
         return next;
       });
     };

     const isLoading = agentsLoading || devicesLoading;

     return (
       <div className="space-y-8 animate-in fade-in duration-500">
         {/* Header */}
         {/* Agent cards */}
         {/* Unassigned devices section */}
       </div>
     );
   }
   ```

3. Add header section with:
   - Icon + title + subtitle
   - Toggle for inactive agents
   - Add Agent button

4. Add agent list rendering:
   ```typescript
   {isLoading ? (
     <SkeletonLoader />
   ) : agents.length === 0 ? (
     <EmptyState />
   ) : (
     <div className="space-y-4">
       {agents.map(agent => (
         <AgentAccordionCard
           key={agent.id}
           agent={agent}
           devices={groupedDevices[agent.id] || []}
           isExpanded={expandedAgents.has(agent.id)}
           onToggleExpand={() => toggleExpanded(agent.id)}
           agentActions={agentActions}
           deviceActions={deviceActions}
         />
       ))}
     </div>
   )}
   ```

5. Add unassigned devices section:
   ```typescript
   {unassignedDevices.length > 0 && (
     <div className="mt-8">
       <h3 className="text-xs font-bold uppercase tracking-widest text-slate-500 mb-4">
         {t("devices.unassigned_title")}
       </h3>
       <div className="space-y-3">
         {unassignedDevices.map(device => (
           <DeviceListItem key={device.id} device={device} {...deviceActions} />
         ))}
       </div>
     </div>
   )}
   ```

## Todo List

- [ ] Create `connectivity` directory under components/features
- [ ] Implement ConnectivityTopology component
- [ ] Add state management for expanded agents
- [ ] Implement device grouping logic
- [ ] Add loading skeleton
- [ ] Add empty state with guide for first agent
- [ ] Add unassigned devices section
- [ ] Add i18n keys for new labels

## Success Criteria

- [ ] Component renders without errors
- [ ] Devices correctly grouped under agents
- [ ] Expand/collapse state persists during session
- [ ] Loading states show skeleton
- [ ] Empty state shows helpful message

## Risk Assessment

| Risk | Mitigation |
|------|------------|
| Performance with many devices | Use `useMemo` for grouping |
| State lost on re-render | Use `useState` with Set |

## Security Considerations

- No new API calls - uses existing authenticated hooks
- No sensitive data exposure

## Next Steps

After Phase 1, proceed to Phase 2: AgentAccordionCard implementation.
