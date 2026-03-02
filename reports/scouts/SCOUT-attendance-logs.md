## Codebase Analysis: Cải thiện UI/UX tìm kiếm & điều hướng tại /attendance-logs
### Current Implementation:
- **Page**: `AttendanceLogsPage.tsx`
- **Filter Component**: It uses `<EmployeeSearch>` inside an action bar.
- **Search Component**: `EmployeeSearch.tsx` wraps `<Combobox>` with `autoFocus` prop.
- **Underlying UI**: `Combobox.tsx` is built using `@radix-ui/react-popover` and `cmdk` (`<Command>`).
- **Data Model**: `Employee` type from API.

### Required Changes:
1. **autoFocus**: `EmployeeSearch` needs to pass `autoFocus={true}` from `AttendanceLogsPage.tsx` or handle it natively so that when the page loads, the input in `<Command.Input>` is focused. 
   - `Combobox` already accepts `autoFocus`, but the trigger is a `<button>`, and `<Command.Input>` also accepts `autoFocus`.
   - Actually, we want the *popover* to open open load if autoFocus is true? Or maybe just open when users type.
   - Wait: the requirement is "focus vào ô filter". In `Combobox.tsx`, the trigger is a button. Clicking it opens the popup with the `Command.Input`. If we just focus the button, the user still has to press Enter/Space to open it. We might want to make it open automatically, OR make the Combobox trigger respond to typing directly.
   - Let's check `Combobox.tsx`: it uses `<Popover.Trigger asChild><button>` and `<Command.Input>`.
   - For a real "focus and type" experience, standard `<select>`-like Combobox requires opening first. `cmdk` inside a Popover means the input is only mounted when the Popover is open.
   - If `autoFocus` is passed to `Combobox`, it does `useState(autoFocus || false)` for `open`. So it DOES open automatically! But does the input get focus? Yes, `<Command.Input autoFocus={autoFocus} />`.

2. **Select First Item on Enter**:
   - `cmdk` automatically selects the first item (visually highlights it).
   - Pressing Enter inside `<Command.Input>` will trigger the `onSelect` of the currently highlighted `<Command.Item>`.
   - This seems to be built-in to `cmdk`! We just need to ensure the `<Command.Item>`'s `onSelect` properly sets the value and closes the popover (`onChange(option.value); setOpen(false);` is already there).
   - So, pressing Enter should already work IF the popover is open.

3. **Re-focusing and typing (Keyboard shortcut)**:
   - Need a global keyboard shortcut (e.g., `/` or `f` or `Esc`) on `AttendanceLogsPage` that sets a state to re-open the EmployeeSearch combobox and focus the search input.
   - Or, when they are focused on the table/page, pressing a key should trigger the Combobox to open.
   - Also, when the Combobox opens again, the search term should ideally be cleared or selected so they can type a new name immediately.
   - In `EmployeeSearch.tsx`, `searchTerm` is local state. We might need a way to clear it via an exposed ref or by passing a `key` to reset it, or simply clear it when it opens.

### Testing Strategy:
- Open `/attendance-logs`.
- Verify the employee search combobox opens and is focused automatically.
- Type a name, verify `cmdk` filters.
- Press Enter, verify the employee is selected and the popover closes.
- Press `ESC` or `/`, verify the combobox re-opens, the input is focused, and previous text is clear.
