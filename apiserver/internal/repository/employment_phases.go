package repository

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type EmploymentPhaseRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r EmploymentPhaseRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("employment phase repository: database is not configured")
	}
	return r.DB, nil
}

func (r EmploymentPhaseRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	d, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	b, ok := d.(TxBeginner)
	if !ok {
		return nil, nil
	}
	return b, nil
}

func (r EmploymentPhaseRepository) CreateDefinition(ctx context.Context, def api.EmploymentPhaseDefinition) (api.EmploymentPhaseDefinition, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmploymentPhaseDefinition{}, err
	}
	var created, updated time.Time
	var out api.EmploymentPhaseDefinition
	var rc []byte
	err = db.QueryRow(ctx, `INSERT INTO employment_phase_definitions (
		id, name, description, phase_type, rule_config, is_active, display_order, created_at, updated_at
	) VALUES (
		$1, $2, $3, $4, $5, $6, $7, NOW(), NOW()
	) RETURNING id, name, description, phase_type, rule_config, is_active, display_order, created_at, updated_at`,
		def.ID, def.Name, def.Description, def.PhaseType, def.RuleConfig, def.IsActive, def.DisplayOrder,
	).Scan(
		&out.ID, &out.Name, &out.Description, &out.PhaseType, &rc, &out.IsActive, &out.DisplayOrder, &created, &updated,
	)
	if err != nil {
		return api.EmploymentPhaseDefinition{}, err
	}
	out.RuleConfig = json.RawMessage(rc)
	out.CreatedAt = created.UTC().Format(time.RFC3339)
	out.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return out, nil
}

func (r EmploymentPhaseRepository) ListDefinitions(ctx context.Context) ([]api.EmploymentPhaseDefinition, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		id, name, description, phase_type, rule_config, is_active, display_order, created_at, updated_at
	FROM employment_phase_definitions
	ORDER BY display_order ASC, created_at DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.EmploymentPhaseDefinition, 0)
	for rows.Next() {
		var item api.EmploymentPhaseDefinition
		var created, updated time.Time
		var rc []byte
		if err := rows.Scan(
			&item.ID, &item.Name, &item.Description, &item.PhaseType, &rc, &item.IsActive, &item.DisplayOrder, &created, &updated,
		); err != nil {
			return nil, err
		}
		item.RuleConfig = json.RawMessage(rc)
		item.CreatedAt = created.UTC().Format(time.RFC3339)
		item.UpdatedAt = updated.UTC().Format(time.RFC3339)
		result = append(result, item)
	}
	return result, rows.Err()
}

func scanPhase(row interface{ Scan(...any) error }) (api.EmployeePhase, error) {
	var p api.EmployeePhase
	var created, updated time.Time
	var completed *time.Time
	var oc []byte
	if err := row.Scan(
		&p.ID, &p.EmploymentID, &p.PhaseDefinitionID, &p.StartDate, &p.EndDate,
		&p.Status, &oc, &p.EvaluationNotes, &p.EvaluatedBy, &completed, &created, &updated,
	); err != nil {
		return api.EmployeePhase{}, err
	}
	if len(oc) > 0 && string(oc) != "null" {
		p.OverrideConfig = json.RawMessage(oc)
	} else {
		p.OverrideConfig = json.RawMessage(`{}`)
	}
	if completed != nil {
		formatted := completed.UTC().Format(time.RFC3339)
		p.CompletedAt = &formatted
	}
	p.CreatedAt = created.UTC().Format(time.RFC3339)
	p.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return p, nil
}

func (r EmploymentPhaseRepository) AssignPhase(ctx context.Context, employmentID uuid.UUID, req api.AssignEmployeePhaseRequest) (api.EmployeePhase, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.EmployeePhase{}, err
	}
	if txBeginner != nil {
		var created api.EmployeePhase
		err := service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
			res, txErr := r.assignPhaseTx(ctx, tx, employmentID, req)
			if txErr != nil {
				return txErr
			}
			created = res
			return nil
		})
		return created, err
	}
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeePhase{}, err
	}
	return r.assignPhaseTx(ctx, db, employmentID, req)
}

