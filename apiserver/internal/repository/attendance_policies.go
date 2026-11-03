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
	"github.com/vkaylee/leeattend/apiserver/internal/apperror"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type AttendancePolicyRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
	SystemDB DBTX
	BeginTx  func(context.Context, uuid.UUID) (TxBeginner, error)
}

func (r AttendancePolicyRepository) db(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("attendance policy repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, workspaceID)
}

func (r AttendancePolicyRepository) begin(ctx context.Context, workspaceID uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, workspaceID)
	}
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("attendance policy repository: tenant database does not support transactions")
	}
	return beginner, nil
}

func (r AttendancePolicyRepository) validateBehalfAssignee(ctx context.Context, workspaceID, userID uuid.UUID) error {
	if r.SystemDB == nil {
		return errors.New("attendance policy repository: system database is not configured")
	}
	var isMember bool
	if err := r.SystemDB.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM workspace_members WHERE workspace_id = $1 AND user_id = $2)`, workspaceID, userID).Scan(&isMember); err != nil {
		return err
	}
	if !isMember {
		return apperror.Forbidden(apperror.KeyNotWorkspaceMember)
	}
	return nil
}

func decodeAttendanceRules(raw []byte) api.AttendanceRules {
	rules := api.DefaultAttendanceRules()
	if len(raw) == 0 || json.Unmarshal(raw, &rules) != nil || !validStoredRules(rules) {
		return api.DefaultAttendanceRules()
	}
	return rules
}

func validStoredRules(rules api.AttendanceRules) bool {
	if rules.WorkUnitMinutes <= 0 || (rules.RoundingPolicy != "floor" && rules.RoundingPolicy != "round" && rules.RoundingPolicy != "ceil" && rules.RoundingPolicy != "none") {
		return false
	}
	return rules.EarlyArrivalStrategy == "allow" || rules.EarlyArrivalStrategy == "block" || rules.EarlyArrivalStrategy == "clamp"
}

func parsePolicyTime(raw *string) (time.Time, error) {
	if raw == nil || strings.TrimSpace(*raw) == "" {
		return time.Now().UTC(), nil
	}
	value := strings.TrimSpace(*raw)
	if parsed, err := time.Parse(time.RFC3339, value); err == nil {
		return parsed.UTC(), nil
	}
	parsed, err := time.Parse("2006-01-02", value)
	if err != nil {
		return time.Time{}, apperror.Validation(apperror.KeyInvalidDateFormat)
	}
	return parsed.UTC(), nil
}

func policyVersionResponse(id uuid.UUID, rules api.AttendanceRules, validFrom time.Time, validTo *time.Time, createdBy *uuid.UUID, createdAt time.Time) api.AttendancePolicyVersionResponse {
	grace := api.DefaultGracePeriodMinutes
	if rules.GracePeriodMinutes != nil {
		grace = *rules.GracePeriodMinutes
	}
	var validToValue, createdByValue *string
	if validTo != nil {
		value := validTo.UTC().Format(time.RFC3339)
		validToValue = &value
	}
	if createdBy != nil {
		value := createdBy.String()
		createdByValue = &value
	}
	return api.AttendancePolicyVersionResponse{
		ID: id.String(), WorkUnitMinutes: rules.WorkUnitMinutes, RoundingPolicy: rules.RoundingPolicy,
		EarlyArrivalStrategy: rules.EarlyArrivalStrategy, GracePeriodMinutes: grace,
		ValidFrom: validFrom.UTC().Format(time.RFC3339), ValidTo: validToValue,
		CreatedBy: createdByValue, CreatedAt: createdAt.UTC().Format(time.RFC3339),
	}
}

func (r AttendancePolicyRepository) GetDefaultPolicyRules(ctx context.Context, workspaceID uuid.UUID) (api.AttendanceRules, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.AttendanceRules{}, err
	}
	var raw []byte
	err = db.QueryRow(ctx, `SELECT v.rules FROM attendance_policy_versions v JOIN attendance_policies p ON p.id = v.policy_id WHERE p.workspace_id = $1 AND p.is_default = TRUE ORDER BY v.valid_from DESC, v.created_at DESC LIMIT 1`, workspaceID).Scan(&raw)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.DefaultAttendanceRules(), nil
	}
	if err != nil {
		return api.AttendanceRules{}, err
	}
	return decodeAttendanceRules(raw), nil
}

func (r AttendancePolicyRepository) GetDefaultPolicyVersions(ctx context.Context, workspaceID uuid.UUID) ([]api.AttendancePolicyVersionResponse, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT v.id, v.rules, v.valid_from, v.valid_to, v.created_by, v.created_at FROM attendance_policy_versions v JOIN attendance_policies p ON p.id = v.policy_id WHERE p.workspace_id = $1 AND p.is_default = TRUE ORDER BY v.valid_from DESC, v.created_at DESC`, workspaceID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.AttendancePolicyVersionResponse, 0)
	for rows.Next() {
		var id uuid.UUID
		var raw []byte
		var validFrom, createdAt time.Time
		var validTo *time.Time
		var createdBy *uuid.UUID
		if err := rows.Scan(&id, &raw, &validFrom, &validTo, &createdBy, &createdAt); err != nil {
			return nil, err
		}
		result = append(result, policyVersionResponse(id, decodeAttendanceRules(raw), validFrom, validTo, createdBy, createdAt))
	}
	return result, rows.Err()
}

