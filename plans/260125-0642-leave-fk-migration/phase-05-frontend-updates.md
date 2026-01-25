# Phase 5: Frontend Updates

## Context Links
- Plan: `./plan.md`
- Phase 4: `./phase-04-backend-handlers.md`
- types: `webdashboard/src/lib/api/types.ts`
- leaves API: `webdashboard/src/lib/api/leaves.ts`
- LeaveRequestsPage: `webdashboard/src/pages/LeaveRequestsPage.tsx`
- LeaveBalancesPage: `webdashboard/src/pages/LeaveBalancesPage.tsx`

## Overview
- **Priority**: P1
- **Status**: pending
- **Effort**: 1.5h
- **Dependencies**: Can start in parallel with Phase 3-4 (type definitions)

Update frontend types and components to work with leave_type_id UUID.

## Key Insights
- Frontend currently uses `leave_type: string` everywhere
- Need to fetch leave types and let user select by ID
- Display should use `leave_type_name` for readability

## Requirements

### Functional
- Update TypeScript types to use `leave_type_id`
- Add leave type selector component
- Update submit forms to send `leave_type_id`
- Display leave type name in lists/tables

### Non-Functional
- Maintain UX (user selects by name, sends ID)
- Cache leave types for performance

## Related Code Files

### Files to Modify
- `webdashboard/src/lib/api/types.ts`
- `webdashboard/src/lib/api/leaves.ts`
- `webdashboard/src/pages/LeaveRequestsPage.tsx`
- `webdashboard/src/pages/LeaveBalancesPage.tsx`
- `webdashboard/src/components/leave/AssignPolicyModal.tsx`
- `webdashboard/src/components/leave/LeaveBalanceDetailModal.tsx`
- `webdashboard/src/components/leave/LeaveHistoryModal.tsx`

## Implementation Steps

### Step 1: Update TypeScript Types (types.ts)

```typescript
// Update LeaveRequest interface
export interface LeaveRequest {
  id: string;
  employee_id: string;
  leave_type_id: string;           // Changed from leave_type: string
  leave_type_code?: string;        // Added for display
  leave_type_name?: string;        // Added for display
  is_paid?: boolean;               // Added
  start_date: string;
  end_date: string;
  reason?: string;
  status: string;
  requested_by_user_id?: string;
  approved_by_user_id?: string;
  created_at: string;
  updated_at: string;
}

// Update EmployeeLeaveBalance interface
export interface EmployeeLeaveBalance {
  leave_type_id: string;           // Added
  leave_type_code: string;         // Changed from leave_type
  leave_type_name: string;         // Added
  is_paid: boolean;                // Added
  entitlement: number;
  accrued_amount: number;
  carried_over: number;
  used: number;
  remaining: number;
}

// Update WorkspaceLeaveBalance interface
export interface WorkspaceLeaveBalance {
  employee_id: string;
  employee_name: string;
  leave_type_id: string;           // Added
  leave_type_code: string;         // Changed from leave_type
  leave_type_name: string;         // Added
  is_paid: boolean;                // Added
  entitlement: number;
  accrued_amount: number;
  carried_over: number;
  used: number;
  remaining: number;
  policy_name?: string;
  assignment_type?: string;
}

// LeaveType already exists and is correct
```

### Step 2: Update leaves.ts API Functions

```typescript
// Update submitLeaveRequest
export async function submitLeaveRequest(
  wsId: string,
  data: {
    employee_id: string;
    leave_type_id: string;      // Changed from leave_type
    start_date: string;
    end_date: string;
    reason?: string
  },
): Promise<LeaveRequest> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/leave/submit`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to submit leave request");
  return response.json();
}

