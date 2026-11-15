package repository

import (
	"context"
	"encoding/json"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// OvertimeRepository persists overtime policies, requests, and attendance OT claims.
type OvertimeRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r OvertimeRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("overtime repository: database is not configured")
	}
	return r.DB, nil
}

func (r OvertimeRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	tx, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("overtime repository: database does not support transactions")
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
	err := row.Scan(&p.ID, &p.Name, &p.Description, &created, &updated)
	p.CreatedAt = created.UTC().Format(time.RFC3339)
	p.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return p, err
}

func (r OvertimeRepository) ListOvertimePolicies(ctx context.Context, page, limit int) (api.OvertimePolicyListResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.OvertimePolicyListResponse{}, err
	}
	out := api.OvertimePolicyListResponse{Data: []api.OvertimePolicy{}, Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM overtime_policy_definitions`).Scan(&out.Total); err != nil {
		return out, err
	}
	rows, err := db.Query(ctx, `SELECT id, name, description, created_at, updated_at FROM overtime_policy_definitions ORDER BY created_at DESC LIMIT $1 OFFSET $2`, limit, (page-1)*limit)
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

func (r OvertimeRepository) CreateOvertimePolicy(ctx context.Context, req api.CreateOvertimePolicyRequest) (api.OvertimePolicy, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.OvertimePolicy{}, err
	}
	return scanPolicy(db.QueryRow(ctx, `INSERT INTO overtime_policy_definitions (id, name, description, created_at, updated_at) VALUES ($1, $2, $3, NOW(), NOW()) RETURNING id, name, description, created_at, updated_at`, uuid.New(), req.Name, req.Description))
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

func (r OvertimeRepository) GetOvertimePolicy(ctx context.Context, id uuid.UUID) (api.OvertimePolicyDetail, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.OvertimePolicyDetail{}, err
	}
	policy, err := scanPolicy(db.QueryRow(ctx, `SELECT id, name, description, created_at, updated_at FROM overtime_policy_definitions WHERE id = $1`, id))
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

func (r OvertimeRepository) AddOvertimeRule(ctx context.Context, policyID uuid.UUID, req api.CreateOvertimeRuleRequest) (api.OvertimeRule, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.OvertimeRule{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM overtime_policy_definitions WHERE id = $1)`, policyID).Scan(&exists); err != nil {
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

func (r OvertimeRepository) AssignOvertimePolicy(ctx context.Context, req api.AssignOvertimePolicyRequest) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	date, err := time.Parse("2006-01-02", req.EffectiveFrom)
	if err != nil {
		return err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM overtime_policy_definitions WHERE id = $1)`, req.PolicyID).Scan(&exists); err != nil {
		return err
	}
	if !exists {
		return pgx.ErrNoRows
	}
	_, err = db.Exec(ctx, `INSERT INTO overtime_policy_assignments (id, assignee_type, assignee_id, policy_id, effective_from, created_at) VALUES ($1, $2, $3, $4, $5, NOW())`, uuid.New(), req.AssigneeType, req.AssigneeID, req.PolicyID, date)
	return err
}

const overtimeRequestColumns = `id, requested_by_user_id, department_id, planned_date, planned_start_time, planned_end_time, planned_minutes, reason, status::text, workflow_id, current_step, quota_warning, reconciled_at, reconciled_by, attachment_file_ids, created_at, updated_at`

func scanOvertimeRequest(row interface{ Scan(...any) error }) (api.OvertimeRequest, error) {
	var item api.OvertimeRequest
	var planned, created, updated time.Time
	var start, end, reconciled *time.Time
	err := row.Scan(&item.ID, &item.RequestedByUserID, &item.DepartmentID, &planned, &start, &end, &item.PlannedMinutes, &item.Reason, &item.Status, &item.WorkflowID, &item.CurrentStep, &item.QuotaWarning, &reconciled, &item.ReconciledBy, &item.AttachmentFileIDs, &created, &updated)
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

func (r OvertimeRepository) ListOvertimeRequests(ctx context.Context, page, limit int) (api.OvertimeRequestListResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.OvertimeRequestListResponse{}, err
	}
	out := api.OvertimeRequestListResponse{Data: []api.OvertimeRequest{}, Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM overtime_requests`).Scan(&out.Total); err != nil {
		return out, err
	}
	rows, err := db.Query(ctx, `SELECT `+overtimeRequestColumns+` FROM overtime_requests ORDER BY created_at DESC LIMIT $1 OFFSET $2`, limit, (page-1)*limit)
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

