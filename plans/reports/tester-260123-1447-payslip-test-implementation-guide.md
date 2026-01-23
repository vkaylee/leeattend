# Payslip Component Test Implementation Guide

**Purpose**: Detailed test implementation specifications for PayslipModal and PayslipDocument components.

---

## Test File 1: PayslipModal.test.tsx

**Location**: `/home/elt1541/leeattend/webdashboard/src/components/payroll/__tests__/PayslipModal.test.tsx`

### Test Setup
```typescript
Dependencies required:
- Testing Library React (render, screen, fireEvent)
- Vitest (describe, it, expect, vi, beforeEach)
- React Router (BrowserRouter)
- TanStack React Query (QueryClientProvider)
- Mock Employee, Payslip, Workspace types
- Mock useDateTimeFormatter hook
```

### Mock Data Structure
```typescript
mockEmployee: {
  id: "emp-1"
  full_name: "Nguyễn Văn A"
  employee_id_number: "EMP001"
  department_name: "Engineering"
}

mockPayslip: {
  id: "slip-1"
  employee_id: "emp-1"
  period_start: "2026-01-01"
  period_end: "2026-01-31"
  base_pay_amount: "10000000"
  overtime_pay_amount: "500000"
  deductions_amount: "500000"
  net_pay_amount: "10000000"
  segments: [...]
  salary_components: [...]
  incentives: [...]
}

mockWorkspace: {
  id: "ws-1"
  name: "Tech Company Ltd"
}
```

### Test Cases (8 tests)

#### 1. Renders modal with payslip data when open
```
Verify:
- Dialog is rendered when isOpen={true}
- Dialog.Content exists with correct className
- PayslipDocument child component receives correct props
- Modal header displays "Official Payslip Preview"
- Period label shows formatted month/year
```

#### 2. Displays formatted month and year correctly
```
Setup: Use payslip.period_start = "2026-01-01"
Verify:
- formatMonthYear hook is called with period_start
- Displays formatted result in header subtitle
- Locale-aware formatting applied (Vietnamese months)
```

#### 3. Renders print button with correct styling
```
Verify:
- Print button exists with "Print Payslip" text
- Button has correct className structure
- Printer icon (from lucide-react) displays
- Styling includes: px-6 py-2.5, bg-slate-900, text-white
```

#### 4. Calls window.print when print button clicked
```
Setup: Mock window.print
Verify:
- window.print is NOT called on initial render
- window.print IS called when print button clicked
- Print button has onClick handler attached
- Other interactions don't trigger print
```

#### 5. Closes modal when close button clicked
```
Verify:
- Close button (X icon) exists in top right
- Dialog.Close wrapper exists
- onClose callback triggered when close clicked
- Dialog closes (isOpen becomes false)
```

#### 6. Modal closes when overlay clicked
```
Verify:
- Dialog.Overlay rendered with correct z-index
- Clicking overlay triggers onClose
- Overlay has backdrop blur effect
```

#### 7. Passes correct props to PayslipDocument child
```
Verify:
- PayslipDocument receives employee prop
- PayslipDocument receives payslip prop
- PayslipDocument receives workspace prop
- Props match exactly (no mutations)
```

#### 8. Print styles applied correctly
```
Verify:
- Dialog.Overlay has print:hidden class
- Dialog.Content has print:static, print:translate-x-0, print:w-auto
- Action bar has print:hidden
- Footer warning has print:hidden
- Content area has print:overflow-visible
```

---

## Test File 2: PayslipDocument.test.tsx

**Location**: `/home/elt1541/leeattend/webdashboard/src/components/payroll/__tests__/PayslipDocument.test.tsx`

### Test Setup
```typescript
Dependencies:
- Testing Library React (render, screen, within)
- Vitest
- Mock useDateTimeFormatter hook with:
  - formatMonthYear(date) => "Tháng 1/2026"
  - formatDate(date) => "01/01/2026"
- Mock Intl.NumberFormat for currency
```

### Mock Data Scenarios

