# Migration phase object matrix

Status: catalog classification; no phase SQL generated.

Source catalog: disposable schema-only export described in `migration-catalog-oracle.md`.
Generated child partitions belong to their parent phase and are listed once.

## Ownership rules

- `foundation` owns shared infrastructure, identity, configuration, audit, workflow, and common entities required by foreign keys.
- `hr_core` owns people, employment, leave, overtime, recruitment, documents, and employee lifecycle.
- `attendance_schedule` owns attendance, shifts, scheduling, time entries, closures, and schedule evidence.
- `payroll` owns payroll calculation, policy, salary, payslips, and payroll reconciliation.
- `runtime_seeds` owns rows only; it owns no tables, indexes, functions, triggers, or views.
- Existing baseline ownership is historical source location, not final phase ownership. Final phase SQL must preserve cross-phase foreign keys and dependency order.

## Foundation

### Shared identity and authorization

```text
organization_settings
users
roles
role_permissions
user_roles
user_extra_roles
user_extra_permissions
sessions
behalf_assignments
authorized_agents
```

### Configuration, storage, audit, and runtime infrastructure

```text
config_entries
feature_flags
compliance_rules
files
file_acknowledgements
document_folders
system_files
system_settings
storage_configs
notifications
outbox_jobs
infra_job_events
infra_job_records
monitor_alerts
monitor_alert_events
system_audit_logs
schema_migrations
```

### Approval and shared workflow

```text
approval_workflows
approval_workflow_instances
approval_workflow_steps
approval_workflow_tasks
approval_action_log
signature_requests
```

### Shared definitions and cross-domain support

```text
condition_type_definitions
custom_field_definitions
custom_field_values
policy_templates
working_conditions
work_locations
lifecycle_checklist_templates
lifecycle_checklist_template_items
lifecycle_checklists
lifecycle_checklist_items
dashboard_preferences
```

### Audit partitions

```text
audit_logs
  + audit_logs_default
  + audit_logs_y2026m01 ... audit_logs_y2026m12
```

## HR core

### Organization and employee lifecycle

```text
employers
departments
employees
employee_profiles
employee_emergency_contacts
employments
employment_relationships
employee_contracts
employee_phases
employment_phase_definitions
employee_assignments
employee_tenures
employee_tenure_reconciliation
```

### Employee documents and compensation definitions

```text
employee_documents
salary_component_definitions
employee_salary_components
recruitment_sources
econtract_envelopes
employer_econtract_configs
employer_econtract_routing_rules
```

### Leave and overtime

```text
leave_types
leave_policy_definitions
leave_policy_assignments
leave_balances
leave_accrual_logs
leave_requests
overtime_policy_definitions
overtime_policy_assignments
overtime_rules
overtime_requests
overtime_request_employees
```

## Attendance and schedule

### Attendance sources and records

```text
attendance_sources
attendance_source_pull_states
external_attendance_events
raw_attendance_logs
attendance_import_jobs
attendance_log_errors
attendance_staging_logs
  + attendance_staging_logs_default
  + attendance_staging_logs_y2026m01 ... attendance_staging_logs_y2026m12
attendance_adjustment_requests
attendance_daily_summaries
attendance_periods
attendance_closures
attendance_closure_days
attendance_policies
attendance_policy_versions
attendance_policy_assignments
time_entries
holidays
```

### Shift and schedule source model

```text
shifts
shift_templates
shift_versions
shift_default_intervals
shift_rosters
employee_shifts
working_patterns
working_pattern_details
employee_working_patterns
assignment_rules
work_segments
shift_swap_requests
shift_audit_log
```

### Official schedule lifecycle and evidence

```text
schedule_previews
schedule_publication_batches
schedule_day_revisions
schedule_office_policies
schedule_shadow_mismatches
schedule_legacy_mutation_evidence
```

### Schedule-owned cross-domain object

```text
payroll_periods
```

`payroll_periods` is listed once here. Payroll consumes it; it is not recreated in payroll phase.