func (r OvertimeRepository) CreateOvertimeRequest(ctx context.Context, user uuid.UUID, req api.CreateOvertimeRequest) (api.OvertimeRequest, error) {
	tx, err := r.begin(ctx)
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
		out, err = scanOvertimeRequest(db.QueryRow(ctx, `INSERT INTO overtime_requests (id, requested_by_user_id, department_id, planned_date, planned_start_time, planned_end_time, planned_minutes, reason, status, current_step, attachment_file_ids, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'draft', 0, $9, NOW(), NOW()) RETURNING `+overtimeRequestColumns, id, user, req.DepartmentID, planned, start, end, req.PlannedMinutes, req.Reason, req.AttachmentFileIDs))
		if err != nil {
			return err
		}
		for _, employeeID := range req.Employees {
			var employmentID uuid.UUID
			if err := db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL AND start_date <= $2 AND (end_date IS NULL OR end_date >= $2) ORDER BY start_date DESC LIMIT 1`, employeeID, planned).Scan(&employmentID); err != nil {
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

func (r OvertimeRepository) GetOvertimeRequest(ctx context.Context, id uuid.UUID) (api.OvertimeRequestDetail, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.OvertimeRequestDetail{}, err
	}
	request, err := scanOvertimeRequest(db.QueryRow(ctx, `SELECT `+overtimeRequestColumns+` FROM overtime_requests WHERE id = $1`, id))
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

func workflowSnapshot(ctx context.Context, db pgx.Tx, entityType string) (json.RawMessage, *uuid.UUID, error) {
	var workflowID uuid.UUID
	err := db.QueryRow(ctx, `SELECT id FROM approval_workflows WHERE entity_type = $1 AND is_active = TRUE ORDER BY created_at DESC LIMIT 1`, entityType).Scan(&workflowID)
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

func submitWorkflowInstance(ctx context.Context, db pgx.Tx, entityType string, entityID uuid.UUID) error {
	snapshot, workflowID, err := workflowSnapshot(ctx, db, entityType)
	if err != nil {
		return err
	}
	if _, err = db.Exec(ctx, `UPDATE approval_workflow_instances SET status = 'cancelled', updated_at = NOW() WHERE entity_type = $1 AND entity_id = $2 AND status = 'pending'`, entityType, entityID); err != nil {
		return err
	}
	_, err = db.Exec(ctx, `INSERT INTO approval_workflow_instances (id, workflow_id, entity_type, entity_id, current_step_order, status, steps_snapshot, created_at, updated_at) VALUES ($1, $2, $3, $4, 1, 'pending', $5, NOW(), NOW())`, uuid.New(), workflowID, entityType, entityID, snapshot)
	return err
}

func (r OvertimeRepository) SubmitOvertimeRequest(ctx context.Context, id uuid.UUID) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		tag, err := db.Exec(ctx, `UPDATE overtime_requests SET status = 'pending_approval', current_step = 1, updated_at = NOW() WHERE id = $1`, id)
		if err != nil {
			return err
		}
		if tag.RowsAffected() == 0 {
			return pgx.ErrNoRows
		}
		return submitWorkflowInstance(ctx, db, "ot_request", id)
	})
}

func (r OvertimeRepository) ConsentOvertimeRequest(ctx context.Context, requestID, employeeID uuid.UUID, req api.EmployeeConsentRequest) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE overtime_request_employees ore SET consent_status = $1, consent_at = NOW(), decline_reason = $2 WHERE ore.overtime_request_id = $3 AND ore.employment_id = (SELECT id FROM employments WHERE employee_id = $4 ORDER BY start_date DESC LIMIT 1)`, req.Status, req.DeclineReason, requestID, employeeID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}

func (r OvertimeRepository) GetEmployee(ctx context.Context, id uuid.UUID) (api.EmployeeRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, `SELECT e.id, e.user_id, e.full_name, e.employee_id_number, e.tags, e.created_at, e.updated_at, NULL::text, (SELECT em.id FROM employments em WHERE em.employee_id = e.id AND em.status = 'active' AND em.deleted_at IS NULL ORDER BY em.start_date DESC LIMIT 1), NULL::uuid, NULL::uuid FROM employees e WHERE e.id = $1 AND e.deleted_at IS NULL`, id).
		Scan(&record.ID, &record.UserID, &record.FullName, &record.EmployeeIDNumber, &record.Tags, &createdAt, &updatedAt, &record.DepartmentName, &record.ActiveEmploymentID, &record.EmployerID, &record.PrimaryLocationID)
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	record.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	record.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return record, nil
}

