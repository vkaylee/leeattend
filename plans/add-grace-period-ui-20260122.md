# Implementation Plan: Add Grace Period Configuration to Shifts UI

This plan addresses the missing "Early Punch-in Grace Period" configuration in the frontend, enabling users to fully utilize the newly implemented "Early Arrival Policy" logic.

## Approach
- **Frontend Type Update**: Add the missing field `early_punch_in_grace_period_minutes` to the `Shift` interface.
- **Frontend Logic Update**: Include this field in the API payloads for creating and updating shifts (`createShift`, `updateShift`).
- **Frontend UI Update**: Add a number input field to the Shift Create/Edit modal in `ShiftsPage.tsx`.
- **Validation**: Ensure the backend receives and persists this value correctly (already implemented, but we'll verify via UI).

## Steps

1.  **Frontend: Update Types** (5 min)
    -   Modify `webdashboard/src/lib/api/types.ts`:
        -   Add `early_punch_in_grace_period_minutes: number;` to the `Shift` interface.
        -   Update `CreateShiftRequest` and `UpdateShiftRequest` types if they exist or are inferred.

2.  **Frontend: Update API Client** (5 min)
    -   Verify `webdashboard/src/lib/api/shifts.ts` to ensuring the new field is passed in the request body. (Usually JSON stringify handles this if the input object has the property).

3.  **Frontend: Update Shift Modal UI** (20 min)
    -   Modify `webdashboard/src/pages/ShiftsPage.tsx`:
        -   Update the `ShiftFormData` interface (or local state type).
        -   Add a new form field input for "Grace Period (minutes)".
        -   Place it logically near the Start/End time or in an "Advanced" section if available.
        -   Label: "Thời gian cho phép vào sớm (phút)" / "Early Arrival Grace Period (min)".
        -   Default value: 15 (or 0).

4.  **Integration Verification** (10 min)
    -   Start the dev server.
    -   Create a new shift with a specific grace period (e.g., 30 mins).
    -   Verify in the list view (optional) or by re-opening the edit modal that the value is persisted.

## Timeline
| Phase | Duration |
|-------|----------|
| Update Types | 5 min |
| Update API Client | 5 min |
| UI Implementation | 20 min |
| Verification | 10 min |
| **Total** | **40 min** |

## Rollback Plan
-   Revert changes to `ShiftsPage.tsx` and `types.ts`.

## Security Checklist
-   [x] Input validation: Ensure only non-negative integers are allowed.
-   [x] Auth checks: Existing admin checks apply.