`payroll_periods` stays in this phase because schedule and attendance guards read payroll lock state while payroll period lifecycle reads attendance closure state. Moving it to `payroll` creates an installation dependency cycle.

## Payroll

### Payroll registry and calculation

```text
payroll_metric_definitions
payroll_input_batches
payroll_inputs
payroll_input_corrections
payroll_formula_rules
payroll_formula_rule_versions
payroll_calculation_runs
```

### Payroll policies and rates

```text
work_credit_policies
pay_basis_policies
labor_policy_groups
labor_policy_group_versions
labor_policy_memberships
```

### Payroll output and snapshots

```text
payslips
payslip_segments
payslip_incentives
payslip_rule_evaluations
payroll_rule_definitions
payroll_rule_tiers
payroll_rule_assignments
payroll_incentive_policies
payroll_incentive_tiers
payroll_incentive_assignments
```

### Payroll view

```text
department_payroll_reconciliation
```

View must be created after `departments`, `config_entries`, and final payroll policy/config constraints exist. Runtime consumers are `internal/repository/departments.go` and `department_payroll_reconciliation_integration_test.go`.

## Runtime seeds

No schema objects. Seed rows:

```text
feature_flags:
  ff_econtracts
  ff_ess
  schedule_lifecycle  # if required by final bootstrap contract

payroll_metric_definitions:
  production.output
  production.target_output
  production.quality_rate

config_entries:
  payroll.kpi.output_weight
  payroll.kpi.quality_weight
  payroll.kpi.target_bonus
  payroll.kpi.score_cap
  payroll.kpi.bonus_cap

post-bootstrap formula:
  payroll_formula_rules(code = 'kpi.bonus')
  payroll_formula_rule_versions(version = 1)
```

Formula seed runs after standalone bootstrap creates administrator. Generic migration must not select arbitrary first user or claim success with no inserted formula.

## Historical migration mapping

| Historical source | Final phase | Fresh-install treatment |
|---|---|---|
| `20261110000000_single_organization_baseline` | all phases, split by object ownership | regenerate final objects; do not preserve baseline marker as phase marker |
| `20261112000000_config_entries_integrity` | foundation | keep final constraints; omit duplicate cleanup |
| `20261112010000_employment_relationships` | hr_core | keep schema, constraints, triggers, indexes |
| `20261112020000_employment_relationship_backfill` | hr_core | omit data backfill |
| `20261112030000_payroll_periods` | attendance_schedule | keep schema and trigger |
| `20261113000000_schedule_lifecycle` | attendance_schedule | keep final schema and guards |
| `20261113010000_feature_flags_runtime` | runtime_seeds | move rows only |
| `20261113020000_retire_salary_base_groups` | hr_core/payroll cleanup | omit retired objects; do not run drop sequence |
| `20261113030000_payroll_input_registry` | payroll + runtime_seeds | split DDL from metric rows |
| `20261113040000_payroll_formula_versions` | payroll | keep schema |
| `20261113050000_payroll_calculation_run_links` | payroll | encode final columns/indexes |
| `20261114000000_payroll_policy_parameters` | runtime_seeds | replace date-dependent identity with deterministic seed strategy |
| `20261114010000_payroll_calculation_run_idempotency` | payroll | keep final constraints/indexes |
| `20261114020000_payroll_input_row_governance` | payroll | keep schema |
| `20261114030000_schedule_activation_ranges` | attendance_schedule | keep final exclusion constraint |
| `20261114040000_schedule_legacy_backfill` | attendance_schedule | omit data backfill; keep final state/provenance schema |
| `20261114050000_schedule_shadow_reconciliation` | attendance_schedule | keep schema |
| `20261114060000_payroll_kpi_formula_seed` | runtime_seeds/bootstrap | move after administrator bootstrap |
| `20261114070000_schedule_legacy_mutation_evidence` | attendance_schedule | keep schema |
| `20261114080000_labor_policy_groups` | payroll | keep schema |
| `20261114090000_payroll_calculation_run_policy_snapshot` | payroll | keep final snapshot state |
| `20261114100000_schedule_office_policies` | attendance_schedule | keep schema |
| `20261114110000_payroll_segment_policy_snapshots` | payroll | keep final column/index |
| `20261115000000_schedule_legacy_mutation_guards` | attendance_schedule | keep function/triggers |
| `20261115010000_department_payroll_reconciliation` | payroll | keep view |
| `20261115020000_pay_basis_rate_components` | payroll | keep final column/check |
| `20261115030000_employee_tenures` | hr_core | keep schema; omit existing-employee inserts |
| `20261115040000_ess_feature_flag` | runtime_seeds | move row only |
| `20261115050000_append_only_payroll_policy_snapshots` | payroll | encode final absence of superseded index |
| `20261116000000_atomic_official_shift_swaps` | attendance_schedule | keep final columns/constraints/index |
| `20261117000000_schedule_day_offsets` | attendance_schedule | keep final columns/checks |

