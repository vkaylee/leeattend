# Technical debt: retire employee create modal wiring

- **Evidence:** `/employees/new` owns employee creation, while `EmployeeFormModal` still implements create mode, fetches organization members, and sends `user_id`. Employees page state still parses `enrollment_id` for a modal path.
- **Outcome:** Employee creation uses dedicated page only. Employee edit modal remains edit-only and cannot mutate account links. Members remains sole link/unlink UI.
- **Files:** employee edit modal, employee overlay/state/data hooks, modal tests, overlay tests; remove dead wrapper.
- **Smallest fix:** delete obsolete create/member-link branches and compatibility state; preserve non-create overlays and edit employment updates.
- **Rules:** coding standards dead-code policy; development workflow behavior-verification loop; testing standards full configured target; project structure dependency direction.
- **Acceptance:** no active create-modal or organization-member fetch references; edit modal sends no `user_id`; existing edit and non-create overlay tests pass; full web target passes.