#### Scenario A: Standard Payslip (Salary Components)
```typescript
// Payslip with salary_components (preferred path)
{
  period_start: "2026-01-01",
  period_end: "2026-01-31",
  base_pay_amount: "10000000",
  overtime_pay_amount: "500000",
  deductions_amount: "500000",
  net_pay_amount: "10000000",
  standard_units: 200,
  overtime_units: 10,
  salary_components: [
    {
      definition: { code: "BS", name: "Base Salary", amount: "10000000" },
      component: { amount: "10000000" }
    },
    {
      definition: { code: "AL", name: "Allowance", amount: "1000000" },
      component: { amount: "1000000" }
    }
  ],
  segments: [{
    start_date: "2026-01-01",
    end_date: "2026-01-31",
    reason: "Standard",
    applied_base_salary: 10000000,
    applied_std_days: 22,
    paid_work_days: 22,
    actual_work_days: 22,
    amount: 10000000,
    applied_hours_per_day: 8
  }],
  incentives: [
    { policy_name: "Diligence Bonus", amount: "500000" }
  ]
}
```

#### Scenario B: Fallback Path (No Salary Components)
```typescript
{
  // salary_components is empty or undefined
  segments: [
    {
      start_date: "2026-01-01",
      end_date: "2026-01-31",
      paid_work_days: 22,
      amount: 11000000,
      applied_base_salary: 11000000,
      applied_std_days: 22
    }
  ]
}
```

### Test Cases (15 tests)

#### 1. Renders document container with correct ID
```
Verify:
- div#payslip-document exists
- Has class="bg-white p-8 md:p-16"
- max-w-[210mm] (A4 width) applied
```

#### 2. Displays company header with workspace name
```
Setup: workspace.name = "Tech Corp Vietnam"
Verify:
- Company name displays in header
- "Attendance & Payroll System" subtitle shows
- Building2 icon renders
- "Verified Document" badge displays
```

#### 3. Displays fallback workspace name when null
```
Setup: workspace = null
Verify:
- Displays "LeeAttend" as fallback
- Still renders valid document
```

#### 4. Shows "Pay Slip" title and period info
```
Verify:
- "Pay Slip" in large text (4xl font-black)
- CalendarDays icon displays
- Period label shows: "Month January/2026"
- Date range: "Period: 01/01/2026 - 31/01/2026"
```

#### 5. Displays employee details section
```
Setup: employee with full_name, employee_id_number, department_name
Verify:
- "Employee Details" header section
- Name displays: "NGUYỄN VĂN A" (uppercase)
- Employee ID: "EMP001"
- Department: "Engineering"
- All labels uppercase
```

#### 6. Renders Net Payment summary box
```
Verify:
- Large rounded box with "Net Payment" label
- Amount formatted as VND currency
- Green badge: "Paid via Bank Transfer"
- Styling: bg-slate-50, rounded-3xl
```

#### 7. Renders salary components breakdown (prorated calculation)
```
Setup: salary_components with 2 items
  Item 1: Code "BS", Name "Base Salary", Monthly "10000000"
  Item 2: Code "AL", Name "Allowance", Monthly "1000000"
  payslip.segments[0].applied_std_days = 22
  payslip.segments[0].paid_work_days = 20 (fewer than standard)

Verify table rows for each component:
- Component name displays: "Base Salary"
- Shows calculation: "Monthly: 10,000,000 • 20 Days @ 454,545/day"
- Prorated amount: (10000000 / 22) * 20 = 9,090,909
- Component code shows: "Code: BS"
- All amounts formatted as VND with commas
```

#### 8. Renders segments fallback when no salary_components
```
Setup: salary_components = [] or undefined
Verify:
- Segment rows render instead
- "Base Salary" or "Base Salary (Segment 2)" for multi-segment
- Shows period range: "2026-01-01 - 2026-01-31"
- Displays paid days and daily rate
```

#### 9. Renders overtime section when present
```
Setup: overtime_pay_amount = "500000", overtime_units = 10
Verify:
- Section title: "Overtime Compensation"
- Shows: "10 Units"
- Amount displays: "500,000"
- Only renders if overtime_pay_amount > 0
```

#### 10. Doesn't render overtime section when zero
```
Setup: overtime_pay_amount = "0"
Verify:
- Overtime section NOT rendered
```

