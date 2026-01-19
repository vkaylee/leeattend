# Implementation Plan: Modernize Shifts Page UI & Search (2026-01-19)

### ## Approach
- **Why this solution**: The current 2-column layout wastes ~30% of horizontal space on a static list. Moving to a **Search-First Header** using the existing modern `Combobox` optimizes UX for large teams, supports ARIA/Search, and gives the Weekly Roster the full width it needs for readability.
- **Goal**: Refactor `ShiftsPage.tsx` to use the `Combobox` for employee selection and remove the static sidebar, allowing the roster to take full width.

### ## Steps

1. **Transform Data for Combobox**
   - Map `employees` from `useQuery` to `ComboboxOption` format.
   - Include `first_name` + `last_name` as label and `employee_id_number` as description.

2. **Layout Refactor**
   - Modify `webdashboard/src/pages/ShiftsPage.tsx`.
   - Remove the `col-span-12 lg:col-span-4` sidebar div.
   - Change the main content div from `lg:col-span-8 xl:col-span-9` to `col-span-12`.

3. **Integrate Combobox Search**
   - Place the `Combobox` in the header or in a dedicated "Selection Bar" above the roster.

4. **Refine UI/UX**
   - Ensure empty states and transitions are smooth.

5. **Verification**
   - Run `npm test -- ShiftsPage.test.tsx` to ensure functionality remains intact.

### ## Timeline
| Phase | Duration |
|-------|----------|
| Preparation | 5 min |
| Layout Refactor | 20 min |
| Integration | 15 min |
| Testing | 20 min |
| **Total** | **1 hour** |
