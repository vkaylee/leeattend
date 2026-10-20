package repository

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// OvertimeRepository persists overtime policies, requests, and attendance OT claims.
type OvertimeRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
	BeginTx  func(context.Context, uuid.UUID) (TxBeginner, error)
}

func (r OvertimeRepository) db(ctx context.Context, ws uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("overtime repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, ws)
}

func (r OvertimeRepository) begin(ctx context.Context, ws uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, ws)
	}
	db, err := r.db(ctx, ws)
	if err != nil {
		return nil, err
	}
	tx, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("overtime repository: tenant database does not support transactions")
	}
	return tx, nil
}

func parseClock(raw *string) (*time.Time, error) {
	if raw == nil || *raw == "" {
		return nil, nil
	}
	for _, layout := range []string{"15:04:05", "15:04"} {
		if value, err := time.Parse(layout, *raw); err == nil {
			return &value, nil
		}
	}
	return nil, api.ErrOvertimeValidation
}

func formatClock(value *time.Time) *string {
	if value == nil {
		return nil
	}
	v := value.Format("15:04:05")
	return &v
}

func scanPolicy(row interface{ Scan(...any) error }) (api.OvertimePolicy, error) {
	var p api.OvertimePolicy
	var created, updated time.Time
	err := row.Scan(&p.ID, &p.WorkspaceID, &p.Name, &p.Description, &created, &updated)
	p.CreatedAt = created.UTC().Format(time.RFC3339)
	p.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return p, err
}

func (r OvertimeRepository) ListOvertimePolicies(ctx context.Context, ws uuid.UUID, page, limit int) (api.OvertimePolicyListResponse, error) {
	db, err := r.db(ctx, ws)
	if err != nil {
		return api.OvertimePolicyListResponse{}, err
	}
	out := api.OvertimePolicyListResponse{Data: []api.OvertimePolicy{}, Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM overtime_policy_definitions WHERE workspace_id = $1`, ws).Scan(&out.Total); err != nil {
		return out, err
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, name, description, created_at, updated_at FROM overtime_policy_definitions WHERE workspace_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3`, ws, limit, (page-1)*limit)
	if err != nil {
		return out, err
	}
	defer rows.Close()
	for rows.Next() {
		item, err := scanPolicy(rows)
		if err != nil {
			return out, err
		}
		out.Data = append(out.Data, item)
	}
	return out, rows.Err()
}

func (r OvertimeRepository) CreateOvertimePolicy(ctx context.Context, ws uuid.UUID, req api.CreateOvertimePolicyRequest) (api.OvertimePolicy, error) {
	db, err := r.db(ctx, ws)
	if err != nil {
		return api.OvertimePolicy{}, err
	}
	return scanPolicy(db.QueryRow(ctx, `INSERT INTO overtime_policy_definitions (id, workspace_id, name, description, created_at, updated_at) VALUES ($1, $2, $3, $4, NOW(), NOW()) RETURNING id, workspace_id, name, description, created_at, updated_at`, uuid.New(), ws, req.Name, req.Description))
}

func scanRule(row interface{ Scan(...any) error }) (api.OvertimeRule, error) {
	var item api.OvertimeRule
	var start, end *time.Time
	var created, updated time.Time
	err := row.Scan(&item.ID, &item.PolicyID, &item.Priority, &item.Multiplier, &start, &end, &item.DaysOfWeek, &item.IsHoliday, &created, &updated)
	item.TimeStart, item.TimeEnd = formatClock(start), formatClock(end)
	item.CreatedAt = created.UTC().Format(time.RFC3339)
	item.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return item, err
}

func (r OvertimeRepository) GetOvertimePolicy(ctx context.Context, ws, id uuid.UUID) (api.OvertimePolicyDetail, error) {
	db, err := r.db(ctx, ws)
	if err != nil {
		return api.OvertimePolicyDetail{}, err
	}
	policy, err := scanPolicy(db.QueryRow(ctx, `SELECT id, workspace_id, name, description, created_at, updated_at FROM overtime_policy_definitions WHERE id = $1 AND workspace_id = $2`, id, ws))
	if err != nil {
		return api.OvertimePolicyDetail{}, err
	}
	rows, err := db.Query(ctx, `SELECT id, policy_id, priority, multiplier, time_start, time_end, days_of_week, is_holiday, created_at, updated_at FROM overtime_rules WHERE policy_id = $1 ORDER BY priority DESC, id`, id)
	if err != nil {
		return api.OvertimePolicyDetail{}, err
	}
	defer rows.Close()
	out := api.OvertimePolicyDetail{Policy: policy, Rules: []api.OvertimeRule{}}
	for rows.Next() {
		rule, err := scanRule(rows)
		if err != nil {
			return out, err
		}
		out.Rules = append(out.Rules, rule)
	}
	return out, rows.Err()
}