func (r EmploymentPhaseRepository) assignPhaseTx(ctx context.Context, db DBTX, employmentID uuid.UUID, req api.AssignEmployeePhaseRequest) (api.EmployeePhase, error) {
	startT, err := time.Parse("2006-01-02", req.StartDate)
	if err != nil {
		return api.EmployeePhase{}, err
	}
	autoCloseDate := startT.AddDate(0, 0, -1).Format("2006-01-02")

	_, err = db.Exec(ctx, fmt.Sprintf(`UPDATE employee_phases
		SET status = 'completed',
		    end_date = LEAST(end_date, GREATEST(start_date, '%s'::date)),
		    completed_at = NOW(),
		    updated_at = NOW()
		WHERE employment_id = $1
		  AND status = 'active'
		  AND start_date <= $2::date`, autoCloseDate), employmentID, req.StartDate)
	if err != nil {
		return api.EmployeePhase{}, err
	}

	var oc any = nil
	if len(req.OverrideConfig) > 0 && string(req.OverrideConfig) != "null" {
		oc = req.OverrideConfig
	}

	phaseID := uuid.New()
	return scanPhase(db.QueryRow(ctx, `INSERT INTO employee_phases (
		id, employment_id, phase_definition_id,
		start_date, end_date, status, override_config,
		evaluation_notes, evaluated_by, completed_at,
		created_at, updated_at
	) VALUES (
		$1, $2, $3,
		$4::date, $5::date, 'active', $6,
		NULL, NULL, NULL,
		NOW(), NOW()
	) RETURNING id, employment_id, phase_definition_id, start_date::text, end_date::text, status, override_config, evaluation_notes, evaluated_by, completed_at, created_at, updated_at`,
		phaseID, employmentID, req.PhaseDefinitionID,
		req.StartDate, req.EndDate, oc,
	))
}

func (r EmploymentPhaseRepository) UpdatePhaseStatus(ctx context.Context, employmentID, phaseID uuid.UUID, req api.UpdateEmployeePhaseStatusRequest, evaluatedBy uuid.UUID) (api.EmployeePhase, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeePhase{}, err
	}
	row := db.QueryRow(ctx, `UPDATE employee_phases
		SET status = $1,
		    evaluation_notes = $2,
		    evaluated_by = $3,
		    completed_at = CASE WHEN $1 IN ('completed', 'terminated') THEN NOW() ELSE NULL END,
		    updated_at = NOW()
		WHERE id = $4 AND employment_id = $5
		RETURNING id, employment_id, phase_definition_id, start_date::text, end_date::text, status, override_config, evaluation_notes, evaluated_by, completed_at, created_at, updated_at`,
		req.Status, req.EvaluationNotes, evaluatedBy, phaseID, employmentID,
	)
	p, err := scanPhase(row)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmployeePhase{}, api.ErrEmployeePhaseNotFound
	}
	return p, err
}