func (r AttendancePolicyRepository) UpdateDefaultPolicyRules(ctx context.Context, workspaceID, userID uuid.UUID, req api.UpdateAttendanceRulesRequest) (api.AttendancePolicyVersionResponse, error) {
	validFrom, err := parsePolicyTime(req.ValidFrom)
	if err != nil {
		return api.AttendancePolicyVersionResponse{}, err
	}
	beginner, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.AttendancePolicyVersionResponse{}, err
	}
	var result api.AttendancePolicyVersionResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var policyID uuid.UUID
		err := tx.QueryRow(ctx, `SELECT id FROM attendance_policies WHERE workspace_id = $1 AND is_default = TRUE LIMIT 1`, workspaceID).Scan(&policyID)
		if errors.Is(err, pgx.ErrNoRows) {
			policyID = uuid.New()
			if _, err = tx.Exec(ctx, `INSERT INTO attendance_policies (id, workspace_id, name, is_default, created_at, updated_at) VALUES ($1, $2, 'Default Policy', TRUE, NOW(), NOW())`, policyID, workspaceID); err != nil {
				return err
			}
		} else if err != nil {
			return err
		}
		var raw []byte
		err = tx.QueryRow(ctx, `SELECT rules FROM attendance_policy_versions WHERE policy_id = $1 ORDER BY valid_from DESC, created_at DESC LIMIT 1`, policyID).Scan(&raw)
		rules := api.DefaultAttendanceRules()
		if err == nil {
			rules = decodeAttendanceRules(raw)
		} else if !errors.Is(err, pgx.ErrNoRows) {
			return err
		}
		if req.WorkUnitMinutes != nil {
			rules.WorkUnitMinutes = *req.WorkUnitMinutes
		}
		if req.RoundingPolicy != nil {
			rules.RoundingPolicy = *req.RoundingPolicy
		}
		if req.EarlyArrivalStrategy != nil {
			rules.EarlyArrivalStrategy = *req.EarlyArrivalStrategy
		}
		if req.GracePeriodMinutes != nil {
			rules.GracePeriodMinutes = req.GracePeriodMinutes
		}
		payload, err := json.Marshal(rules)
		if err != nil {
			return err
		}
		if _, err = tx.Exec(ctx, `UPDATE attendance_policy_versions SET valid_to = $2 WHERE policy_id = $1 AND valid_to IS NULL`, policyID, validFrom); err != nil {
			return err
		}
		versionID := uuid.New()
		createdAt := time.Now().UTC()
		if _, err = tx.Exec(ctx, `INSERT INTO attendance_policy_versions (id, policy_id, rules, valid_from, valid_to, created_by, created_at) VALUES ($1, $2, $3, $4, NULL, $5, $6)`, versionID, policyID, payload, validFrom, userID, createdAt); err != nil {
			return err
		}
		result = policyVersionResponse(versionID, rules, validFrom, nil, &userID, createdAt)
		return nil
	})
	return result, err
}