func (r OvertimeRepository) AddOvertimeRule(ctx context.Context, ws, policyID uuid.UUID, req api.CreateOvertimeRuleRequest) (api.OvertimeRule, error) {
	db, err := r.db(ctx, ws)
	if err != nil {
		return api.OvertimeRule{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM overtime_policy_definitions WHERE id = $1 AND workspace_id = $2)`, policyID, ws).Scan(&exists); err != nil {
		return api.OvertimeRule{}, err
	}
	if !exists {
		return api.OvertimeRule{}, pgx.ErrNoRows
	}
	start, err := parseClock(req.TimeStart)
	if err != nil {
		return api.OvertimeRule{}, err
	}
	end, err := parseClock(req.TimeEnd)
	if err != nil {
		return api.OvertimeRule{}, err
	}
	return scanRule(db.QueryRow(ctx, `INSERT INTO overtime_rules (id, policy_id, priority, multiplier, time_start, time_end, days_of_week, is_holiday, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW()) RETURNING id, policy_id, priority, multiplier, time_start, time_end, days_of_week, is_holiday, created_at, updated_at`, uuid.New(), policyID, req.Priority, req.Multiplier, start, end, req.DaysOfWeek, req.IsHoliday))
}

func (r OvertimeRepository) AssignOvertimePolicy(ctx context.Context, ws uuid.UUID, req api.AssignOvertimePolicyRequest) error {
	db, err := r.db(ctx, ws)
	if err != nil {
		return err
	}
	date, err := time.Parse("2006-01-02", req.EffectiveFrom)
	if err != nil {
		return err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM overtime_policy_definitions WHERE id = $1 AND workspace_id = $2)`, req.PolicyID, ws).Scan(&exists); err != nil {
		return err
	}
	if !exists {
		return pgx.ErrNoRows
	}
	_, err = db.Exec(ctx, `INSERT INTO overtime_policy_assignments (id, workspace_id, assignee_type, assignee_id, policy_id, effective_from, created_at) VALUES ($1, $2, $3, $4, $5, $6, NOW())`, uuid.New(), ws, req.AssigneeType, req.AssigneeID, req.PolicyID, date)
	return err
}

const overtimeRequestColumns = `id, workspace_id, requested_by_user_id, department_id, planned_date, planned_start_time, planned_end_time, planned_minutes, reason, status::text, workflow_id, current_step, quota_warning, reconciled_at, reconciled_by, attachment_file_ids, created_at, updated_at`

func scanOvertimeRequest(row interface{ Scan(...any) error }) (api.OvertimeRequest, error) {
	var item api.OvertimeRequest
	var planned, created, updated time.Time
	var start, end, reconciled *time.Time
	err := row.Scan(&item.ID, &item.WorkspaceID, &item.RequestedByUserID, &item.DepartmentID, &planned, &start, &end, &item.PlannedMinutes, &item.Reason, &item.Status, &item.WorkflowID, &item.CurrentStep, &item.QuotaWarning, &reconciled, &item.ReconciledBy, &item.AttachmentFileIDs, &created, &updated)
	item.PlannedDate = planned.Format("2006-01-02")
	item.PlannedStartTime, item.PlannedEndTime = formatClock(start), formatClock(end)
	if reconciled != nil {
		value := reconciled.UTC().Format(time.RFC3339)
		item.ReconciledAt = &value
	}
	item.CreatedAt = created.UTC().Format(time.RFC3339)
	item.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return item, err
}