// EmployeeDepartmentID resolves the target department for scoped OvertimeApprove checks.
func (r OvertimeRepository) EmployeeDepartmentID(ctx context.Context, employeeID uuid.UUID) (*uuid.UUID, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	var id *uuid.UUID
	err = db.QueryRow(ctx, `SELECT department_id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC LIMIT 1`, employeeID).Scan(&id)
	return id, err
}

type overtimeMultiplierAssignment struct {
	assigneeType  string
	effectiveFrom time.Time
	effectiveTo   *time.Time
	multiplier    decimal.Decimal
	priority      int
	ruleID        uuid.UUID
}

func loadOvertimeMultiplierAssignments(ctx context.Context, db DBTX, policyAssignments []workPolicyAssignment) ([]overtimeMultiplierAssignment, error) {
	policyIDs := make([]uuid.UUID, 0, len(policyAssignments))
	seenPolicyIDs := make(map[uuid.UUID]struct{}, len(policyAssignments))
	for _, assignment := range policyAssignments {
		policyID := assignment.policy.OvertimePolicyID
		if policyID == uuid.Nil {
			continue
		}
		if _, exists := seenPolicyIDs[policyID]; !exists {
			seenPolicyIDs[policyID] = struct{}{}
			policyIDs = append(policyIDs, policyID)
		}
	}
	if len(policyIDs) == 0 {
		return []overtimeMultiplierAssignment{}, nil
	}

	rows, err := db.Query(ctx, `
		SELECT policy_id, multiplier, priority, id
		FROM overtime_rules
		WHERE policy_id = ANY($1)
		ORDER BY priority DESC, id DESC`, policyIDs)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	type rule struct {
		multiplier decimal.Decimal
		priority   int
		id         uuid.UUID
	}
	rulesByPolicy := make(map[uuid.UUID][]rule, len(policyIDs))
	for rows.Next() {
		var policyID uuid.UUID
		var item rule
		if err := rows.Scan(&policyID, &item.multiplier, &item.priority, &item.id); err != nil {
			return nil, err
		}
		rulesByPolicy[policyID] = append(rulesByPolicy[policyID], item)
	}
	if err := rows.Err(); err != nil {
		return nil, err
	}

	assignments := make([]overtimeMultiplierAssignment, 0)
	for _, policyAssignment := range policyAssignments {
		from, err := policyAssignmentStart(policyAssignment)
		if err != nil {
			return nil, err
		}
		to, err := policyAssignmentEnd(policyAssignment)
		if err != nil {
			return nil, err
		}
		if to != nil && !to.After(from) {
			continue
		}
		inclusiveTo := (*time.Time)(nil)
		if to != nil {
			value := to.AddDate(0, 0, -1)
			inclusiveTo = &value
		}
		for _, item := range rulesByPolicy[policyAssignment.policy.OvertimePolicyID] {
			assignments = append(assignments, overtimeMultiplierAssignment{
				assigneeType:  "labor_policy",
				effectiveFrom: from,
				effectiveTo:   inclusiveTo,
				multiplier:    item.multiplier,
				priority:      item.priority,
				ruleID:        item.id,
			})
		}
	}
	return assignments, nil
}