func (r AttendancePolicyRepository) ListPolicyAssignments(ctx context.Context, workspaceID uuid.UUID) ([]api.PolicyAssignmentResponse, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT DISTINCT ON (a.id) a.id, a.target_type, a.target_id, a.valid_from, a.valid_to, v.rules FROM attendance_policy_assignments a JOIN attendance_policies p ON p.id = a.policy_id JOIN attendance_policy_versions v ON v.policy_id = a.policy_id WHERE p.workspace_id = $1 AND a.valid_from <= NOW() AND (a.valid_to IS NULL OR a.valid_to >= NOW()) AND v.valid_from <= NOW() AND (v.valid_to IS NULL OR v.valid_to >= NOW()) ORDER BY a.id, v.valid_from DESC, v.created_at DESC`, workspaceID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PolicyAssignmentResponse, 0)
	for rows.Next() {
		var id, targetID uuid.UUID
		var targetType string
		var validFrom time.Time
		var validTo *time.Time
		var raw []byte
		if err := rows.Scan(&id, &targetType, &targetID, &validFrom, &validTo, &raw); err != nil {
			return nil, err
		}
		var validToValue *string
		if validTo != nil {
			value := validTo.UTC().Format(time.RFC3339)
			validToValue = &value
		}
		result = append(result, api.PolicyAssignmentResponse{ID: id.String(), TargetType: targetType, TargetID: targetID.String(), ValidFrom: validFrom.UTC().Format(time.RFC3339), ValidTo: validToValue, Rules: decodeAttendanceRules(raw)})
	}
	return result, rows.Err()
}

func (r AttendancePolicyRepository) UpsertPolicyAssignment(ctx context.Context, workspaceID, userID uuid.UUID, req api.UpsertPolicyAssignmentRequest) (api.PolicyAssignmentResponse, error) {
	validFrom, err := parsePolicyTime(req.ValidFrom)
	if err != nil {
		return api.PolicyAssignmentResponse{}, err
	}
	beginner, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.PolicyAssignmentResponse{}, err
	}
	var result api.PolicyAssignmentResponse
	err = service.WithTx(ctx, beginner, func(ctx context.Context, tx pgx.Tx) error {
		var assignmentID, policyID uuid.UUID
		err := tx.QueryRow(ctx, `SELECT a.id, a.policy_id FROM attendance_policy_assignments a JOIN attendance_policies p ON p.id = a.policy_id WHERE p.workspace_id = $1 AND a.target_type = $2 AND a.target_id = $3 AND a.valid_to IS NULL LIMIT 1 FOR UPDATE`, workspaceID, req.TargetType, req.TargetID).Scan(&assignmentID, &policyID)
		if errors.Is(err, pgx.ErrNoRows) {
			policyID, assignmentID = uuid.New(), uuid.New()
			if _, err = tx.Exec(ctx, `INSERT INTO attendance_policies (id, workspace_id, name, is_default, created_at, updated_at) VALUES ($1, $2, $3, FALSE, NOW(), NOW())`, policyID, workspaceID, fmt.Sprintf("%s %s Override", req.TargetType, req.TargetID)); err != nil {
				return err
			}
			if _, err = tx.Exec(ctx, `INSERT INTO attendance_policy_assignments (id, policy_id, target_type, target_id, valid_from, valid_to, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, NULL, NOW(), NOW())`, assignmentID, policyID, req.TargetType, req.TargetID, validFrom); err != nil {
				return err
			}
		} else if err != nil {
			return err
		} else if _, err = tx.Exec(ctx, `UPDATE attendance_policy_versions SET valid_to = $2 WHERE policy_id = $1 AND valid_to IS NULL`, policyID, validFrom); err != nil {
			return err
		}
		rules := api.AttendanceRules{WorkUnitMinutes: req.WorkUnitMinutes, RoundingPolicy: req.RoundingPolicy, EarlyArrivalStrategy: req.EarlyArrivalStrategy, GracePeriodMinutes: req.GracePeriodMinutes}
		payload, err := json.Marshal(rules)
		if err != nil {
			return err
		}
		if _, err = tx.Exec(ctx, `INSERT INTO attendance_policy_versions (id, policy_id, rules, valid_from, valid_to, created_by, created_at) VALUES ($1, $2, $3, $4, NULL, $5, NOW())`, uuid.New(), policyID, payload, validFrom, userID); err != nil {
			return err
		}
		result = api.PolicyAssignmentResponse{ID: assignmentID.String(), TargetType: req.TargetType, TargetID: req.TargetID.String(), ValidFrom: validFrom.UTC().Format(time.RFC3339), Rules: rules}
		return nil
	})
	return result, err
}

func (r AttendancePolicyRepository) DeletePolicyAssignment(ctx context.Context, workspaceID, assignmentID uuid.UUID) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE attendance_policy_assignments a SET valid_to = NOW(), updated_at = NOW() FROM attendance_policies p WHERE a.id = $1 AND a.policy_id = p.id AND p.workspace_id = $2`, assignmentID, workspaceID)
	return err
}

func scanBehalf(row interface{ Scan(...any) error }) (api.BehalfAssignment, error) {
	var item api.BehalfAssignment
	var createdAt, updatedAt time.Time
	if err := row.Scan(&item.ID, &item.WorkspaceID, &item.AssigneeUserID, &item.TargetEmployeeID, &item.TargetDepartmentID, &item.AssignmentType, &createdAt, &updatedAt, &item.EmployerID, &item.TargetEmployeeName, &item.TargetEmployeeIDNumber, &item.TargetDepartmentName); err != nil {
		return item, err
	}
	item.CreatedAt, item.UpdatedAt = createdAt.UTC().Format(time.RFC3339), updatedAt.UTC().Format(time.RFC3339)
	return item, nil
}

