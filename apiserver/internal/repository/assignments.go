package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type EmployeeAssignmentRepository struct {
	DB DBTX
}

func scanEmployeeAssignment(row interface{ Scan(...any) error }) (api.EmployeeAssignment, error) {
	var item api.EmployeeAssignment
	var created, updated time.Time
	var status string
	if err := row.Scan(&item.ID, &item.HomeLocationID, &item.HostLocationID, &item.StartDate, &item.EndDate, &item.Reason, &status, &created, &updated, &item.EmploymentID); err != nil {
		return api.EmployeeAssignment{}, err
	}
	item.Status = api.AssignmentStatus(status)
	item.CreatedAt = created.UTC().Format(time.RFC3339)
	item.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return item, nil
}

const employeeAssignmentColumns = `ea.id, ea.home_location_id, ea.host_location_id,
	ea.start_date::text, ea.end_date::text, ea.reason, ea.status::text,
	ea.created_at, ea.updated_at, ea.employment_id`

func (r EmployeeAssignmentRepository) ListAssignments(ctx context.Context) ([]api.EmployeeAssignmentWithEmployee, error) {
	db, err := requireDB(r.DB, "employee assignment repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT `+employeeAssignmentColumns+`,
		e.id, e.full_name, COALESCE(t.employee_number, ''),
		COALESCE(hl.name, ''), COALESCE(hostl.name, '')
		FROM employee_assignments ea
		JOIN employments em ON em.id = ea.employment_id
		JOIN employees e ON e.id = em.employee_id
		LEFT JOIN employment_relationships er ON er.id = em.relationship_id AND er.employee_id = em.employee_id
		LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= ea.start_date AND (t.end_date IS NULL OR t.end_date >= ea.start_date)
		LEFT JOIN work_locations hl ON hl.id = ea.home_location_id
		LEFT JOIN work_locations hostl ON hostl.id = ea.host_location_id
		WHERE em.deleted_at IS NULL AND e.deleted_at IS NULL
		ORDER BY ea.start_date DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	items := make([]api.EmployeeAssignmentWithEmployee, 0)
	for rows.Next() {
		var item api.EmployeeAssignmentWithEmployee
		var created, updated time.Time
		var status string
		if err := rows.Scan(
			&item.ID, &item.HomeLocationID, &item.HostLocationID,
			&item.StartDate, &item.EndDate, &item.Reason, &status,
			&created, &updated, &item.EmploymentID,
			&item.EmployeeID, &item.EmployeeName, &item.EmployeeCode,
			&item.HomeLocationName, &item.HostLocationName,
		); err != nil {
			return nil, err
		}
		item.Status = api.AssignmentStatus(status)
		item.CreatedAt = created.UTC().Format(time.RFC3339)
		item.UpdatedAt = updated.UTC().Format(time.RFC3339)
		items = append(items, item)
	}
	return items, rows.Err()
}

func (r EmployeeAssignmentRepository) ListEmployeeAssignments(ctx context.Context, employeeID uuid.UUID) ([]api.EmployeeAssignment, error) {
	db, err := requireDB(r.DB, "employee assignment repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT `+employeeAssignmentColumns+` FROM employee_assignments ea
		JOIN employments em ON em.id = ea.employment_id
		WHERE em.employee_id = $1
		ORDER BY em.start_date DESC, ea.start_date DESC`, employeeID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	items := make([]api.EmployeeAssignment, 0)
	for rows.Next() {
		item, err := scanEmployeeAssignment(rows)
		if err != nil {
			return nil, err
		}
		items = append(items, item)
	}
	return items, rows.Err()
}

func (r EmployeeAssignmentRepository) GetEmployeeAssignment(ctx context.Context, employeeID, assignmentID uuid.UUID) (api.EmployeeAssignment, error) {
	db, err := requireDB(r.DB, "employee assignment repository")
	if err != nil {
		return api.EmployeeAssignment{}, err
	}
	item, err := scanEmployeeAssignment(db.QueryRow(ctx, `SELECT `+employeeAssignmentColumns+` FROM employee_assignments ea
		JOIN employments em ON em.id = ea.employment_id
		WHERE ea.id = $1 AND em.employee_id = $2`, assignmentID, employeeID))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmployeeAssignment{}, api.ErrEmployeeAssignmentNotFound
	}
	return item, err
}

func (r EmployeeAssignmentRepository) GetActiveEmployment(ctx context.Context, employeeID uuid.UUID, onDate string) (api.ActiveEmployment, error) {
	db, err := requireDB(r.DB, "employee assignment repository")
	if err != nil {
		return api.ActiveEmployment{}, err
	}
	var item api.ActiveEmployment
	err = db.QueryRow(ctx, `SELECT id, primary_location_id FROM employments
		WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL
		AND start_date <= $2::date AND (end_date IS NULL OR end_date >= $2::date)
		ORDER BY start_date DESC, id LIMIT 1`, employeeID, onDate).Scan(&item.ID, &item.PrimaryLocationID)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.ActiveEmployment{}, api.ErrNoActiveEmployment
	}
	return item, err
}

func (r EmployeeAssignmentRepository) CreateEmployeeAssignment(ctx context.Context, employmentID, homeLocationID uuid.UUID, req api.CreateEmployeeAssignmentRequest) (api.EmployeeAssignment, error) {
	db, err := requireDB(r.DB, "employee assignment repository")
	if err != nil {
		return api.EmployeeAssignment{}, err
	}
	if err := guardLifecycleDateRange(ctx, db, []uuid.UUID{employmentID}, req.StartDate, patternEndDate(req.StartDate, req.EndDate)); err != nil {
		return api.EmployeeAssignment{}, finishLifecycleMutationDB(ctx, db, err)
	}
	id := uuid.New()
	item, err := scanEmployeeAssignment(db.QueryRow(ctx, `INSERT INTO employee_assignments
		(id, home_location_id, host_location_id, start_date, end_date, reason, status, created_at, updated_at, employment_id)
		VALUES ($1, $2, $3, $4::date, $5::date, $6, 'active', NOW(), NOW(), $7)
		RETURNING id, home_location_id, host_location_id, start_date::text, end_date::text, reason, status::text, created_at, updated_at, employment_id`, id, homeLocationID, req.HostLocationID, req.StartDate, nullableDate(req.EndDate), req.Reason, employmentID))
	if err != nil {
		return api.EmployeeAssignment{}, mapConstraintError(err)
	}
	return item, nil
}

func (r EmployeeAssignmentRepository) UpdateEmployeeAssignment(ctx context.Context, employeeID, assignmentID uuid.UUID, req api.UpdateEmployeeAssignmentRequest) (api.EmployeeAssignment, error) {
	db, err := requireDB(r.DB, "employee assignment repository")
	if err != nil {
		return api.EmployeeAssignment{}, err
	}
	var assignmentEmploymentID uuid.UUID
	var startDate, endDate string
	if err := db.QueryRow(ctx, `SELECT employment_id, start_date::text, COALESCE(end_date, start_date)::text FROM employee_assignments WHERE id = $1`, assignmentID).Scan(&assignmentEmploymentID, &startDate, &endDate); err != nil {
		return api.EmployeeAssignment{}, err
	}
	if err := guardLifecycleDateRange(ctx, db, []uuid.UUID{assignmentEmploymentID}, startDate, endDate); err != nil {
		return api.EmployeeAssignment{}, finishLifecycleMutationDB(ctx, db, err)
	}
	sets := []string{"ea.updated_at = NOW()"}
	args := make([]any, 0, 5)
	add := func(col string, value any) {
		args = append(args, value)
		sets = append(sets, fmt.Sprintf("ea.%s = $%d", col, len(args)))
	}
	if req.EndDatePresent {
		if req.EndDate == nil || *req.EndDate == nil {
			sets = append(sets, "ea.end_date = NULL")
		} else {
			add("end_date", **req.EndDate)
		}
	}
	if req.Status != nil {
		add("status", string(*req.Status))
	}
	if req.ReasonPresent {
		if req.Reason == nil || *req.Reason == nil {
			sets = append(sets, "ea.reason = NULL")
		} else {
			add("reason", **req.Reason)
		}
	}
	args = append(args, assignmentID, employeeID)
	query := fmt.Sprintf(`UPDATE employee_assignments ea SET %s
		FROM employments em
		WHERE ea.employment_id = em.id AND ea.id = $%d
		AND em.employee_id = $%d
		RETURNING ea.id, ea.home_location_id, ea.host_location_id, ea.start_date::text, ea.end_date::text, ea.reason, ea.status::text, ea.created_at, ea.updated_at, ea.employment_id`, strings.Join(sets, ", "), len(args)-1, len(args))
	item, err := scanEmployeeAssignment(db.QueryRow(ctx, query, args...))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmployeeAssignment{}, api.ErrEmployeeAssignmentNotFound
	}
	return item, err
}

func (r EmployeeAssignmentRepository) DeleteEmployeeAssignment(ctx context.Context, employeeID, assignmentID uuid.UUID) error {
	db, err := requireDB(r.DB, "employee assignment repository")
	if err != nil {
		return err
	}
	var assignmentEmploymentID uuid.UUID
	var startDate, endDate string
	if err := db.QueryRow(ctx, `SELECT employment_id, start_date::text, COALESCE(end_date, start_date)::text FROM employee_assignments WHERE id = $1`, assignmentID).Scan(&assignmentEmploymentID, &startDate, &endDate); err != nil {
		return err
	}
	if err := guardLifecycleDateRange(ctx, db, []uuid.UUID{assignmentEmploymentID}, startDate, endDate); err != nil {
		return finishLifecycleMutationDB(ctx, db, err)
	}
	result, err := db.Exec(ctx, `DELETE FROM employee_assignments ea USING employments em
		WHERE ea.employment_id = em.id AND ea.id = $1
		AND em.employee_id = $2`, assignmentID, employeeID)
	if err != nil {
		return err
	}
	if result.RowsAffected() == 0 {
		return api.ErrEmployeeAssignmentNotFound
	}
	return nil
}

type AssignmentRuleRepository struct {
	DB DBTX
}

func scanAssignmentRule(row interface{ Scan(...any) error }) (api.AssignmentRule, error) {
	var item api.AssignmentRule
	var conditions []byte
	var created, updated *time.Time
	if err := row.Scan(&item.ID, &item.PatternID, &item.Name, &item.Priority, &conditions, &item.IsActive, &created, &updated, &item.StartDate, &item.EndDate); err != nil {
		return api.AssignmentRule{}, err
	}
	if err := json.Unmarshal(conditions, &item.Conditions); err != nil {
		return api.AssignmentRule{}, err
	}
	if created != nil {
		item.CreatedAt = created.UTC().Format(time.RFC3339)
	}
	if updated != nil {
		item.UpdatedAt = updated.UTC().Format(time.RFC3339)
	}
	return item, nil
}

const assignmentRuleColumns = `id, pattern_id, name, priority, conditions,
	is_active, created_at, updated_at, start_date::text, end_date::text`

func (r AssignmentRuleRepository) ListAssignmentRules(ctx context.Context) ([]api.AssignmentRule, error) {
	db, err := requireDB(r.DB, "assignment rule repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT `+assignmentRuleColumns+` FROM assignment_rules
		ORDER BY priority DESC, created_at ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	items := make([]api.AssignmentRule, 0)
	for rows.Next() {
		item, err := scanAssignmentRule(rows)
		if err != nil {
			return nil, err
		}
		items = append(items, item)
	}
	return items, rows.Err()
}

func marshalConditions(conditions api.AssignmentRuleConditions) ([]byte, error) {
	return json.Marshal(conditions)
}

func (r AssignmentRuleRepository) CreateAssignmentRule(ctx context.Context, req api.CreateAssignmentRuleRequest) (api.AssignmentRule, error) {
	db, err := requireDB(r.DB, "assignment rule repository")
	if err != nil {
		return api.AssignmentRule{}, err
	}
	if err := guardLifecycleGlobalMutation(ctx, db); err != nil {
		return api.AssignmentRule{}, finishLifecycleMutationDB(ctx, db, err)
	}
	conditions, err := marshalConditions(req.Conditions)
	if err != nil {
		return api.AssignmentRule{}, err
	}
	id := uuid.New()
	item, err := scanAssignmentRule(db.QueryRow(ctx, `INSERT INTO assignment_rules
		(id, pattern_id, name, priority, conditions, is_active, created_at, updated_at, start_date, end_date)
		VALUES ($1, $2, $3, $4, $5::jsonb, TRUE, NOW(), NOW(), $6::date, $7::date)
		RETURNING `+assignmentRuleColumns, id, req.PatternID, req.Name, req.Priority, conditions, nullableDate(req.StartDate), nullableDate(req.EndDate)))
	if err != nil {
		return api.AssignmentRule{}, mapConstraintError(err)
	}
	return item, nil
}

func (r AssignmentRuleRepository) UpdateAssignmentRule(ctx context.Context, ruleID uuid.UUID, req api.UpdateAssignmentRuleRequest) (api.AssignmentRule, error) {
	db, err := requireDB(r.DB, "assignment rule repository")
	if err != nil {
		return api.AssignmentRule{}, err
	}
	if err := guardLifecycleGlobalMutation(ctx, db); err != nil {
		return api.AssignmentRule{}, finishLifecycleMutationDB(ctx, db, err)
	}
	sets := []string{"updated_at = NOW()"}
	args := make([]any, 0, 9)
	add := func(col string, value any) {
		args = append(args, value)
		sets = append(sets, fmt.Sprintf("%s = $%d", col, len(args)))
	}
	if req.PatternID != nil {
		add("pattern_id", *req.PatternID)
	}
	if req.Name != nil {
		add("name", *req.Name)
	}
	if req.Priority != nil {
		add("priority", *req.Priority)
	}
	if req.Conditions != nil {
		conditions, marshalErr := marshalConditions(*req.Conditions)
		if marshalErr != nil {
			return api.AssignmentRule{}, marshalErr
		}
		add("conditions", conditions)
	}
	if req.IsActive != nil {
		add("is_active", *req.IsActive)
	}
	if req.StartDatePresent {
		if req.StartDate == nil || *req.StartDate == nil {
			sets = append(sets, "start_date = NULL")
		} else {
			add("start_date", **req.StartDate)
		}
	}
	if req.EndDatePresent {
		if req.EndDate == nil || *req.EndDate == nil {
			sets = append(sets, "end_date = NULL")
		} else {
			add("end_date", **req.EndDate)
		}
	}
	args = append(args, ruleID)
	query := fmt.Sprintf(`UPDATE assignment_rules SET %s WHERE id = $%d RETURNING `+assignmentRuleColumns, strings.Join(sets, ", "), len(args))
	item, err := scanAssignmentRule(db.QueryRow(ctx, query, args...))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.AssignmentRule{}, api.ErrAssignmentRuleNotFound
	}
	return item, err
}

func (r AssignmentRuleRepository) DeleteAssignmentRule(ctx context.Context, ruleID uuid.UUID) error {
	db, err := requireDB(r.DB, "assignment rule repository")
	if err != nil {
		return err
	}
	if err := guardLifecycleGlobalMutation(ctx, db); err != nil {
		return finishLifecycleMutationDB(ctx, db, err)
	}
	_, err = db.Exec(ctx, `DELETE FROM assignment_rules WHERE id = $1`, ruleID)
	return err
}

func nullableDate(value *string) any {
	if value == nil || *value == "" {
		return nil
	}
	return *value
}