// Update adjustLeaveBalance
export async function adjustLeaveBalance(
  wsId: string,
  employeeId: string,
  data: {
    leave_type_id: string;      // Changed from leave_type
    amount: number;
    reason: string;
    transaction_date: string
  }
): Promise<void> {
  const response = await apiFetch(`${API_BASE_URL}/workspaces/${wsId}/employees/${employeeId}/leave/adjust`, {
    method: "POST",
    body: JSON.stringify(data),
  });
  if (!response.ok) throw new Error("Failed to adjust leave balance");
}
```

### Step 3: Create LeaveTypeSelect Component

Create `webdashboard/src/components/leave/LeaveTypeSelect.tsx`:

```typescript
import React from "react";
import { useQuery } from "@tanstack/react-query";
import { fetchLeaveTypes } from "@/lib/api/leaves";
import { LeaveType } from "@/lib/api/types";
import { useWorkspace } from "@/hooks/useWorkspace";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

interface LeaveTypeSelectProps {
  value: string;
  onChange: (leaveTypeId: string) => void;
  placeholder?: string;
  disabled?: boolean;
}

export function LeaveTypeSelect({
  value,
  onChange,
  placeholder = "Select leave type",
  disabled = false,
}: LeaveTypeSelectProps) {
  const { workspaceId } = useWorkspace();

  const { data: leaveTypes = [], isLoading } = useQuery({
    queryKey: ["leave-types", workspaceId],
    queryFn: () => fetchLeaveTypes(workspaceId!),
    enabled: !!workspaceId,
    staleTime: 5 * 60 * 1000, // Cache for 5 minutes
  });

  return (
    <Select value={value} onValueChange={onChange} disabled={disabled || isLoading}>
      <SelectTrigger>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent>
        {leaveTypes.map((lt: LeaveType) => (
          <SelectItem key={lt.id} value={lt.id}>
            {lt.name} ({lt.code})
            {!lt.is_paid && " - Unpaid"}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
```

### Step 4: Update LeaveRequestsPage.tsx

Update the submit form to use LeaveTypeSelect:

```typescript
// In LeaveRequestsPage.tsx

// Replace leave_type string input with LeaveTypeSelect
const [formData, setFormData] = useState({
  employee_id: "",
  leave_type_id: "",  // Changed from leave_type
  start_date: "",
  end_date: "",
  reason: "",
});

// In form JSX:
<LeaveTypeSelect
  value={formData.leave_type_id}
  onChange={(id) => setFormData({ ...formData, leave_type_id: id })}
/>

// Update table display to use leave_type_name
<td>{request.leave_type_name || request.leave_type_code}</td>
```

### Step 5: Update LeaveBalancesPage.tsx

Update table columns:

```typescript
// In table headers:
<th>Leave Type</th>

// In table rows:
<td>
  {balance.leave_type_name}
  {!balance.is_paid && <span className="text-xs text-muted-foreground ml-1">(Unpaid)</span>}
</td>
```

### Step 6: Update LeaveBalanceDetailModal.tsx

Update to use leave_type_id for adjustments:

```typescript
// When adjusting balance:
await adjustLeaveBalance(workspaceId, employeeId, {
  leave_type_id: balance.leave_type_id,  // Use ID instead of code
  amount: adjustmentAmount,
  reason: adjustmentReason,
  transaction_date: format(new Date(), 'yyyy-MM-dd'),
});
```

### Step 7: Update AssignPolicyModal.tsx

This component already uses leave_type_id since policies reference leave types by ID. Verify no changes needed.

## Todo List
- [ ] Update LeaveRequest type in types.ts
- [ ] Update EmployeeLeaveBalance type in types.ts
- [ ] Update WorkspaceLeaveBalance type in types.ts
- [ ] Update submitLeaveRequest in leaves.ts
- [ ] Update adjustLeaveBalance in leaves.ts
- [ ] Create LeaveTypeSelect component
- [ ] Update LeaveRequestsPage form to use LeaveTypeSelect
- [ ] Update LeaveRequestsPage table display
- [ ] Update LeaveBalancesPage table display
- [ ] Update LeaveBalanceDetailModal adjustment logic
- [ ] Update LeaveHistoryModal if needed
- [ ] Run `npm run type-check` to verify types
- [ ] Run `npm run lint` to verify code quality
- [ ] Test UI manually

## Success Criteria
- Forms submit leave_type_id instead of string
- Tables display leave type name
- Type checking passes
- UI works as expected

## Next Steps
Proceed to Phase 6 (Testing & Validation).