## Dependency boundary strategy

The final schema has dependency cycles and cross-phase foreign keys. A phase cannot be implemented as `CREATE TABLE + all final FKs` in strict phase order.

Use two sections per phase:

```text
phase A: types, tables, local indexes, local checks, local functions/triggers
phase B: cross-phase and cyclic foreign keys, views, final indexes, marker
```

Keep each phase transaction atomic. Add a constraint only after both referenced tables exist. For a cross-phase FK, assign constraint creation to later phase in install order. Example: `departments.default_working_pattern_id -> working_patterns` is declared by HR but created in `attendance_schedule`. This preserves five phases without a hidden sixth finalization phase. Same-phase cycles are created after all tables in that phase exist. Do not weaken or omit final constraints.

Current catalog has 162 table/view objects, 178 FK edges, and 67 cross-phase FK edges. Full edge list: `migration-phase-cross-dependencies.tsv`. Object-to-source mapping: `migration-phase-source-map.tsv`.

Known dependency cycles/boundaries from current catalog:

| Boundary | Cycle or dependency | Resolution |
|---|---|---|
| HR ↔ attendance | `departments.default_working_pattern_id -> working_patterns`; departments otherwise belongs to HR while working patterns belongs to attendance | Create both tables in owning phases; add department FK during attendance final-constraint section |
| HR self-cycle | `employees -> recruitment_sources -> employees` | Create both tables; add both FKs after both tables exist in HR phase |
| Attendance self-cycle | `attendance_periods -> attendance_closures -> attendance_periods` | Create both tables; add both FKs after both tables exist in attendance phase |
| Attendance self-cycle | `shift_templates -> shift_versions -> shift_templates` | Create both tables; add both FKs after both tables exist in attendance phase |
| Attendance internal order | `work_segments -> schedule_day_revisions`, `shift_default_intervals`, `shift_templates`, `shift_versions` | Create referenced schedule/shift tables first; add work-segment FKs in final attendance section |
| Payroll → HR | payslips/segments/policy tables reference employments, departments, salary definitions | HR tables and keys must exist before payroll final constraints |
| Payroll → attendance | `work_credit_policies -> attendance_policies`; payroll period lifecycle depends on attendance closure | Keep `payroll_periods` in attendance; payroll runs after attendance |

Phase ownership means logical ownership, not permission to leave a broken intermediate schema. Every fresh install must end each phase with all objects required by that phase's contract and all constraints that can now be safely added. Cross-phase constraints must be listed in manifest and tested explicitly.

## Gaps before SQL generation

- Extract complete table definitions and classify each final FK as local, cross-phase, or cyclic.
- Define phase marker names and runner state model.
- Define bootstrap boundary for `runtime_seeds` formula data.
- Build fresh five-phase database and compare against catalog oracle.
- Decide whether all phase files remain human-readable sections or use generated SQL from catalog-normalized source; do not hand-copy 161 objects.
