# Scout Report: Connectivity Topology UI Redesign

**Task**: Find files related to Connectivity Settings UI redesign
**Date**: 2026-02-01 13:25
**Scout ID**: ac9f8e2

## 1. Current Workspace Settings Structure

### Settings Layout
- **File**: `/home/elt1541/leeattend/webdashboard/src/pages/settings/SettingsLayout.tsx`
- Left sidebar navigation with categories
- Uses React Router `<Outlet />` for content
- Route: `/workspace/settings/connectivity`

### Connectivity Settings Page
- **File**: `/home/elt1541/leeattend/webdashboard/src/pages/settings/ConnectivitySettings.tsx`
- Tab switcher between "agents" and "devices"
- Embeds full `AgentsPage` and `DevicesPage` components
- Uses `useState<TabType>` for active tab switching

## 2. Existing Components

### AgentsPage
- **File**: `/home/elt1541/leeattend/webdashboard/src/pages/AgentsPage.tsx`
- Table-based display
- Features: Add, Edit, Delete, Trust/Revoke, Restore
- Toggle for active/inactive agents
- Auto-refresh every 15s
- Modal: `/home/elt1541/leeattend/webdashboard/src/components/features/agents/AgentEditModal.tsx`

### DevicesPage
- **File**: `/home/elt1541/leeattend/webdashboard/src/pages/DevicesPage.tsx`
- Card-based grid display
- Features: Add, Edit, Delete, Enable/Disable toggle
- Device fields: `id`, `agent_id`, `name`, `ip`, `port`, `brand`, `is_enabled`
- Dropdown menu for actions
- **EXISTING TOGGLE**: Line 74-88 has `handleToggleDevice()` function with confirmation

### UI Components Available
- **Modal**: `/home/elt1541/leeattend/webdashboard/src/components/ui/Modal.tsx`
- **Dropdown**: `/home/elt1541/leeattend/webdashboard/src/components/ui/dropdown-menu.tsx`
- **Combobox**: `/home/elt1541/leeattend/webdashboard/src/components/ui/Combobox.tsx`
- **Tabs**: `/home/elt1541/leeattend/webdashboard/src/components/ui/Tabs.tsx`
- **Sheet**: `/home/elt1541/leeattend/webdashboard/src/components/ui/Sheet.tsx`

**NO ACCORDION COMPONENT** - Would need to create custom collapsible

## 3. API Endpoints

### Agents API
- **File**: `/home/elt1541/leeattend/webdashboard/src/lib/api/agents.ts`
- `GET /authorized-agents` (includes `?include_inactive=true`)
- `POST /authorized-agents`
- `PATCH /authorized-agents/:id`
- `DELETE /authorized-agents/:id`
- `PUT /authorized-agents/:id/trust`

### Devices API
- **File**: `/home/elt1541/leeattend/webdashboard/src/lib/api/agents.ts`
- `GET /devices`
- `POST /devices`
- `PUT /devices/:id`
- `DELETE /devices/:id`

### Type Definitions
- **File**: `/home/elt1541/leeattend/webdashboard/src/lib/api/types.ts`

**AuthorizedAgent**:
```typescript
interface AuthorizedAgent {
  id: string;
  name: string;
  token: string;
  is_trusted: boolean;
  last_ip?: string;
  last_seen_at: string;
  created_at: string;
  is_active: boolean;
  max_concurrent_workers: number;
  workspace_id: string;
}
```

