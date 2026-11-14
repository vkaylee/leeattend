# LeeAttend UI viewport contract

LeeAttend UI MUST support CSS viewport widths from `360px` through `1920px`. Supported regression viewports live in [`config/ui-viewports.json`](../config/ui-viewports.json), which is the single source of truth for Playwright and browser-assisted harnesses.

## Browser and MCP procedure

Before browser interaction:

1. Read `config/ui-viewports.json`.
2. Choose a named viewport from `supported` for the scenario.
3. Announce name and dimensions.
4. For MCP Playwright, call `browser_resize` with `width` and `height` from the manifest.
5. Verify `window.innerWidth` and `window.innerHeight`.
6. Report viewport name, dimensions, console evidence, and network evidence.

Default baseline: `desktop-large` (`1920×1080`). Do not invent viewport sizes unless testing an explicit breakpoint.

## Required responsive checks

At every selected viewport, check:

- no horizontal page overflow;
- no blank screen, crash, or repeated runtime error;
- navigation matches the `1024px` desktop boundary;
- tables, dialogs, forms, and primary actions stay usable inside viewport;
- mobile interactive controls remain at least `44px` tall;
- console and network contain no unexpected errors.

The viewport list is a regression matrix, not a limit on real browser sizes. Components MUST remain fluid between listed widths.
