# Implementation Plan - Vertical Metro Map Priority Legend

## Task
Update `PriorityLegend` to use a vertical "Metro Map" style to better visualize the hierarchy of schedule tracing priorities.

## Proposed Design
- A vertical line on the left.
- Dots (stations) representing each priority level.
- Labels on the right.
- "Highest Priority" indicator at the top.
- "Fallback Path" arrow or visual indicator.

## Files to Modify
- `webdashboard/src/components/shifts/PriorityLegend.tsx`
- `webdashboard/src/components/shifts/__tests__/PriorityLegend.test.tsx` (if needed)

## Steps
1. Update `PriorityLegend.tsx` with new layout and styles.
2. Verify visual appearance.
3. Run and update tests.