func (r OvertimeRepository) ListOvertimeRequests(ctx context.Context, ws uuid.UUID, page, limit int) (api.OvertimeRequestListResponse, error) {
	db, err := r.db(ctx, ws)
	if err != nil {
		return api.OvertimeRequestListResponse{}, err
	}
	out := api.OvertimeRequestListResponse{Data: []api.OvertimeRequest{}, Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM overtime_requests WHERE workspace_id = $1`, ws).Scan(&out.Total); err != nil {
		return out, err
	}
	rows, err := db.Query(ctx, `SELECT `+overtimeRequestColumns+` FROM overtime_requests WHERE workspace_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3`, ws, limit, (page-1)*limit)
	if err != nil {
		return out, err
	}
	defer rows.Close()
	for rows.Next() {
		item, err := scanOvertimeRequest(rows)
		if err != nil {
			return out, err
		}
		out.Data = append(out.Data, item)
	}
	return out, rows.Err()
}

func (r OvertimeRepository) CreateOvertimeRequest(ctx context.Context, ws, user uuid.UUID, req api.CreateOvertimeRequest) (api.OvertimeRequest, error) {
	tx, err := r.begin(ctx, ws)
	if err != nil {
		return api.OvertimeRequest{}, err
	}
	var out api.OvertimeRequest
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		planned, err := time.Parse("2006-01-02", req.PlannedDate)
		if err != nil {
			return err
		}
		start, err := parseClock(req.PlannedStartTime)
		if err != nil {
			return err
		}
		end, err := parseClock(req.PlannedEndTime)
		if err != nil {
			return err
		}
		id := uuid.New()
		out, err = scanOvertimeRequest(db.QueryRow(ctx, `INSERT INTO overtime_requests (id, workspace_id, requested_by_user_id, department_id, planned_date, planned_start_time, planned_end_time, planned_minutes, reason, status, current_step, attachment_file_ids, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'draft', 0, $10, NOW(), NOW()) RETURNING `+overtimeRequestColumns, id, ws, user, req.DepartmentID, planned, start, end, req.PlannedMinutes, req.Reason, req.AttachmentFileIDs))
		if err != nil {
			return err
		}
		for _, employeeID := range req.Employees {
			var employmentID uuid.UUID
			if err := db.QueryRow(ctx, `SELECT id FROM employments WHERE workspace_id = $1 AND employee_id = $2 AND status = 'active' AND deleted_at IS NULL AND start_date <= $3 AND (end_date IS NULL OR end_date >= $3) ORDER BY start_date DESC LIMIT 1`, ws, employeeID, planned).Scan(&employmentID); err != nil {
				return err
			}
			if _, err := db.Exec(ctx, `INSERT INTO overtime_request_employees (id, overtime_request_id, employment_id, consent_status, created_at) VALUES ($1, $2, $3, 'pending', NOW())`, uuid.New(), id, employmentID); err != nil {
				return err
			}
		}
		return nil
	})
	return out, err
}

func (r OvertimeRepository) GetOvertimeRequest(ctx context.Context, ws, id uuid.UUID) (api.OvertimeRequestDetail, error) {
	db, err := r.db(ctx, ws)
	if err != nil {
		return api.OvertimeRequestDetail{}, err
	}
	request, err := scanOvertimeRequest(db.QueryRow(ctx, `SELECT `+overtimeRequestColumns+` FROM overtime_requests WHERE id = $1 AND workspace_id = $2`, id, ws))
	if err != nil {
		return api.OvertimeRequestDetail{}, err
	}
	rows, err := db.Query(ctx, `SELECT id, overtime_request_id, employment_id, consent_status, consent_at, decline_reason, actual_ot_minutes, matched_summary_id, created_at FROM overtime_request_employees WHERE overtime_request_id = $1 ORDER BY created_at, id`, id)
	if err != nil {
		return api.OvertimeRequestDetail{}, err
	}
	defer rows.Close()
	out := api.OvertimeRequestDetail{Request: request, Employees: []api.OvertimeRequestEmployee{}}
	for rows.Next() {
		var item api.OvertimeRequestEmployee
		var consentAt, createdAt *time.Time
		if err := rows.Scan(&item.ID, &item.OvertimeRequestID, &item.EmploymentID, &item.ConsentStatus, &consentAt, &item.DeclineReason, &item.ActualOTMinutes, &item.MatchedSummaryID, &createdAt); err != nil {
			return out, err
		}
		if consentAt != nil {
			value := consentAt.UTC().Format(time.RFC3339)
			item.ConsentAt = &value
		}
		if createdAt != nil {
			item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		}
		out.Employees = append(out.Employees, item)
	}
	return out, rows.Err()
}

type workflowStepSnapshot struct {
	StepOrder      int             `json:"step_order"`
	StepName       string          `json:"step_name"`
	ApproverType   string          `json:"approver_type"`
	ApproverConfig json.RawMessage `json:"approver_config"`
	ActionRequired string          `json:"action_required"`
	CanSkip        bool            `json:"can_skip"`
	Status         string          `json:"status"`
}

func workflowSnapshot(ctx context.Context, db pgx.Tx, ws uuid.UUID, entityType string) (json.RawMessage, *uuid.UUID, error) {
	var workflowID uuid.UUID
	err := db.QueryRow(ctx, `SELECT id FROM approval_workflows WHERE workspace_id = $1 AND entity_type = $2 AND is_active = TRUE ORDER BY created_at DESC LIMIT 1`, ws, entityType).Scan(&workflowID)
	if errors.Is(err, pgx.ErrNoRows) {
		// ponytail: static fallback snapshot when workflow definition absent; upgrade path: load active workflow rules dynamically with custom SLA evaluator
		fallback := []workflowStepSnapshot{{
			StepOrder: 1, StepName: "Direct Manager Approval (Fallback)", ApproverType: "department_manager",
			ApproverConfig: json.RawMessage(`{}`), ActionRequired: "approve", Status: "pending",
		}}
		payload, err := json.Marshal(fallback)
		return payload, nil, err
	}
	if err != nil {
		return nil, nil, err
	}
	rows, err := db.Query(ctx, `SELECT step_order, step_name, approver_type, approver_config, action_required, can_skip FROM approval_workflow_steps WHERE workflow_id = $1 ORDER BY step_order ASC`, workflowID)
	if err != nil {
		return nil, nil, err
	}
	defer rows.Close()
	steps := make([]workflowStepSnapshot, 0)
	for rows.Next() {
		step := workflowStepSnapshot{Status: "queued"}
		if err := rows.Scan(&step.StepOrder, &step.StepName, &step.ApproverType, &step.ApproverConfig, &step.ActionRequired, &step.CanSkip); err != nil {
			return nil, nil, err
		}
		if step.StepOrder == 1 {
			step.Status = "pending"
		}
		steps = append(steps, step)
	}
	if err := rows.Err(); err != nil {
		return nil, nil, err
	}
	payload, err := json.Marshal(steps)
	return payload, &workflowID, err
}

func submitWorkflowInstance(ctx context.Context, db pgx.Tx, ws uuid.UUID, entityType string, entityID uuid.UUID) error {
	snapshot, workflowID, err := workflowSnapshot(ctx, db, ws, entityType)
	if err != nil {
		return err
	}
	if _, err = db.Exec(ctx, `UPDATE approval_workflow_instances SET status = 'cancelled', updated_at = NOW() WHERE workspace_id = $1 AND entity_type = $2 AND entity_id = $3 AND status = 'pending'`, ws, entityType, entityID); err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO approval_workflow_instances (id, workspace_id, workflow_id, entity_type, entity_id, current_step_order, status, steps_snapshot, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 1, 'pending', $6, NOW(), NOW())`, uuid.New(), ws, workflowID, entityType, entityID, snapshot)
	return err
}