func (r EmploymentPhaseRepository) ListOrganizationPhases(ctx context.Context, page, limit int, query, status string, employeeID *uuid.UUID) (api.EmploymentPhaseAssignmentListResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmploymentPhaseAssignmentListResponse{}, err
	}
	offset := (page - 1) * limit
	search := "%" + query + "%"
	rows, err := db.Query(ctx, `SELECT ep.id, ep.employment_id, ep.phase_definition_id, ep.start_date::text, ep.end_date::text,
		ep.status, ep.override_config, ep.evaluation_notes, ep.evaluated_by, ep.completed_at, ep.created_at, ep.updated_at,
		e.id, e.full_name, COALESCE(t.employee_number, ''), d.name, pd.name, pd.phase_type
		FROM employee_phases ep
		JOIN employments em ON em.id = ep.employment_id AND em.deleted_at IS NULL
		JOIN employees e ON e.id = em.employee_id AND e.deleted_at IS NULL
		LEFT JOIN departments d ON d.id = em.department_id
		JOIN employment_phase_definitions pd ON pd.id = ep.phase_definition_id
		WHERE ($1 = '%%' OR e.full_name ILIKE $1 OR COALESCE(t.employee_number, '') ILIKE $1 OR pd.name ILIKE $1)
		  AND ($2 = '' OR ep.status = $2)
		  AND ($5::uuid IS NULL OR e.id = $5::uuid)
		ORDER BY ep.start_date DESC, ep.id ASC LIMIT $3 OFFSET $4`, search, status, limit, offset, employeeID)
	if err != nil {
		return api.EmploymentPhaseAssignmentListResponse{}, err
	}
	defer rows.Close()
	result := api.EmploymentPhaseAssignmentListResponse{Data: make([]api.EmploymentPhaseAssignment, 0), Page: page, Limit: limit}
	for rows.Next() {
		var item api.EmploymentPhaseAssignment
		var created, updated time.Time
		var completed *time.Time
		var override []byte
		if err := rows.Scan(&item.ID, &item.EmploymentID, &item.PhaseDefinitionID, &item.StartDate, &item.EndDate, &item.Status, &override, &item.EvaluationNotes, &item.EvaluatedBy, &completed, &created, &updated, &item.EmployeeID, &item.EmployeeName, &item.EmployeeCode, &item.DepartmentName, &item.PhaseName, &item.PhaseType); err != nil {
			return api.EmploymentPhaseAssignmentListResponse{}, err
		}
		item.OverrideConfig = override
		item.CreatedAt = created.UTC().Format(time.RFC3339)
		item.UpdatedAt = updated.UTC().Format(time.RFC3339)
		if completed != nil {
			value := completed.UTC().Format(time.RFC3339)
			item.CompletedAt = &value
		}
		result.Data = append(result.Data, item)
	}
	if err := rows.Err(); err != nil {
		return api.EmploymentPhaseAssignmentListResponse{}, err
	}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM employee_phases ep JOIN employments em ON em.id = ep.employment_id AND em.deleted_at IS NULL JOIN employees e ON e.id = em.employee_id AND e.deleted_at IS NULL LEFT JOIN employment_relationships er ON er.id = em.relationship_id AND er.employee_id = em.employee_id LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= ep.start_date AND (t.end_date IS NULL OR t.end_date >= ep.start_date) JOIN employment_phase_definitions pd ON pd.id = ep.phase_definition_id WHERE ($1 = '%%' OR e.full_name ILIKE $1 OR COALESCE(t.employee_number, '') ILIKE $1 OR pd.name ILIKE $1) AND ($2 = '' OR ep.status = $2) AND ($3::uuid IS NULL OR e.id = $3::uuid)`, search, status, employeeID).Scan(&result.Total); err != nil {
		return api.EmploymentPhaseAssignmentListResponse{}, err
	}
	return result, nil
}

func (r EmploymentPhaseRepository) ListPhases(ctx context.Context, employmentID uuid.UUID) ([]api.EmployeePhase, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		id, employment_id, phase_definition_id, start_date::text, end_date::text,
		status, override_config, evaluation_notes, evaluated_by, completed_at, created_at, updated_at
	FROM employee_phases
	WHERE employment_id = $1
	ORDER BY start_date ASC, created_at DESC`, employmentID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.EmployeePhase, 0)
	for rows.Next() {
		p, err := scanPhase(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, p)
	}
	return result, rows.Err()
}

func (r EmploymentPhaseRepository) ResolveActiveEmployment(ctx context.Context, employeeID uuid.UUID) (uuid.UUID, error) {
	db, err := r.db(ctx)
	if err != nil {
		return uuid.Nil, err
	}
	var exists bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND deleted_at IS NULL)`, employeeID).Scan(&exists)
	if err != nil {
		return uuid.Nil, err
	}
	if !exists {
		return uuid.Nil, api.ErrEmployeeNotFound
	}

	var employmentID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employments
		WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL
		ORDER BY start_date DESC, id LIMIT 1`, employeeID).Scan(&employmentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, api.ErrNoActiveEmployment
	}
	return employmentID, err
}
