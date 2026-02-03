# Scout Report: Device Enable/Disable Toggle Implementation

**Date:** 2026-02-01
**Scout ID:** ac3356d
**Task:** Locate device toggle implementation components and identify gaps

---

## Backend (Rust) - Current State

### 1. Device Model & Schema

**File:** `/home/elt1541/leeattend/apiserver/src/models.rs` (Lines 152-186)

```rust
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Device {
    pub id: Uuid,
    pub agent_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
    pub is_enabled: bool,              // ✅ Field exists
    pub last_status: String,
    pub created_at: DateTime<Utc>,
    pub workspace_id: Uuid,
    pub max_concurrent_workers: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDeviceRequest {
    pub name: Option<String>,
    pub ip: Option<String>,
    pub port: Option<i32>,
    pub brand: Option<String>,
    pub is_enabled: Option<bool>,      // ✅ Field exists in DTO
}
```

**Schema:** `/home/elt1541/leeattend/apiserver/migrations/20260107000000_create_devices_table.sql`

```sql
CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY,
    agent_id VARCHAR(50) NOT NULL,
    name VARCHAR(100) NOT NULL,
    ip VARCHAR(45) NOT NULL,
    port INTEGER DEFAULT 4370,
    is_enabled BOOLEAN DEFAULT true,   -- ✅ Database column exists
    last_status VARCHAR(20) DEFAULT 'unknown',
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
```

**Status:** `is_enabled` field exists in model, DTO, and database schema.

---

### 2. Device Update Handler

**File:** `/home/elt1541/leeattend/apiserver/src/handlers/agents.rs` (Lines 276-319)

```rust
pub async fn update_device(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    MemberContext { workspace_id, .. }: MemberContext,
    Json(req): Json<UpdateDeviceRequest>,
) -> impl IntoResponse {
    // Fetch current device with workspace_id validation
    let current_device = match sqlx::query_as::<_, Device>(
        "SELECT * FROM devices WHERE id = $1 AND workspace_id = $2",
    )
    .bind(id)
    .bind(workspace_id)
    .fetch_one(&state.db)
    .await
    {
        Ok(d) => d,
        Err(_) => return StatusCode::NOT_FOUND.into_response(),
    };

    // Extract is_enabled with fallback to current value
    let is_enabled = req.is_enabled.unwrap_or(current_device.is_enabled);

    // UPDATE query includes is_enabled
    match sqlx::query(
        "UPDATE devices SET name = $1, ip = $2, port = $3, is_enabled = $4, brand = $5 WHERE id = $6 AND workspace_id = $7",
    )
    .bind(name)
    .bind(ip)
    .bind(port)
    .bind(is_enabled)  // ✅ is_enabled is persisted
    .bind(brand)
    .bind(id)
    .bind(workspace_id)
    .execute(&state.db)
    .await
    {
        Ok(_) => {
            // ✅ ConfigPush trigger on device update
            let _ = state.agent_service.notify_config_update(&current_device.agent_id).await;
            StatusCode::OK.into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
```

**Workspace Validation:**
- Uses `MemberContext { workspace_id, .. }` from auth middleware
- Query filters by `workspace_id` to prevent cross-workspace access
- Returns `404 NOT_FOUND` if device doesn't belong to workspace

**Status:** ✅ Handler supports `is_enabled` updates with workspace security.

---

### 3. ConfigPush Mechanism

**File:** `/home/elt1541/leeattend/apiserver/src/services/agent_service.rs` (Lines 16-38)

```rust
pub async fn notify_config_update(&self, agent_id: &str) -> Result<(), String> {
    let devices = self.repo.find_devices_by_agent_id(agent_id).await?;
    let agent = self.repo.find_by_id(agent_id).await?;
    let max_workers = agent.map(|a| a.max_concurrent_workers);

    let config_data = ConfigData {
        devices: devices.into_iter().map(|d| {
            DeviceConfigPush {
                device_id: d.id.to_string(),
                name: d.name,
                ip: d.ip,
                port: d.port,
                brand: d.brand
            }
        }).collect(),
        max_concurrent_workers: max_workers
    };

    self.manager
        .push_message(agent_id, CentralMessage::ConfigPush { data: config_data })
        .await
        .map_err(|e| e.to_string())
}
```

