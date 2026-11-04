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
	BeginTx func(context.Context, uuid.UUID) (TxBeginner, error)
}

func (r EmploymentPhaseRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("employment phase repository: database is not configured")
	}
	return r.DB, nil
}

func (r EmploymentPhaseRepository) begin(ctx context.Context, wsID uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, wsID)
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
		id, workspace_id, name, description, phase_type, rule_config, is_active, display_order, created_at, updated_at
	) VALUES (
		$1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW()
	) RETURNING id, workspace_id, name, description, phase_type, rule_config, is_active, display_order, created_at, updated_at`,
		def.ID, def.WorkspaceID, def.Name, def.Description, def.PhaseType, def.RuleConfig, def.IsActive, def.DisplayOrder,
	).Scan(
		&out.ID, &out.WorkspaceID, &out.Name, &out.Description, &out.PhaseType, &rc, &out.IsActive, &out.DisplayOrder, &created, &updated,
	)
	if err != nil {
		return api.EmploymentPhaseDefinition{}, err
	}
	out.RuleConfig = json.RawMessage(rc)
	out.CreatedAt = created.UTC().Format(time.RFC3339)
	out.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return out, nil
}

func (r EmploymentPhaseRepository) ListDefinitions(ctx context.Context, wsID uuid.UUID) ([]api.EmploymentPhaseDefinition, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		id, workspace_id, name, description, phase_type, rule_config, is_active, display_order, created_at, updated_at
	FROM employment_phase_definitions
	WHERE workspace_id = $1
	ORDER BY display_order ASC, created_at DESC`, wsID)
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
			&item.ID, &item.WorkspaceID, &item.Name, &item.Description, &item.PhaseType, &rc, &item.IsActive, &item.DisplayOrder, &created, &updated,
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
		&p.ID, &p.WorkspaceID, &p.EmploymentID, &p.PhaseDefinitionID, &p.StartDate, &p.EndDate,
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

func (r EmploymentPhaseRepository) AssignPhase(ctx context.Context, wsID, employmentID uuid.UUID, req api.AssignEmployeePhaseRequest) (api.EmployeePhase, error) {
	txBeginner, err := r.begin(ctx, wsID)
	if err != nil {
		return api.EmployeePhase{}, err
	}
	if txBeginner != nil {
		var created api.EmployeePhase
		err := service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
			res, txErr := r.assignPhaseTx(ctx, tx, wsID, employmentID, req)
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
	return r.assignPhaseTx(ctx, db, wsID, employmentID, req)
}

func (r EmploymentPhaseRepository) assignPhaseTx(ctx context.Context, db DBTX, wsID, employmentID uuid.UUID, req api.AssignEmployeePhaseRequest) (api.EmployeePhase, error) {
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
		WHERE workspace_id = $1
		  AND employment_id = $2
		  AND status = 'active'
		  AND start_date <= $3::date`, autoCloseDate), wsID, employmentID, req.StartDate)
	if err != nil {
		return api.EmployeePhase{}, err
	}

	var oc any = nil
	if len(req.OverrideConfig) > 0 && string(req.OverrideConfig) != "null" {
		oc = req.OverrideConfig
	}

	phaseID := uuid.New()
	return scanPhase(db.QueryRow(ctx, `INSERT INTO employee_phases (
		id, workspace_id, employment_id, phase_definition_id,
		start_date, end_date, status, override_config,
		evaluation_notes, evaluated_by, completed_at,
		created_at, updated_at
	) VALUES (
		$1, $2, $3, $4,
		$5::date, $6::date, 'active', $7,
		NULL, NULL, NULL,
		NOW(), NOW()
	) RETURNING id, workspace_id, employment_id, phase_definition_id, start_date::text, end_date::text, status, override_config, evaluation_notes, evaluated_by, completed_at, created_at, updated_at`,
		phaseID, wsID, employmentID, req.PhaseDefinitionID,
		req.StartDate, req.EndDate, oc,
	))
}

func (r EmploymentPhaseRepository) UpdatePhaseStatus(ctx context.Context, wsID, employmentID, phaseID uuid.UUID, req api.UpdateEmployeePhaseStatusRequest, evaluatedBy uuid.UUID) (api.EmployeePhase, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeePhase{}, err
	}
	row := db.QueryRow(ctx, `UPDATE employee_phases
		SET status = $1,
		    evaluation_notes = $2,
		    evaluated_by = $3,
		    completed_at = NOW(),
		    updated_at = NOW()
		WHERE id = $4 AND workspace_id = $5 AND employment_id = $6
		RETURNING id, workspace_id, employment_id, phase_definition_id, start_date::text, end_date::text, status, override_config, evaluation_notes, evaluated_by, completed_at, created_at, updated_at`,
		req.Status, req.EvaluationNotes, evaluatedBy, phaseID, wsID, employmentID,
	)
	p, err := scanPhase(row)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.EmployeePhase{}, api.ErrEmployeePhaseNotFound
	}
	return p, err
}

func (r EmploymentPhaseRepository) ListPhases(ctx context.Context, wsID, employmentID uuid.UUID) ([]api.EmployeePhase, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		id, workspace_id, employment_id, phase_definition_id, start_date::text, end_date::text,
		status, override_config, evaluation_notes, evaluated_by, completed_at, created_at, updated_at
	FROM employee_phases
	WHERE workspace_id = $1 AND employment_id = $2
	ORDER BY start_date ASC, created_at DESC`, wsID, employmentID)
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

func (r EmploymentPhaseRepository) ResolveActiveEmployment(ctx context.Context, wsID, employeeID uuid.UUID) (uuid.UUID, error) {
	db, err := r.db(ctx)
	if err != nil {
		return uuid.Nil, err
	}
	var exists bool
	err = db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employees WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, employeeID, wsID).Scan(&exists)
	if err != nil {
		return uuid.Nil, err
	}
	if !exists {
		return uuid.Nil, api.ErrEmployeeNotFound
	}

	var employmentID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employments
		WHERE employee_id = $1 AND workspace_id = $2 AND status = 'active' AND deleted_at IS NULL
		ORDER BY start_date DESC, id LIMIT 1`, employeeID, wsID).Scan(&employmentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, api.ErrNoActiveEmployment
	}
	return employmentID, err
}
