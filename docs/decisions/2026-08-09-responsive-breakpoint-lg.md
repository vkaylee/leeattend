# ADR: Standardize desktop/mobile chrome at `lg`

- **Status:** Accepted
- **Date:** 2026-08-09
- **Decision:** Use Tailwind's `lg` breakpoint (1024px) for the desktop navigation chrome and desktop/mobile presentation boundary.

## Context

The product standards define screens below 1024px as mobile navigation, while the app shell previously switched the icon rail, floating panel, bottom navigation, and shell offset at `md` (768px). Data tables and navigable lists already defaulted to `lg`, which created a mixed tablet layout between 768px and 1023px: desktop chrome consumed rail/panel width while content rendered mobile cards.

## Decision details

- `lg` is the source breakpoint for icon rail, floating panel, bottom navigation, shell rail offset, and desktop-oriented navigable-list chrome.
- `ResponsiveTable`, `ResponsiveToolbar`, and `NavigableList` retain their explicit breakpoint APIs for specialized consumers, but their defaults remain `lg`.
- `md` remains valid for unrelated spacing, grid, and content-density adjustments where it does not switch the desktop/mobile chrome or presentation mode.

## UX and accessibility impact

- Pixel 7 and tablet portrait viewports use bottom navigation and mobile cards, avoiding a narrow icon-rail-plus-panel workspace.
- A 1024px viewport is the first width that exposes the icon rail and desktop panel, matching the documented 1024px threshold.
- Touch users do not need to decode an unlabeled desktop rail in the 768px–1023px range; mobile navigation provides text labels and larger touch targets.

## Guidance for new components

- Use `lg:hidden` for mobile-only alternatives and `hidden lg:block`/`hidden lg:flex` for desktop-only presentation when the component changes interaction model or navigation chrome.
- Keep `md:` utilities for spacing or grid refinements only when both presentations remain functionally equivalent.
- Add responsive tests at 800px and 1024px whenever a component participates in the app shell or changes between table/card presentations.

## Consequences

The rail and optional floating panel can consume up to 296px beginning at 1024px. Desktop pages must continue to use `min-w-0`, overflow-safe containers, and existing table scroll affordances so large data sets remain usable without horizontal page overflow.