const behalfColumns = `ba.id, ba.workspace_id, ba.assignee_user_id, ba.target_employee_id, ba.target_department_id, ba.assignment_type, ba.created_at, ba.updated_at, ba.employer_id, te.full_name, te.employee_id_number, d.name`

func (r AttendancePolicyRepository) CreateBehalfAssignment(ctx context.Context, workspaceID uuid.UUID, req api.CreateBehalfAssignmentRequest, assignmentType string) (api.BehalfAssignment, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.BehalfAssignment{}, err
	}
	if (req.TargetEmployeeID == nil) == (req.TargetDepartmentID == nil) {
		return api.BehalfAssignment{}, apperror.Validation(apperror.KeyValidation)
	}
	if err := r.validateBehalfAssignee(ctx, workspaceID, req.AssigneeUserID); err != nil {
		return api.BehalfAssignment{}, err
	}
	return scanBehalf(db.QueryRow(ctx, `WITH inserted AS (INSERT INTO behalf_assignments (id, workspace_id, assignee_user_id, target_employee_id, target_department_id, assignment_type, employer_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW(), NOW()) RETURNING *) SELECT `+behalfColumns+` FROM inserted ba LEFT JOIN employees te ON te.workspace_id = ba.workspace_id AND te.id = ba.target_employee_id AND te.deleted_at IS NULL LEFT JOIN departments d ON d.workspace_id = ba.workspace_id AND d.id = ba.target_department_id`, uuid.New(), workspaceID, req.AssigneeUserID, req.TargetEmployeeID, req.TargetDepartmentID, assignmentType, req.EmployerID))
}

func (r AttendancePolicyRepository) ListBehalfAssignments(ctx context.Context, workspaceID uuid.UUID, assignmentType string) ([]api.BehalfAssignment, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT `+behalfColumns+` FROM behalf_assignments ba LEFT JOIN employees te ON te.workspace_id = ba.workspace_id AND te.id = ba.target_employee_id AND te.deleted_at IS NULL LEFT JOIN departments d ON d.workspace_id = ba.workspace_id AND d.id = ba.target_department_id WHERE ba.workspace_id = $1 AND ba.assignment_type = $2 ORDER BY ba.created_at DESC`, workspaceID, assignmentType)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.BehalfAssignment, 0)
	for rows.Next() {
		item, err := scanBehalf(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (r AttendancePolicyRepository) DeleteBehalfAssignment(ctx context.Context, workspaceID, assignmentID uuid.UUID) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	result, err := db.Exec(ctx, `DELETE FROM behalf_assignments WHERE id = $1 AND workspace_id = $2`, assignmentID, workspaceID)
	if err != nil {
		return err
	}
	if result.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}

func (r AttendancePolicyRepository) ListMyBehalfTargets(ctx context.Context, workspaceID, userID uuid.UUID, assignmentType string) ([]api.EmployeeRecord, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT DISTINCT e.id, e.workspace_id, e.user_id, e.full_name, e.employee_id_number, e.tags, e.created_at, e.updated_at, d.name, em.id, em.employer_id, em.primary_location_id FROM employees e JOIN employments em ON em.employee_id = e.id AND em.workspace_id = e.workspace_id AND em.status = 'active' AND em.deleted_at IS NULL LEFT JOIN departments d ON d.id = em.department_id LEFT JOIN behalf_assignments ba ON ba.workspace_id = e.workspace_id AND ba.assignee_user_id = $2 AND ba.assignment_type = $3 AND (ba.target_employee_id = e.id OR ba.target_department_id = em.department_id) WHERE e.workspace_id = $1 AND e.deleted_at IS NULL AND ba.id IS NOT NULL ORDER BY e.full_name, e.id`, workspaceID, userID, assignmentType)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.EmployeeRecord, 0)
	for rows.Next() {
		var item api.EmployeeRecord
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&item.ID, &item.WorkspaceID, &item.UserID, &item.FullName, &item.EmployeeIDNumber, &item.Tags, &createdAt, &updatedAt, &item.DepartmentName, &item.ActiveEmploymentID, &item.EmployerID, &item.PrimaryLocationID); err != nil {
			return nil, err
		}
		item.CreatedAt, item.UpdatedAt = createdAt.UTC().Format(time.RFC3339), updatedAt.UTC().Format(time.RFC3339)
		result = append(result, item)
	}
	return result, rows.Err()
}
