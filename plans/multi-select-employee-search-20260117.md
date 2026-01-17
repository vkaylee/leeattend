# Implementation Plan: Multi-select Employee Search with Chips

## Approach
- **Why this solution**: Provides a fluid "Search -> Select -> Repeat" workflow. Users can see all active filters (employees) as chips and remove them easily.
- **Goal**: Refactor `EmployeeSearch` to support both single and multiple selection modes.

## Steps

### 1. Refactor `EmployeeSearch.tsx`
- Add `multiple?: boolean` prop.
- Add `selectedEmployees` metadata state to display names on chips (to avoid looking up names from the ID list alone).
- Implement the "Chips" display area above or inside the search bar.

### 2. Update `Combobox.tsx`
- Ensure `onValueChange` and selection events work correctly when multiple selections happen in sequence.

### 3. Implementation Details
```tsx
// Logic for multiple selection
const handleSelect = (employeeId: string) => {
  if (multiple) {
    const currentValues = Array.isArray(value) ? value : [];
    if (!currentValues.includes(employeeId)) {
      onChange([...currentValues, employeeId]);
    }
    setSearchTerm(""); // Clear search for the next person
  } else {
    onChange(employeeId);
  }
};
```

## Timeline
- Refactor: 20 min
- UI/UX (Chips): 20 min
- Testing: 20 min
- **Total: 1 hour**

## Rollback
- `git checkout webdashboard/src/components/common/EmployeeSearch.tsx`
