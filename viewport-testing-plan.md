# Viewport testing contract

- `UI-VP-001`: `config/ui-viewports.json` is single source of truth.
- `UI-VP-002`: Viewport tiers identify required desktop, secondary tablet, and mobile smoke coverage.
- `UI-VP-003`: Playwright projects consume every supported regression viewport.
- `UI-VP-004`: MCP role wrapper exposes manifest viewport guidance before browser interaction.
- `UI-VP-005`: UI audit records viewport, console, and network evidence.

Test impact: config parsing, tier membership and completeness, supported width boundaries (`360px`–`1920px`), duplicate viewport identity, Playwright project compatibility, MCP initialization guidance, responsive overflow, navigation boundary, dialog/form containment, and mobile touch target behavior.
