# Phase 6: Testing

## Context Links
- [Phase 1: Foundation](phase-01-foundation.md) (hook + context to test)
- [Phase 5: Toggle](phase-05-settings-and-toggle.md) (UI to test)
- [Existing use-theme tests](../../webdashboard/src/hooks/__tests__/use-theme.test.tsx)
- [Existing Modal tests](../../webdashboard/src/components/ui/Modal.test.tsx)

## Overview
- **Priority**: P2
- **Status**: pending
- **Effort**: 1h
- **Description**: Unit tests for `useDarkMode` hook and `DarkModeToggle` component. Visual verification checklist.

## Key Insights
- Test framework: Vitest + React Testing Library (existing pattern in `__tests__/` dirs)
- All tests run in Docker containers via `./run-tests.sh --web`
- `useDarkMode` depends on `localStorage` and `matchMedia` — both need mocking
- Toggle component needs `DarkModeProvider` wrapper in test setup
- Visual regression via Playwright optional; manual checklist sufficient for initial release

## Requirements

### Functional
- F1: Unit tests for `useDarkMode` hook: initialization, mode cycling, persistence, system preference, cross-tab sync
- F2: Unit tests for `DarkModeToggle`: render, click cycle, icon change
- F3: Integration test: toggle in Navbar changes mode, reflected in Settings
- F4: Visual QA checklist covering all routes

### Non-Functional
- NF1: Tests run inside Docker containers (WSL constraint)
- NF2: No flaky tests from timing issues with matchMedia mocking

## Architecture

### Test Structure
```
webdashboard/src/
  hooks/__tests__/
    use-dark-mode.test.tsx          # Hook unit tests
  components/ui/__tests__/
    DarkModeToggle.test.tsx         # Component tests
```

### matchMedia Mock
```typescript
function mockMatchMedia(matches: boolean) {
  const listeners: Array<(e: MediaQueryListEvent) => void> = [];
  Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: jest.fn().mockImplementation((query: string) => ({
      matches,
      media: query,
      onchange: null,
      addListener: jest.fn(),
      removeListener: jest.fn(),
      addEventListener: (_: string, cb: any) => listeners.push(cb),
      removeEventListener: (_: string, cb: any) => {
        const idx = listeners.indexOf(cb);
        if (idx >= 0) listeners.splice(idx, 1);
      },
      dispatchEvent: jest.fn(),
    })),
  });
  return { trigger: (matches: boolean) => listeners.forEach(cb => cb({ matches } as any)) };
}
```

## Related Code Files

### Files to Create
| File | Purpose |
|------|---------|
| `src/hooks/__tests__/use-dark-mode.test.tsx` | Hook unit tests |
| `src/components/ui/__tests__/DarkModeToggle.test.tsx` | Toggle component tests |

## Implementation Steps

### Step 1: `use-dark-mode.test.tsx`

```typescript
import { renderHook, act } from '@testing-library/react';
import { useDarkMode } from '../use-dark-mode';

describe('useDarkMode', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove('dark');
  });

  it('defaults to system mode when no localStorage value', () => {
    mockMatchMedia(false);
    const { result } = renderHook(() => useDarkMode());
    expect(result.current.mode).toBe('system');
    expect(result.current.resolved).toBe('light');
  });

  it('resolves system mode to dark when OS prefers dark', () => {
    mockMatchMedia(true);
    const { result } = renderHook(() => useDarkMode());
    expect(result.current.resolved).toBe('dark');
    expect(result.current.isDark).toBe(true);
  });

  it('reads mode from localStorage', () => {
    localStorage.setItem('theme_mode', 'dark');
    mockMatchMedia(false);
    const { result } = renderHook(() => useDarkMode());
    expect(result.current.mode).toBe('dark');
    expect(result.current.resolved).toBe('dark');
  });

  it('setMode updates localStorage and DOM class', () => {
    mockMatchMedia(false);
    const { result } = renderHook(() => useDarkMode());
    act(() => result.current.setMode('dark'));
    expect(localStorage.getItem('theme_mode')).toBe('dark');
    expect(document.documentElement.classList.contains('dark')).toBe(true);
  });

  it('setMode to light removes dark class', () => {
    mockMatchMedia(false);
    document.documentElement.classList.add('dark');
    const { result } = renderHook(() => useDarkMode());
    act(() => result.current.setMode('light'));
    expect(document.documentElement.classList.contains('dark')).toBe(false);
  });

  it('responds to OS preference change when mode is system', () => {
    const { trigger } = mockMatchMedia(false);
    const { result } = renderHook(() => useDarkMode());
    expect(result.current.resolved).toBe('light');
    act(() => trigger(true));
    expect(document.documentElement.classList.contains('dark')).toBe(true);
  });
});
```