func (r OvertimeRepository) SubmitOvertimeRequest(ctx context.Context, ws, id uuid.UUID) error {
	tx, err := r.begin(ctx, ws)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		tag, err := db.Exec(ctx, `UPDATE overtime_requests SET status = 'pending_approval', current_step = 1, updated_at = NOW() WHERE id = $1 AND workspace_id = $2`, id, ws)
		if err != nil {
			return err
		}
		if tag.RowsAffected() == 0 {
			return pgx.ErrNoRows
		}
		var employeeID uuid.UUID
		if err := db.QueryRow(ctx, `SELECT e.employee_id FROM overtime_request_employees ore JOIN employments e ON e.id = ore.employment_id WHERE ore.overtime_request_id = $1 ORDER BY ore.created_at, ore.id LIMIT 1`, id).Scan(&employeeID); err != nil {
			return err
		}
		return submitWorkflowInstance(ctx, db, ws, "ot_request", id)
	})
}

func (r OvertimeRepository) ConsentOvertimeRequest(ctx context.Context, ws, requestID, employeeID uuid.UUID, req api.EmployeeConsentRequest) error {
	db, err := r.db(ctx, ws)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE overtime_request_employees ore SET consent_status = $1, consent_at = NOW(), decline_reason = $2 WHERE ore.overtime_request_id = $3 AND ore.employment_id = (SELECT id FROM employments WHERE employee_id = $4 AND workspace_id = $5 ORDER BY start_date DESC LIMIT 1)`, req.Status, req.DeclineReason, requestID, employeeID, ws)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}

func (r OvertimeRepository) GetEmployee(ctx context.Context, ws, id uuid.UUID) (api.EmployeeRecord, error) {
	db, err := r.db(ctx, ws)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	err = db.QueryRow(ctx, `SELECT e.id, e.workspace_id, e.user_id, e.full_name, e.employee_id_number, e.tags, e.created_at, e.updated_at, NULL::text, (SELECT em.id FROM employments em WHERE em.workspace_id = e.workspace_id AND em.employee_id = e.id AND em.status = 'active' AND em.deleted_at IS NULL ORDER BY em.start_date DESC LIMIT 1), NULL::uuid, NULL::uuid FROM employees e WHERE e.id = $1 AND e.workspace_id = $2 AND e.deleted_at IS NULL`, id, ws).
		Scan(&record.ID, &record.WorkspaceID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &record.CreatedAt, &record.UpdatedAt, &record.DepartmentName, &record.ActiveEmploymentID, &record.EmployerID, &record.PrimaryLocationID)
	return record, err
}

// EmployeeDepartmentID resolves the target department for scoped OvertimeApprove checks.
func (r OvertimeRepository) EmployeeDepartmentID(ctx context.Context, ws, employeeID uuid.UUID) (*uuid.UUID, error) {
	db, err := r.db(ctx, ws)
	if err != nil {
		return nil, err
	}
	var id *uuid.UUID
	err = db.QueryRow(ctx, `SELECT department_id FROM employments WHERE workspace_id = $1 AND employee_id = $2 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC LIMIT 1`, ws, employeeID).Scan(&id)
	return id, err
}

func (r OvertimeRepository) ClaimOvertime(ctx context.Context, ws, employeeID uuid.UUID, req api.ClaimOTRequest, user uuid.UUID) error {
	tx, err := r.begin(ctx, ws)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		date, err := time.Parse("2006-01-02", req.Date)
		if err != nil {
			return err
		}
		var employmentID uuid.UUID
		if err := db.QueryRow(ctx, `SELECT id FROM employments WHERE workspace_id = $1 AND employee_id = $2 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC LIMIT 1`, ws, employeeID).Scan(&employmentID); err != nil {
			return err
		}
		var summaryID uuid.UUID
		err = db.QueryRow(ctx, `INSERT INTO attendance_daily_summaries (id, workspace_id, employment_id, date, claimed_ot_minutes, ot_reason, ot_status, entered_via, claim_entered_by_user_id, claim_entered_at) VALUES ($1, $2, $3, $4, $5, $6, 'pending_manager', 'self_service', $7, NOW()) ON CONFLICT (employment_id, date) DO UPDATE SET claimed_ot_minutes = EXCLUDED.claimed_ot_minutes, ot_reason = EXCLUDED.ot_reason, ot_status = 'pending_manager', entered_via = 'self_service', claim_entered_by_user_id = EXCLUDED.claim_entered_by_user_id, claim_entered_at = NOW(), updated_at = NOW() WHERE attendance_daily_summaries.ot_status IN ('pending_employee', 'rejected') RETURNING id`, uuid.New(), ws, employmentID, date, req.ClaimedMinutes, req.Reason, user).Scan(&summaryID)
		if err != nil {
			return err
		}
		return submitWorkflowInstance(ctx, db, ws, "ot_claim", summaryID)
	})
}

func (r OvertimeRepository) ApproveOvertime(ctx context.Context, ws, employeeID uuid.UUID, req api.ApproveOTRequest, _ uuid.UUID) error {
	db, err := r.db(ctx, ws)
	if err != nil {
		return err
	}
	date, err := time.Parse("2006-01-02", req.Date)
	if err != nil {
		return err
	}
	var employmentID uuid.UUID
	if err := db.QueryRow(ctx, `SELECT id FROM employments WHERE workspace_id = $1 AND employee_id = $2 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC LIMIT 1`, ws, employeeID).Scan(&employmentID); err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE attendance_daily_summaries SET approved_ot_minutes = $1, manager_note = $2, ot_status = $3, updated_at = NOW() WHERE workspace_id = $4 AND employment_id = $5 AND date = $6 AND ot_status = 'pending_manager'`, req.ApprovedMinutes, req.ManagerNote, req.Status, ws, employmentID, date)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}

var _ api.OvertimeRepository = OvertimeRepository{}
