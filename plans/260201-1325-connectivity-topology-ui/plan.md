---
title: "Connectivity Topology UI Redesign"
description: "Replace tab-based agent/device management with hierarchical accordion topology view"
status: completed
priority: P2
effort: 6h
branch: main
tags: [frontend, ui-redesign, connectivity, agents, devices]
created: 2026-02-01
---

# Connectivity Topology UI Redesign

## Overview

Transform the current tab-based agent/device management into a hierarchical topology view where devices are nested under their parent agents, providing better visual representation of the connectivity infrastructure.

## Current State

| Component | Description |
|-----------|-------------|
| `ConnectivitySettings.tsx` | Tab switcher between AgentsPage and DevicesPage |
| `AgentsPage.tsx` | Table-based agent list (173 lines) |
| `DevicesPage.tsx` | Card-based device list (260 lines) |
| Hooks | `useAuthorizedAgents`, `useDevices` separate data fetching |

## Target State

Single unified topology view with:
- Expandable agent accordion cards
- Nested device list inside expanded agents
- Separate section for unassigned devices
- Visual dimming when parent agent is offline/inactive

---

## Phase Summary

| Phase | Description | Effort |
|-------|-------------|--------|
| 1 | Create ConnectivityTopology component | 1.5h |
| 2 | Create AgentAccordionCard component | 2h |
| 3 | Create DeviceListItem component | 1h |
| 4 | Modify ConnectivitySettings integration | 0.5h |
| 5 | Testing and polish | 1h |

---

## Phase Details

### Phase 1: ConnectivityTopology Component

**File:** `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/ConnectivityTopology.tsx`

**Responsibilities:**
- Fetch agents and devices using existing hooks
- Client-side grouping of devices by `agent_id`
- Render agent cards and unassigned devices section
- Manage expanded state for all accordions

### Phase 2: AgentAccordionCard Component

**File:** `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/AgentAccordionCard.tsx`

**Responsibilities:**
- Expandable card with chevron icon
- Display agent info: Name, Status, Last Seen, Trust status
- Actions: Edit, Toggle Trust, Delete
- Render nested DeviceListItem components when expanded

### Phase 3: DeviceListItem Component

**File:** `/home/elt1541/leeattend/webdashboard/src/components/features/connectivity/DeviceListItem.tsx`

**Responsibilities:**
- Compact row design for nested view
- Display: Name, Brand, IP:Port
- Controls: Enable/Disable toggle
- Actions: Edit, Delete (via dropdown)

### Phase 4: ConnectivitySettings Integration

**File:** `/home/elt1541/leeattend/webdashboard/src/pages/settings/ConnectivitySettings.tsx`

**Changes:**
- Remove tab switcher UI
- Replace with single ConnectivityTopology component
- Keep header/description unchanged

### Phase 5: Testing and Polish

- Verify all CRUD operations work
- Test accordion animations
- Verify loading states
- Test responsive behavior

---

## Links to Phase Details

- [Phase 1: ConnectivityTopology](./phase-01-connectivity-topology.md)
- [Phase 2: AgentAccordionCard](./phase-02-agent-accordion-card.md)
- [Phase 3: DeviceListItem](./phase-03-device-list-item.md)
- [Phase 4: ConnectivitySettings Integration](./phase-04-connectivity-settings.md)
- [Phase 5: Testing](./phase-05-testing.md)