### Step 2: `DarkModeToggle.test.tsx`

```typescript
import { render, screen, fireEvent } from '@testing-library/react';
import { DarkModeToggle } from '../DarkModeToggle';
import { DarkModeProvider } from '@/context/DarkModeContext';

function renderToggle() {
  return render(
    <DarkModeProvider>
      <DarkModeToggle />
    </DarkModeProvider>
  );
}

describe('DarkModeToggle', () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove('dark');
    mockMatchMedia(false);
  });

  it('renders with data-testid', () => {
    renderToggle();
    expect(screen.getByTestId('dark-mode-toggle')).toBeInTheDocument();
  });

  it('cycles through modes on click', () => {
    renderToggle();
    const btn = screen.getByTestId('dark-mode-toggle');
    // Default: system -> click -> light
    fireEvent.click(btn); // system -> light
    expect(localStorage.getItem('theme_mode')).toBe('light');
    fireEvent.click(btn); // light -> dark
    expect(localStorage.getItem('theme_mode')).toBe('dark');
    fireEvent.click(btn); // dark -> system
    expect(localStorage.getItem('theme_mode')).toBe('system');
  });
});
```

Note: Cycle order is `light -> dark -> system` per Phase 5 implementation. Adjust initial state in test based on actual default (`system` -> first click goes to next in cycle).

### Step 3: Run Tests

```bash
./run-tests.sh --web
```

Or directly in container:
```bash
docker compose -f docker-compose.test.yml exec web_test npx vitest run src/hooks/__tests__/use-dark-mode.test.tsx
docker compose -f docker-compose.test.yml exec web_test npx vitest run src/components/ui/__tests__/DarkModeToggle.test.tsx
```

### Step 4: Visual QA Checklist

Manual verification in browser (toggle dark mode, visit each route):

| Route | Status |
|-------|--------|
| `/` (Dashboard) | [ ] |
| `/employees` | [ ] |
| `/departments` | [ ] |
| `/attendance-logs` | [ ] |
| `/leave-requests` | [ ] |
| `/leave-balances` | [ ] |
| `/shifts` | [ ] |
| `/payroll` | [ ] |
| `/reports` | [ ] |
| `/agents` | [ ] |
| `/devices` | [ ] |
| `/workspace/settings/general` | [ ] |
| `/workspace/settings/people` | [ ] |
| `/workspace/settings/time` | [ ] |
| `/workspace/settings/payroll` | [ ] |
| `/workspace/settings/connectivity` | [ ] |
| `/profile` | [ ] |
| `/ess/profile` | [ ] |
| `/ess/attendance` | [ ] |
| `/ess/clock` | [ ] |
| Login page | [ ] |
| Modal (any) | [ ] |
| Toast (trigger one) | [ ] |
| Combobox dropdown | [ ] |
| Sheet/slide panel | [ ] |

### Step 5: FOUC Test

1. Set theme to dark mode
2. Hard refresh (Ctrl+Shift+R)
3. Verify NO flash of light theme before dark renders
4. Repeat for system mode with OS set to dark

## Todo List
- [ ] Create `use-dark-mode.test.tsx` with all test cases
- [ ] Create `DarkModeToggle.test.tsx`
- [ ] Run tests via `./run-tests.sh --web`
- [ ] Fix any failing tests
- [ ] Manual visual QA on all routes (both modes)
- [ ] FOUC verification test
- [ ] Verify cross-tab sync works

## Success Criteria
- All unit tests pass
- `useDarkMode` hook: 6+ test cases covering init, cycling, persistence, system preference, cross-tab
- `DarkModeToggle` component: 3+ test cases covering render, cycling, data-testid
- Visual QA: no white patches on any route in dark mode
- FOUC: no flash on hard refresh

## Risk Assessment
| Risk | Impact | Mitigation |
|------|--------|------------|
| matchMedia mock complexity | Flaky tests | Use well-tested mock pattern above |
| i18n not loaded in test env | Missing translations | Mock `useTranslation` or use test i18n config |
| Docker container missing web test service | Can't run tests | Verify `docker-compose.test.yml` has web service |

## Security Considerations
- No security impact in tests.

## Next Steps
- After all tests pass and visual QA complete, dark mode feature is ready for merge
- Consider: Playwright screenshot regression tests as follow-up (not MVP)