**Device**:
```typescript
interface Device {
  id: string;
  agent_id: string;  // ← Links device to agent
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

**KEY FINDING**: `Device.agent_id` field exists for nested topology

## 4. Hooks

### useAuthorizedAgents
- **File**: `/home/elt1541/leeattend/webdashboard/src/hooks/use-agents.ts`
- React Query with auto-refetch
- CRUD operations: `createAgent`, `updateAgent`, `updateTrust`, `deleteAgent`
- Includes `includeInactive` state toggle

### useDevices
- **File**: `/home/elt1541/leeattend/webdashboard/src/hooks/use-devices.ts`
- React Query for devices
- CRUD operations: `createDevice`, `updateDevice`, `deleteDevice`

## 5. UI Patterns

### Hierarchical Views
- **Sidebar**: `/home/elt1541/leeattend/webdashboard/src/components/layout/Sidebar.tsx`
- Uses `openSections` state + `toggleSection()` handler
- ChevronDown/ChevronRight icons from lucide-react
- Expandable sections with `isCollapsed` state

### Toggle Pattern (Already Exists!)
- **DevicesPage** line 74-88: `handleToggleDevice()`
- Confirmation dialog before toggle
- Loading state during API call (`togglingId`)
- Success/error toast notifications
- Pattern: `bg-emerald-600` (enabled) / `text-rose-600` (disabled)

### Animation Patterns
- `animate-in fade-in duration-500` (ConnectivitySettings)
- `group-hover:` prefixes for hover effects
- `transition-all` for smooth state changes
- `active:scale-95` for button press feedback

### Design System
- Rounded corners: `rounded-[2.5rem]`, `rounded-2xl`, `rounded-xl`
- Shadows: `shadow-sm`, `shadow-xl`, `shadow-premium-2xl`
- Colors: `primary-600`, `slate-500`, `emerald-600`, `rose-600`
- Spacing: `gap-4`, `gap-8`, `p-8`, `px-10`
- Typography: `font-bold uppercase tracking-widest`

## 6. Key Files for Implementation

### Must Modify
1. `/home/elt1541/leeattend/webdashboard/src/pages/settings/ConnectivitySettings.tsx` - Main page
2. `/home/elt1541/leeattend/webdashboard/src/pages/AgentsPage.tsx` - Refactor for topology
3. `/home/elt1541/leeattend/webdashboard/src/pages/DevicesPage.tsx` - Extract device cards

### May Create
1. `/home/elt1541/leeattend/webdashboard/src/components/connectivity/AgentTopologyCard.tsx` - New accordion card
2. `/home/elt1541/leeattend/webdashboard/src/components/connectivity/DeviceListItem.tsx` - Nested device component
3. `/home/elt1541/leeattend/webdashboard/src/components/ui/Accordion.tsx` - If needed

### Hooks (Reuse Existing)
- `/home/elt1541/leeattend/webdashboard/src/hooks/use-agents.ts` ✓
- `/home/elt1541/leeattend/webdashboard/src/hooks/use-devices.ts` ✓

## 7. Implementation Notes

### Data Grouping Strategy
Fetch both agents + devices, then group in frontend:
```typescript
const devicesByAgent = devices.reduce((acc, device) => {
  if (!acc[device.agent_id]) acc[device.agent_id] = [];
  acc[device.agent_id].push(device);
  return acc;
}, {} as Record<string, Device[]>);
```

### Accordion State Management
```typescript
const [expandedAgents, setExpandedAgents] = useState<Set<string>>(new Set());
const toggleAgent = (agentId: string) => {
  setExpandedAgents(prev => {
    const next = new Set(prev);
    next.has(agentId) ? next.delete(agentId) : next.add(agentId);
    return next;
  });
};
```

### Toggle Consistency
Reuse existing toggle pattern from DevicesPage:
- Confirmation modal before disable
- Loading spinner during API call
- Toast notification on success/error
- Visual state: emerald (enabled) / rose (disabled)

## 8. Testing Files

- `/home/elt1541/leeattend/webdashboard/src/pages/__tests__/AgentsPage.test.tsx`
- `/home/elt1541/leeattend/webdashboard/src/pages/__tests__/DevicesPage.test.tsx`

## Unresolved Questions

1. Should we remove standalone AgentsPage/DevicesPage routes or keep them?
2. Default state: all agents collapsed or all expanded?
3. Batch operations for devices under an agent?
4. Visual indicator for agents with no devices?
5. Filter/search for devices within expanded agent?
