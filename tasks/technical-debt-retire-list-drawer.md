# Technical debt: retire employee list drawer plumbing

- **Evidence:** Employee rows navigate to `/employees/$employeeId`, but list-page modal state still stores `selectedEmployee`, fetches department history, and exposes termination mutation for the removed list drawer. `EmployeeOverlays` still lazy-loads `EmployeeDetailDrawer` even when list page cannot provide one.
- **Outcome:** Employee list owns navigation and table selection only. Employee detail page owns detail data, drawer content, and employment overlays.
- **Files:** employee list/table, overlay component and tests, employee page hooks, detail-page overlay wiring.
- **Smallest fix:** remove unreachable drawer state/query/mutation and obsolete overlay props; preserve detail-page edit, transfer, terminate, rehire, and import flows.
- **Rules:** coding standards dead-code policy; development workflow behavior-verification loop; testing standards configured target and failure reporting; project structure dependency direction.
- **Acceptance:** no list-page `EmployeeDetailDrawer` or department-history query; no unused selected-employee drawer state; detail overlays retain current modal behavior; focused tests, typecheck, i18n, lint, and full web target pass.