func policyAssignmentStart(assignment workPolicyAssignment) (time.Time, error) {
	policyFrom, err := time.Parse("2006-01-02", assignment.policy.EffectiveFrom)
	if err != nil {
		return time.Time{}, err
	}
	membershipFrom, err := time.Parse("2006-01-02", assignment.membershipFrom)
	if err != nil {
		return time.Time{}, err
	}
	if membershipFrom.After(policyFrom) {
		return membershipFrom, nil
	}
	return policyFrom, nil
}

func policyAssignmentEnd(assignment workPolicyAssignment) (*time.Time, error) {
	var result *time.Time
	for _, raw := range []*string{assignment.policy.EffectiveTo, assignment.membershipTo} {
		if raw == nil {
			continue
		}
		value, err := time.Parse("2006-01-02", *raw)
		if err != nil {
			return nil, err
		}
		if result == nil || value.Before(*result) {
			copy := value
			result = &copy
		}
	}
	return result, nil
}

func overtimeMultiplierAt(assignments []overtimeMultiplierAssignment, asOf time.Time) decimal.Decimal {
	fallback := decimal.RequireFromString("1.5")
	for _, assignment := range assignments {
		if asOf.Before(assignment.effectiveFrom) || assignment.effectiveTo != nil && asOf.After(*assignment.effectiveTo) {
			continue
		}
		if assignment.multiplier.IsPositive() {
			return assignment.multiplier
		}
		return fallback
	}
	return fallback
}

func (r OvertimeRepository) ClaimOvertime(ctx context.Context, employeeID uuid.UUID, req api.ClaimOTRequest, user uuid.UUID) error {
	tx, err := r.begin(ctx)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		date, err := time.Parse("2006-01-02", req.Date)
		if err != nil {
			return err
		}
		var employmentID uuid.UUID
		if err := db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC LIMIT 1`, employeeID).Scan(&employmentID); err != nil {
			return err
		}
		var detectedMinutes int
		var status string
		if err := db.QueryRow(ctx, `SELECT system_detected_ot_minutes, ot_status::text FROM attendance_daily_summaries WHERE employment_id = $1 AND date = $2 FOR UPDATE`, employmentID, date).Scan(&detectedMinutes, &status); err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.ErrOvertimeValidation
			}
			return err
		}
		if detectedMinutes <= 0 || req.ClaimedMinutes > detectedMinutes || status != "pending_employee" && status != "rejected" {
			return api.ErrOvertimeValidation
		}
		var summaryID uuid.UUID
		err = db.QueryRow(ctx, `UPDATE attendance_daily_summaries SET claimed_ot_minutes = $1, ot_reason = $2, ot_status = 'pending_manager', entered_via = 'self_service', claim_entered_by_user_id = $3, claim_entered_at = NOW(), updated_at = NOW() WHERE employment_id = $4 AND date = $5 RETURNING id`, req.ClaimedMinutes, req.Reason, user, employmentID, date).Scan(&summaryID)
		if err != nil {
			return err
		}
		return submitWorkflowInstance(ctx, db, "ot_claim", summaryID)
	})
}

func (r OvertimeRepository) ApproveOvertime(ctx context.Context, employeeID uuid.UUID, req api.ApproveOTRequest, _ uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	date, err := time.Parse("2006-01-02", req.Date)
	if err != nil {
		return err
	}
	var employmentID uuid.UUID
	if err := db.QueryRow(ctx, `SELECT id FROM employments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC LIMIT 1`, employeeID).Scan(&employmentID); err != nil {
		return err
	}
	approvedMinutes := req.ApprovedMinutes
	if req.Status == "rejected" {
		approvedMinutes = 0
	}
	tag, err := db.Exec(ctx, `UPDATE attendance_daily_summaries SET approved_ot_minutes = $1, manager_note = $2, ot_status = $3, updated_at = NOW() WHERE employment_id = $4 AND date = $5 AND ot_status = 'pending_manager' AND $1 <= claimed_ot_minutes AND $1 <= system_detected_ot_minutes`, approvedMinutes, req.ManagerNote, req.Status, employmentID, date)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return pgx.ErrNoRows
	}
	return nil
}

var _ api.OvertimeRepository = OvertimeRepository{}
