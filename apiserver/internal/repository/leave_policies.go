package repository

import (
	"context"
	"encoding/json"
	"errors"
	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
	"strings"
	"time"
)

func (r LeaveRepository) ListLeavePolicies(ctx context.Context) ([]api.LeavePolicyDefinition, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `
		SELECT id, name, description, leave_type_id, rule_config, created_at, updated_at
		FROM leave_policy_definitions
		WHERE deleted_at IS NULL
		ORDER BY created_at DESC
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	policies := make([]api.LeavePolicyDefinition, 0)
	for rows.Next() {
		var p api.LeavePolicyDefinition
		var rawConfig []byte
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&p.ID, &p.Name, &p.Description, &p.LeaveTypeID, &rawConfig, &createdAt, &updatedAt); err != nil {
			return nil, err
		}
		_ = json.Unmarshal(rawConfig, &p.RuleConfig)
		p.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		p.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		policies = append(policies, p)
	}
	return policies, rows.Err()
}

func (r LeaveRepository) CreateLeavePolicy(ctx context.Context, req api.CreateLeavePolicyRequest) (api.LeavePolicyDefinition, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return api.LeavePolicyDefinition{}, err
	}
	rawConfig, err := json.Marshal(req.RuleConfig)
	if err != nil {
		rawConfig = []byte("{}")
	}
	var def api.LeavePolicyDefinition
	var createdAt, updatedAt time.Time
	var outConfig []byte
	err = db.QueryRow(ctx, `
		INSERT INTO leave_policy_definitions (id, name, description, leave_type_id, rule_config, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, NOW(), NOW())
		RETURNING id, name, description, leave_type_id, rule_config, created_at, updated_at
	`, uuid.New(), req.Name, req.Description, req.LeaveTypeID, rawConfig).Scan(
		&def.ID, &def.Name, &def.Description, &def.LeaveTypeID, &outConfig, &createdAt, &updatedAt,
	)
	if err != nil {
		if isConflictError(err) {
			return api.LeavePolicyDefinition{}, api.ErrLeaveConflict
		}
		return api.LeavePolicyDefinition{}, err
	}
	_ = json.Unmarshal(outConfig, &def.RuleConfig)
	def.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	def.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return def, nil
}

func mergeLeavePolicyPatch(existingName string, existingDesc *string, existingLeaveType uuid.UUID, existingRaw []byte, req api.UpdateLeavePolicyRequest) (string, *string, uuid.UUID, []byte) {
	name := existingName
	if req.Name != nil && strings.TrimSpace(*req.Name) != "" {
		name = strings.TrimSpace(*req.Name)
	}
	desc := existingDesc
	if req.Description != nil {
		desc = req.Description
	}
	leaveTypeID := existingLeaveType
	if req.LeaveTypeID != nil && *req.LeaveTypeID != uuid.Nil {
		leaveTypeID = *req.LeaveTypeID
	}
	configBytes := existingRaw
	if req.RuleConfig != nil {
		if b, err := json.Marshal(req.RuleConfig); err == nil {
			configBytes = b
		}
	}
	return name, desc, leaveTypeID, configBytes
}

func (r LeaveRepository) UpdateLeavePolicy(ctx context.Context, policyID uuid.UUID, req api.UpdateLeavePolicyRequest) (api.LeavePolicyDefinition, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return api.LeavePolicyDefinition{}, err
	}
	var existingRaw []byte
	var existingName string
	var existingDesc *string
	var existingLeaveType uuid.UUID
	err = db.QueryRow(ctx, `
		SELECT name, description, leave_type_id, rule_config
		FROM leave_policy_definitions
		WHERE id = $1 AND deleted_at IS NULL
	`, policyID).Scan(&existingName, &existingDesc, &existingLeaveType, &existingRaw)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LeavePolicyDefinition{}, api.ErrLeaveNotFound
	}
	if err != nil {
		return api.LeavePolicyDefinition{}, err
	}
	name, desc, leaveTypeID, configBytes := mergeLeavePolicyPatch(existingName, existingDesc, existingLeaveType, existingRaw, req)
	var def api.LeavePolicyDefinition
	var outConfig []byte
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, `
		UPDATE leave_policy_definitions
		SET name = $1, description = $2, leave_type_id = $3, rule_config = $4, updated_at = NOW()
		WHERE id = $5 AND deleted_at IS NULL
		RETURNING id, name, description, leave_type_id, rule_config, created_at, updated_at
	`, name, desc, leaveTypeID, configBytes, policyID).Scan(
		&def.ID, &def.Name, &def.Description, &def.LeaveTypeID, &outConfig, &createdAt, &updatedAt,
	)
	if err != nil {
		if isConflictError(err) {
			return api.LeavePolicyDefinition{}, api.ErrLeaveConflict
		}
		return api.LeavePolicyDefinition{}, err
	}
	_ = json.Unmarshal(outConfig, &def.RuleConfig)
	def.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	def.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return def, nil
}

func (r LeaveRepository) DeleteLeavePolicy(ctx context.Context, policyID uuid.UUID) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if _, err := db.Exec(ctx, `UPDATE leave_policy_assignments SET deleted_at = NOW() WHERE policy_id = $1 AND deleted_at IS NULL`, policyID); err != nil {
			return err
		}
		tag, err := db.Exec(ctx, `UPDATE leave_policy_definitions SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL`, policyID)
		if err != nil {
			return err
		}
		if tag.RowsAffected() == 0 {
			return api.ErrLeaveNotFound
		}
		return nil
	})
}

func (r LeaveRepository) AssignLeavePolicy(ctx context.Context, req api.AssignLeavePolicyRequest) (api.LeavePolicyAssignment, error) {
	tx, err := r.begin(ctx)
	if err != nil {
		return api.LeavePolicyAssignment{}, err
	}
	var assignment api.LeavePolicyAssignment
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var leaveTypeID uuid.UUID
		err := db.QueryRow(ctx, `SELECT leave_type_id FROM leave_policy_definitions WHERE id = $1 AND deleted_at IS NULL`, req.PolicyID).Scan(&leaveTypeID)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrLeaveNotFound
		}
		if err != nil {
			return err
		}
		if _, err = db.Exec(ctx, `
			UPDATE leave_policy_assignments SET deleted_at = NOW()
			WHERE assignee_type = $1 AND assignee_id IS NOT DISTINCT FROM $2
			  AND policy_id IN (SELECT id FROM leave_policy_definitions WHERE leave_type_id = $3)
			  AND effective_from >= $4::date AND deleted_at IS NULL
		`, req.AssigneeType, req.AssigneeID, leaveTypeID, req.EffectiveFrom); err != nil {
			return err
		}
		if _, err = db.Exec(ctx, `
			UPDATE leave_policy_assignments SET effective_to = ($4::date - interval '1 day')::date
			WHERE assignee_type = $1 AND assignee_id IS NOT DISTINCT FROM $2
			  AND policy_id IN (SELECT id FROM leave_policy_definitions WHERE leave_type_id = $3)
			  AND effective_from < $4::date AND (effective_to IS NULL OR effective_to >= $4::date) AND deleted_at IS NULL
		`, req.AssigneeType, req.AssigneeID, leaveTypeID, req.EffectiveFrom); err != nil {
			return err
		}
		var createdAt time.Time
		err = db.QueryRow(ctx, `
			INSERT INTO leave_policy_assignments (id, assignee_type, assignee_id, policy_id, effective_from, effective_to, created_at)
			VALUES ($1, $2, $3, $4, $5::date, $6::date, NOW())
			RETURNING id, assignee_type, assignee_id, policy_id, effective_from::text, effective_to::text, created_at
		`, uuid.New(), req.AssigneeType, req.AssigneeID, req.PolicyID, req.EffectiveFrom, req.EffectiveTo).Scan(
			&assignment.ID, &assignment.AssigneeType, &assignment.AssigneeID, &assignment.PolicyID, &assignment.EffectiveFrom, &assignment.EffectiveTo, &createdAt,
		)
		if err != nil {
			if isConflictError(err) {
				return api.ErrLeaveConflict
			}
			return err
		}
		assignment.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		return nil
	})
	if err != nil {
		return api.LeavePolicyAssignment{}, err
	}
	return assignment, nil
}

func (r LeaveRepository) ListLeavePolicyAssignments(ctx context.Context) ([]api.LeavePolicyAssignmentDetail, error) {
	db, err := requireDB(r.DB, "leave repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `
		SELECT a.id, a.assignee_type, a.assignee_id, a.policy_id, a.effective_from::text, a.effective_to::text, a.created_at, d.name, lt.name
		FROM leave_policy_assignments a
		JOIN leave_policy_definitions d ON a.policy_id = d.id
		JOIN leave_types lt ON d.leave_type_id = lt.id
		WHERE a.deleted_at IS NULL AND d.deleted_at IS NULL AND lt.deleted_at IS NULL
		ORDER BY a.created_at DESC
	`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	details := make([]api.LeavePolicyAssignmentDetail, 0)
	for rows.Next() {
		var d api.LeavePolicyAssignmentDetail
		var createdAt time.Time
		if err := rows.Scan(&d.ID, &d.AssigneeType, &d.AssigneeID, &d.PolicyID, &d.EffectiveFrom, &d.EffectiveTo, &createdAt, &d.PolicyName, &d.LeaveTypeName); err != nil {
			return nil, err
		}
		d.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		details = append(details, d)
	}
	return details, rows.Err()
}

func (r LeaveRepository) DeleteLeavePolicyAssignment(ctx context.Context, assignmentID uuid.UUID) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var assigneeType string
		var assigneeID *uuid.UUID
		var policyID uuid.UUID
		var effectiveFrom, effectiveTo *string
		err := db.QueryRow(ctx, `SELECT assignee_type, assignee_id, policy_id, effective_from::text, effective_to::text FROM leave_policy_assignments WHERE id = $1 AND deleted_at IS NULL`, assignmentID).Scan(&assigneeType, &assigneeID, &policyID, &effectiveFrom, &effectiveTo)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrLeaveNotFound
		}
		if err != nil {
			return err
		}
		var leaveTypeID uuid.UUID
		if err := db.QueryRow(ctx, `SELECT leave_type_id FROM leave_policy_definitions WHERE id = $1`, policyID).Scan(&leaveTypeID); err != nil {
			return err
		}
		if _, err := db.Exec(ctx, `UPDATE leave_policy_assignments SET deleted_at = NOW() WHERE id = $1`, assignmentID); err != nil {
			return err
		}
		if effectiveFrom != nil {
			tFrom, parseErr := time.Parse("2006-01-02", *effectiveFrom)
			if parseErr == nil {
				prevEnd := tFrom.AddDate(0, 0, -1).Format("2006-01-02")
				var prevAssignID uuid.UUID
				err = db.QueryRow(ctx, `
					SELECT a.id FROM leave_policy_assignments a
					JOIN leave_policy_definitions d ON a.policy_id = d.id
					WHERE a.assignee_type = $1 AND a.assignee_id IS NOT DISTINCT FROM $2
					  AND d.leave_type_id = $3 AND a.effective_to = $4::date AND a.deleted_at IS NULL
					ORDER BY a.created_at DESC LIMIT 1
				`, assigneeType, assigneeID, leaveTypeID, prevEnd).Scan(&prevAssignID)
				if err == nil {
					_, _ = db.Exec(ctx, `UPDATE leave_policy_assignments SET effective_to = $1::date WHERE id = $2`, effectiveTo, prevAssignID)
				}
			}
		}
		return nil
	})
}
