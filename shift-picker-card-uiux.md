# Rule Compliance Check — Shift Picker Card UI/UX

Scope: the shift information card rendered inside the shift selection modal on `/shifts`
(`webdashboard/src/components/shifts/shift-picker-modal.tsx`).

## Rule IDs to Comply With

1. **Rule UI2.B: Keyboard Accessibility & Focus** (`design-rules.md`) — cards form a composite
   widget: `↑`/`↓`/`Home`/`End` roam focus between cards, `Enter`/`Space` select natively,
   `↑` from the first card returns focus to the search box. Every card keeps a visible
   `focus-visible:ring-2` ring.
2. **Rule UI2.C: Screen Reader Support** (`design-rules.md`) — each card exposes a descriptive
   `aria-label` (name, symbol, time range, working hours, shift kind). The mini 24h timeline is
   decorative and marked `aria-hidden`, since every value it encodes is present as text.
3. **Rule UI2.D: Multi-channel Cues** (`design-rules.md`) — shift kind is never conveyed by the
   colour strip alone; it always carries a text badge (night / split) and the overnight rollover
   is spelled out as `+1`.
4. **Rule CS6: Enterprise-Grade Premium Standard** (`coding-standards.md`) — premium micro
   interactions (hover lift, strip growth, `duration-200`), full i18n for all three locales,
   no hardcoded strings.
5. **Rule CS9: Frontend Architecture** (`coding-standards.md`) — all derived data lives in pure
   modules (`lib/shift-card-metrics.ts`) or the `useShiftPicker` hook; the card component stays
   presentational.

## Correctness defect fixed alongside (Boy Scout Rule)

`ShiftTimeBadge` read `intervals[0]` only, so a split shift `08:00–12:00 + 14:00–18:00`
rendered as a plain `08:00–12:00` card — under-reporting both the schedule and the hours.
The card now renders every interval and sums working time across all of them.
