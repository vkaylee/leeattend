# Implementation Plan: Roster Pattern System (2026-01-16)

## Problem
Assigning shifts to thousands of employees individually is unscalable. We need a way to assign "Working Patterns" (templates) to employees.

## Proposed Schema

### 1. `working_patterns`
- `id`: UUID (PK)
- `workspace_id`: UUID (FK)
- `name`: TEXT
- `description`: TEXT
- `is_active`: BOOLEAN DEFAULT true
- `created_at`: TIMESTAMP
- `updated_at`: TIMESTAMP

### 2. `working_pattern_details`
- `id`: UUID (PK)
- `pattern_id`: UUID (FK)
- `day_of_week`: INT (0=Sunday, 6=Saturday)
- `shift_id`: UUID (FK, NULL = Day Off)

### 3. `employee_working_patterns` (Assignments)
- `id`: UUID (PK)
- `employee_id`: UUID (FK)
- `pattern_id`: UUID (FK)
- `start_date`: DATE
- `end_date`: DATE (NULL = Forever)
- `created_at`: TIMESTAMP

## Logic Update
In `roster_repository.rs`, the `get_effective_shift` priority should be:
1. `shift_rosters` (Manual Override)
2. `employee_working_patterns` (New Pattern System)
3. `employee_shifts` (Legacy Fixed)
4. `department_default_shifts` (Fallback)

## Steps
1. Create Migration.
2. Update `models.rs`.
3. Create `working_pattern_repository.rs`.
4. Update `roster_repository.rs`.
5. Create `working_pattern_service.rs` and `handlers/working_patterns.rs`.
6. Add integration tests.