**File:** `/home/elt1541/leeattend/apiserver/src/agent_manager.rs` (Lines 10-22)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfigPush {
    pub device_id: String,
    pub name: String,
    pub ip: String,
    pub port: i32,
    pub brand: String,
    // ❌ MISSING: is_enabled field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigData {
    pub devices: Vec<DeviceConfigPush>,
    pub max_concurrent_workers: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CentralMessage {
    ConfigPush { data: ConfigData },
}
```

**Status:** ❌ `DeviceConfigPush` struct does NOT include `is_enabled` field. Agent receives device config without enable/disable state.

**ConfigPush Triggers:**
1. Device create: Line 269 in `handlers/agents.rs`
2. Device update: Line 314 in `handlers/agents.rs`
3. Device delete: Line 342 in `handlers/agents.rs`
4. Agent handshake: Line 145 in `handlers/agents.rs`
5. Agent update: Line 522 in `handlers/agents.rs`

---

### 4. Device Query in Repository

**File:** `/home/elt1541/leeattend/apiserver/src/repositories/agent_repository.rs` (Lines 59-69)

```rust
async fn find_devices_by_agent_id(&self, agent_id: &str) -> Result<Vec<Device>, String> {
    let devices = sqlx::query_as::<_, Device>(
        "SELECT * FROM devices WHERE agent_id = $1 ORDER BY created_at DESC",
    )
    .bind(agent_id)
    .fetch_all(&self.pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(devices)
}
```

**Status:** ✅ Query returns full `Device` model including `is_enabled`, but NOT filtered.

**Gap:** Repository returns ALL devices (enabled + disabled). ConfigPush sends all devices to agent without filtering by `is_enabled`.

---

## Frontend (React) - Current State

### 1. Device API Client

**File:** `/home/elt1541/leeattend/webdashboard/src/lib/api/agents.ts` (Lines 36-46)

```typescript
export async function updateDevice(
  id: string,
  device: Partial<Device>,
): Promise<Device> {
  const response = await apiFetch(`${API_BASE_URL}/devices/${id}`, {
    method: "PUT",
    body: JSON.stringify(device),
  });
  if (!response.ok) throw new Error("Failed to update device");
  return response.json();
}
```

**Status:** ✅ API client supports sending `is_enabled` in payload.

---

### 2. Device Hook

**File:** `/home/elt1541/leeattend/webdashboard/src/hooks/use-devices.ts` (Lines 23-29)

```typescript
const updateMutation = useMutation({
  mutationFn: ({ id, data }: { id: string; data: Partial<Device> }) =>
    updateDevice(id, data),
  onSuccess: () => {
    queryClient.invalidateQueries({ queryKey: devicesKeys.all });
  },
});
```

**Status:** ✅ Hook supports updating `is_enabled` via `updateDevice` function.

---

### 3. DevicesPage UI

**File:** `/home/elt1541/leeattend/webdashboard/src/pages/DevicesPage.tsx`

**Current Implementation (Lines 56-62):**

```typescript
async function handleUpdate(e: React.FormEvent) {
  e.preventDefault();
  try {
    await updateDevice(editingDevice.id, { 
      name: editingDevice.name, 
      agent_id: editingDevice.agent_id, 
      ip: editingDevice.ip, 
      port: editingDevice.port, 
      brand: editingDevice.brand, 
      is_enabled: editingDevice.is_enabled  // ✅ is_enabled is sent
    });
    toast.success(t("devices.toast.update_success", { name: editingDevice.name }));
    setEditingDevice(null);
  } catch (e) { console.error(e); }
}
```

**Display State (Lines 144-148):**

```tsx
<span className={cn("px-4 py-1.5 rounded-full text-[9px] font-bold uppercase tracking-widest border shadow-sm", 
  device.is_enabled ? "bg-emerald-50 text-emerald-600 border-emerald-100" : "bg-slate-100 text-slate-500 border-slate-200 opacity-60"
)}>
  {device.is_enabled ? t("devices.table.active").toUpperCase() : t("devices.table.disabled").toUpperCase()}
</span>
```

**Status:**
- ✅ UI displays device enable/disable state
- ❌ NO TOGGLE BUTTON - only visible in edit modal
- ❌ User must open edit modal to change `is_enabled`

---

### 4. Toggle Component Patterns

**Reference:** `/home/elt1541/leeattend/webdashboard/src/pages/AgentsPage.tsx` (Lines 53-58, 156)

```typescript
async function handleToggleTrust(agent: AuthorizedAgent) {
  if (!confirm(t(agent.is_trusted ? "agents.messages.confirm_revoke" : "agents.messages.confirm_approve", { name: agent.name }))) return;
  try {
    await updateTrust({ id: agent.id, is_trusted: !agent.is_trusted });
    toast.success(t(agent.is_trusted ? "agents.messages.revoke_success" : "agents.messages.approve_success", { name: agent.name }));
  } catch (e) { console.error(e); }
}

// Button usage:
<button onClick={() => handleToggleTrust(agent)} className={cn(...)}>
  {agent.is_trusted ? t("common.revoke").toUpperCase() : t("common.approve").toUpperCase()}
</button>
```

**Pattern:** Button-based toggle (not Switch component) with confirmation dialog.

**No Switch Component Found:**
- No files matching `switch.tsx` or `toggle.tsx` in `/home/elt1541/leeattend/webdashboard`
- Existing toggles use button components with conditional styling

---

## Gap Analysis

### Backend Gaps

1. **ConfigPush Missing `is_enabled`**
   - `DeviceConfigPush` struct in `agent_manager.rs` lacks `is_enabled` field
   - Agent receives device configs without enable/disable state
   - Agent cannot distinguish between active/inactive devices

2. **No Device Filtering**
   - `find_devices_by_agent_id` returns ALL devices
   - ConfigPush sends both enabled and disabled devices to agent
   - Agent may process logs from disabled devices

**Impact:** Agent receives disabled device configs and may attempt connections.

---

### Frontend Gaps

1. **No Quick Toggle UI**
   - Device list shows `is_enabled` state as badge only
   - User must open edit modal to toggle device
   - No inline toggle button on device list

2. **No Real-Time Update Feedback**
   - No indication when ConfigPush is sent to agent
   - No confirmation if agent received updated config
   - User doesn't know if toggle took effect on device

---

## Recommendations

### Backend Changes Required

1. **Add `is_enabled` to `DeviceConfigPush`**
   ```rust
   pub struct DeviceConfigPush {
       pub device_id: String,
       pub name: String,
       pub ip: String,
       pub port: i32,
       pub brand: String,
       pub is_enabled: bool,  // ADD THIS
   }
   ```

2. **Update ConfigPush Mapping**
   ```rust
   DeviceConfigPush {
       device_id: d.id.to_string(),
       name: d.name,
       ip: d.ip,
       port: d.port,
       brand: d.brand,
       is_enabled: d.is_enabled,  // ADD THIS
   }
   ```

3. **Optional: Filter Disabled Devices**
   - Consider filtering disabled devices before ConfigPush
   - OR let agent decide to skip disabled devices

---

### Frontend Changes Required

1. **Add Toggle Button to Device List**
   - Add inline toggle button next to status badge
   - Follow `AgentsPage` pattern with confirmation dialog
   - Use optimistic updates for instant feedback

2. **Implement Toggle Handler**
   ```typescript
   async function handleToggleDevice(device: Device) {
     if (!confirm(t(device.is_enabled ? "devices.confirm_disable" : "devices.confirm_enable", { name: device.name }))) return;
     try {
       await updateDevice(device.id, { is_enabled: !device.is_enabled });
       toast.success(t(device.is_enabled ? "devices.disabled_success" : "devices.enabled_success", { name: device.name }));
     } catch (e) { console.error(e); }
   }
   ```

3. **Add UI Feedback**
   - Show loading state during toggle
   - Display success/error toast
   - Optional: Show "Config pushed to agent" notification

---

## Existing Patterns

### Toggle Pattern (AgentsPage)
- Button-based toggle (not Switch component)
- Confirmation dialog before action
- Success/error toast notifications
- Conditional button styling based on state

### Workspace Validation Pattern
- All handlers use `MemberContext` for workspace_id
- Queries filter by workspace_id AND resource_id
- Returns 404 if resource doesn't belong to workspace

### ConfigPush Pattern
- Triggered after device create/update/delete
- Triggered after agent authorization changes
- Uses WebSocket to push config to connected agents
- Agent receives `ConfigPush` message type with devices array

---

## Files Summary

### Backend
- `/home/elt1541/leeattend/apiserver/src/models.rs` - Device model with `is_enabled`
- `/home/elt1541/leeattend/apiserver/src/handlers/agents.rs` - Device CRUD handlers
- `/home/elt1541/leeattend/apiserver/src/services/agent_service.rs` - ConfigPush logic
- `/home/elt1541/leeattend/apiserver/src/agent_manager.rs` - ConfigPush message structs
- `/home/elt1541/leeattend/apiserver/src/repositories/agent_repository.rs` - Device queries
- `/home/elt1541/leeattend/apiserver/migrations/20260107000000_create_devices_table.sql` - Schema

### Frontend
- `/home/elt1541/leeattend/webdashboard/src/lib/api/agents.ts` - Device API client
- `/home/elt1541/leeattend/webdashboard/src/hooks/use-devices.ts` - Device hook
- `/home/elt1541/leeattend/webdashboard/src/pages/DevicesPage.tsx` - Device list UI
- `/home/elt1541/leeattend/webdashboard/src/pages/AgentsPage.tsx` - Toggle pattern reference

---

## Unresolved Questions

1. Should disabled devices be filtered out from ConfigPush entirely, or sent with `is_enabled: false` flag?
2. Should agent disconnect from disabled devices immediately, or wait for next poll?
3. Should UI show real-time agent connection status after toggle?
4. Should there be a "Force Sync" button to manually trigger ConfigPush?