#### 11. Renders incentives section when present
```
Setup: incentives = [
  { policy_name: "Diligence Bonus", amount: "500000" },
  { policy_name: "Safety Bonus", amount: "200000" }
]
Verify:
- Each incentive displays in table row
- Policy name: "Diligence Bonus"
- Amount: "500,000"
- Amber accent color (bg-amber-400)
- Only renders if incentives.length > 0
```

#### 12. Doesn't render incentives section when empty
```
Setup: incentives = []
Verify:
- Incentives section NOT rendered
```

#### 13. Calculates and displays gross pay correctly
```
Setup:
  base_pay_amount: "10000000"
  overtime_pay_amount: "500000"
  incentives: [{ amount: "300000" }]

Verify:
- Gross pay = 10000000 + 500000 + 300000 = 10800000
- Displays as: "10,800,000"
```

#### 14. Displays deductions and net pay
```
Verify:
- Deductions line: "-{deductions_amount}"
- Red color for deductions (text-rose-600)
- Net Pay = Gross Pay - Deductions
- Net pay displayed with emphasis
```

#### 15. Renders footer with timestamp
```
Verify:
- "Generated by LeeAttend System" text
- Current ISO timestamp displays
- Footer styled with opacity-50
- Timestamp in ISO 8601 format
```

### Currency Formatting Test
```
Verify for all amounts:
- Uses Intl.NumberFormat('vi-VN')
- style: 'currency'
- currency: 'VND'
- minimumFractionDigits: 0
- Examples:
  - 1000000 → "1,000,000 ₫"
  - 454545 → "454,545 ₫"
  - 10800000 → "10,800,000 ₫"
```

---

## Edge Cases & Error Scenarios

### For Both Components

#### 1. Missing or Malformed Data
```
Test with:
- employee.full_name = "" (empty)
- payslip.period_start = null
- payslip.period_end = invalid date
- workspace = undefined

Expected: Component renders without crashing
```

#### 2. Very Large Amounts
```
Test with amounts > 1,000,000,000:
- Verify formatting still works
- No overflow or truncation
```

#### 3. Negative Amounts
```
Test with:
- deductions_amount = "-100000"
- Verify display doesn't double-negate
```

#### 4. String vs Number Types
```
PayslipDocument uses:
- parseFloat(payslip.base_pay_amount)
- payslip.segments[0].applied_base_salary (appears to be number)

Test both string and number inputs
```

---

## Accessibility Testing

### For PayslipModal
```
Verify:
- Dialog.Close button is keyboard accessible
- Print button is keyboard accessible
- Tab order logical
- Dialog has proper ARIA attributes from Radix
```

### For PayslipDocument
```
Verify:
- Heading hierarchy: h1 > h2 > h3
- Table headers use <th>
- Currency values readable by screen readers
- Images have alt text (icons from lucide-react)
```

---

## Testing Utilities Reference

### Mock useDateTimeFormatter
```typescript
vi.mock('@/hooks/use-datetime', () => ({
  useDateTimeFormatter: () => ({
    formatMonthYear: (date: string) => {
      const d = new Date(date);
      return `Tháng ${d.getMonth() + 1}/${d.getFullYear()}`;
    },
    formatDate: (date: string) => {
      const d = new Date(date);
      const day = String(d.getDate()).padStart(2, '0');
      const month = String(d.getMonth() + 1).padStart(2, '0');
      const year = d.getFullYear();
      return `${day}/${month}/${year}`;
    }
  })
}));
```

### Render with Providers Helper
```typescript
const renderWithProviders = (ui: React.ReactElement) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 } },
  });

  return render(
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <ToastProvider>
          {ui}
        </ToastProvider>
      </BrowserRouter>
    </QueryClientProvider>
  );
};
```

---

## Success Criteria

All tests pass with:
- ✅ No console errors or warnings
- ✅ No unintended API calls
- ✅ All assertions pass deterministically
- ✅ Coverage > 80% for both files
- ✅ No timeout failures
- ✅ Execution time < 5s per file

---

## Estimated Implementation Time

| Task | Time |
|------|------|
| PayslipModal.test.tsx | 45 minutes |
| PayslipDocument.test.tsx | 90 minutes |
| Debugging & fixing | 30 minutes |
| **Total** | **~3 hours** |
