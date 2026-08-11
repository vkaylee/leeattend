# Mobile responsive spacing normalization

## Rule IDs applied
- DESIGN-RESPONSIVE: mobile-first layout with consistent responsive spacing and no horizontal overflow.
- CODING-DRY: shared layout primitives over repeated page-shell utility strings.
- CODING-FRONTEND-ARCH: keep layout concerns in reusable UI components and keep feature pages focused on content.
- TESTING-RESPONSIVE: verify frontend lint, unit, and integration/E2E responsive coverage through `./leedevkit test web`.

## Scope
Introduce a shared authenticated page container and improve the shared page header so mobile gutters, vertical rhythm, and action wrapping are consistent without adding horizontal padding on top of `AppShell`'s existing main padding. Migrate representative/high-frequency page shells and form cards, then extend focused tests.

## Acceptance criteria
- Authenticated pages use one shared responsive vertical page shell instead of repeating shell utility strings.
- Mobile content maintains a 16px minimum gutter through the existing AppShell padding and does not receive accidental double horizontal padding.
- Header actions wrap or stack safely on narrow viewports and retain keyboard-visible focus styles.
- Form cards use a shared responsive padding primitive.
- Existing responsive tests and full frontend verification pass via leedevkit.
