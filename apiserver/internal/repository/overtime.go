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
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

// OvertimeRepository persists overtime policies, requests, and attendance OT claims.
type OvertimeRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r OvertimeRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "overtime repository")
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
	err := row.Scan(&p.ID, &p.Name, &p.Description, &p.LatestPublishedVersionID, &p.LatestPublishedVersionNo, &created, &updated)
	p.CreatedAt = created.UTC().Format(time.RFC3339)
	p.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return p, err
}

const overtimePolicyVersionColumns = `id, policy_id, version_no, status, row_version, daily_cap_minutes, monthly_cap_hours::text, yearly_cap_hours::text, rules, created_by, created_at, updated_at, published_by, published_at`

func parseOptionalDecimal(raw *string) (*decimal.Decimal, error) {
	if raw == nil || *raw == "" {
		return nil, nil
	}
	d, err := decimal.NewFromString(*raw)
	if err != nil {
		return nil, err
	}
	return &d, nil
}

func scanOvertimePolicyVersion(row interface{ Scan(...any) error }) (api.OvertimePolicyVersion, error) {
	var (
		item        api.OvertimePolicyVersion
		status      string
		monthlyCap  *string
		yearlyCap   *string
		rulesRaw    []byte
		createdAt   time.Time
		updatedAt   time.Time
		publishedAt *time.Time
	)
	if err := row.Scan(
		&item.ID, &item.PolicyID, &item.VersionNo, &status, &item.RowVersion,
		&item.DailyCapMinutes, &monthlyCap, &yearlyCap, &rulesRaw,
		&item.CreatedBy, &createdAt, &updatedAt, &item.PublishedBy, &publishedAt,
	); err != nil {
		return api.OvertimePolicyVersion{}, err
	}
	item.Status = workpolicy.VersionStatus(status)
	item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	item.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	if publishedAt != nil {
		v := publishedAt.UTC().Format(time.RFC3339)
		item.PublishedAt = &v
	}
	var err error
	if item.MonthlyCapHours, err = parseOptionalDecimal(monthlyCap); err != nil {
		return api.OvertimePolicyVersion{}, err
	}
	if item.YearlyCapHours, err = parseOptionalDecimal(yearlyCap); err != nil {
		return api.OvertimePolicyVersion{}, err
	}
	item.Rules = make([]workpolicy.OvertimeRule, 0)
	if len(rulesRaw) > 0 {
		if err := json.Unmarshal(rulesRaw, &item.Rules); err != nil {
			return api.OvertimePolicyVersion{}, err
		}
	}
	return item, nil
}

func marshalOvertimeRules(rules []workpolicy.OvertimeRule) ([]byte, error) {
	if rules == nil {
		rules = make([]workpolicy.OvertimeRule, 0)
	}
	return json.Marshal(rules)
}

func (r OvertimeRepository) ListOvertimePolicies(ctx context.Context, page, limit int) (api.OvertimePolicyListResponse, error) {
	db, err := requireDB(r.DB, "overtime repository")
	if err != nil {
		return api.OvertimePolicyListResponse{}, err
	}
	out := api.OvertimePolicyListResponse{Data: []api.OvertimePolicy{}, Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM overtime_policy_definitions`).Scan(&out.Total); err != nil {
		return out, err
	}
	rows, err := db.Query(ctx, `
		SELECT p.id, p.name, p.description, pub.id, pub.version_no, p.created_at, p.updated_at
		FROM overtime_policy_definitions p
		LEFT JOIN LATERAL (
			SELECT id, version_no
			FROM overtime_policy_versions
			WHERE policy_id = p.id AND status = 'published'
			ORDER BY version_no DESC
			LIMIT 1
		) pub ON TRUE
		ORDER BY p.created_at DESC
		LIMIT $1 OFFSET $2`, limit, (page-1)*limit)
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

func (r OvertimeRepository) CreateOvertimePolicy(ctx context.Context, req api.CreateOvertimePolicyRequest, actorID uuid.UUID) (api.OvertimePolicyDetail, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.OvertimePolicyDetail{}, err
	}
	rulesJSON, err := marshalOvertimeRules(req.Rules)
	if err != nil {
		return api.OvertimePolicyDetail{}, err
	}
	var out api.OvertimePolicyDetail
	err = service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
		policyID := uuid.New()
		var created, updated time.Time
		if err := tx.QueryRow(ctx, `
			INSERT INTO overtime_policy_definitions (id, name, description, created_at, updated_at)
			VALUES ($1, $2, $3, NOW(), NOW())
			RETURNING id, name, description, created_at, updated_at`,
			policyID, req.Name, req.Description,
		).Scan(&out.Policy.ID, &out.Policy.Name, &out.Policy.Description, &created, &updated); err != nil {
			return err
		}
		out.Policy.CreatedAt = created.UTC().Format(time.RFC3339)
		out.Policy.UpdatedAt = updated.UTC().Format(time.RFC3339)
		version, err := scanOvertimePolicyVersion(tx.QueryRow(ctx, `
			INSERT INTO overtime_policy_versions (
				id, policy_id, version_no, status, row_version,
				daily_cap_minutes, monthly_cap_hours, yearly_cap_hours, rules, created_by
			)
			VALUES ($1, $2, 1, 'draft', 1, $3, $4, $5, $6::jsonb, $7)
			RETURNING `+overtimePolicyVersionColumns,
			uuid.New(), policyID, req.DailyCapMinutes, req.MonthlyCapHours, req.YearlyCapHours, string(rulesJSON), actorID,
		))
		if err != nil {
			return mapConstraintError(err)
		}
		out.Versions = []api.OvertimePolicyVersion{version}
		return nil
	})
	return out, err
}

func (r OvertimeRepository) GetOvertimePolicy(ctx context.Context, id uuid.UUID) (api.OvertimePolicyDetail, error) {
	db, err := requireDB(r.DB, "overtime repository")
	if err != nil {
		return api.OvertimePolicyDetail{}, err
	}
	policy, err := scanPolicy(db.QueryRow(ctx, `
		SELECT p.id, p.name, p.description, pub.id, pub.version_no, p.created_at, p.updated_at
		FROM overtime_policy_definitions p
		LEFT JOIN LATERAL (
			SELECT id, version_no
			FROM overtime_policy_versions
			WHERE policy_id = p.id AND status = 'published'
			ORDER BY version_no DESC
			LIMIT 1
		) pub ON TRUE
		WHERE p.id = $1`, id))
	if err != nil {
		return api.OvertimePolicyDetail{}, err
	}
	versions, err := r.ListOvertimePolicyVersions(ctx, id)
	if err != nil {
		return api.OvertimePolicyDetail{}, err
	}
	return api.OvertimePolicyDetail{Policy: policy, Versions: versions}, nil
}

func (r OvertimeRepository) ListOvertimePolicyVersions(ctx context.Context, policyID uuid.UUID) ([]api.OvertimePolicyVersion, error) {
	db, err := requireDB(r.DB, "overtime repository")
	if err != nil {
		return nil, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM overtime_policy_definitions WHERE id = $1)`, policyID).Scan(&exists); err != nil {
		return nil, err
	}
	if !exists {
		return nil, errNoRows
	}
	rows, err := db.Query(ctx, `
		SELECT `+overtimePolicyVersionColumns+`
		FROM overtime_policy_versions
		WHERE policy_id = $1
		ORDER BY version_no DESC, id DESC`, policyID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	out := make([]api.OvertimePolicyVersion, 0)
	for rows.Next() {
		v, err := scanOvertimePolicyVersion(rows)
		if err != nil {
			return nil, err
		}
		out = append(out, v)
	}
	return out, rows.Err()
}

func (r OvertimeRepository) CreateOvertimePolicyVersion(ctx context.Context, policyID uuid.UUID, req api.CreateOvertimePolicyVersionRequest, actorID uuid.UUID) (api.OvertimePolicyVersion, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.OvertimePolicyVersion{}, err
	}
	rulesJSON, err := marshalOvertimeRules(req.Rules)
	if err != nil {
		return api.OvertimePolicyVersion{}, err
	}
	var version api.OvertimePolicyVersion
	err = service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
		var exists bool
		if err := tx.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM overtime_policy_definitions WHERE id = $1 FOR UPDATE)`, policyID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return errNoRows
		}
		var nextVersion int
		if err := tx.QueryRow(ctx, `SELECT COALESCE(MAX(version_no), 0) + 1 FROM overtime_policy_versions WHERE policy_id = $1`, policyID).Scan(&nextVersion); err != nil {
			return err
		}
		version, err = scanOvertimePolicyVersion(tx.QueryRow(ctx, `
			INSERT INTO overtime_policy_versions (
				id, policy_id, version_no, status, row_version,
				daily_cap_minutes, monthly_cap_hours, yearly_cap_hours, rules, created_by
			)
			VALUES ($1, $2, $3, 'draft', 1, $4, $5, $6, $7::jsonb, $8)
			RETURNING `+overtimePolicyVersionColumns,
			uuid.New(), policyID, nextVersion, req.DailyCapMinutes, req.MonthlyCapHours, req.YearlyCapHours, string(rulesJSON), actorID,
		))
		if err != nil {
			return mapConstraintError(err)
		}
		return nil
	})
	return version, err
}

func (r OvertimeRepository) PatchOvertimePolicyVersion(ctx context.Context, policyID, versionID uuid.UUID, req api.PatchOvertimePolicyVersionRequest) (api.OvertimePolicyVersion, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.OvertimePolicyVersion{}, err
	}
	rulesJSON, err := marshalOvertimeRules(req.Rules)
	if err != nil {
		return api.OvertimePolicyVersion{}, err
	}
	var updated api.OvertimePolicyVersion
	err = service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
		var status string
		var rowVersion int64
		if err := tx.QueryRow(ctx, `
			SELECT status, row_version
			FROM overtime_policy_versions
			WHERE policy_id = $1 AND id = $2
			FOR UPDATE`, policyID, versionID).Scan(&status, &rowVersion); err != nil {
			return err
		}
		if status != string(workpolicy.StatusDraft) || rowVersion != req.ExpectedRowVersion {
			return api.ErrOvertimeConflict
		}
		updated, err = scanOvertimePolicyVersion(tx.QueryRow(ctx, `
			UPDATE overtime_policy_versions
			SET daily_cap_minutes = $3,
			    monthly_cap_hours = $4,
			    yearly_cap_hours = $5,
			    rules = $6::jsonb,
			    row_version = row_version + 1,
			    updated_at = CURRENT_TIMESTAMP
			WHERE policy_id = $1 AND id = $2 AND status = 'draft' AND row_version = $7
			RETURNING `+overtimePolicyVersionColumns,
			policyID, versionID, req.DailyCapMinutes, req.MonthlyCapHours, req.YearlyCapHours, string(rulesJSON), req.ExpectedRowVersion,
		))
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrOvertimeConflict
		}
		return mapConstraintError(err)
	})
	return updated, err
}

func (r OvertimeRepository) PublishOvertimePolicyVersion(ctx context.Context, policyID, versionID uuid.UUID, req api.PublishOvertimePolicyVersionRequest, actorID uuid.UUID) (api.OvertimePolicyVersion, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.OvertimePolicyVersion{}, err
	}
	var published api.OvertimePolicyVersion
	err = service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
		current, err := scanOvertimePolicyVersion(tx.QueryRow(ctx, `
			SELECT `+overtimePolicyVersionColumns+`
			FROM overtime_policy_versions
			WHERE policy_id = $1 AND id = $2
			FOR UPDATE`, policyID, versionID))
		if err != nil {
			return err
		}
		if current.Status != workpolicy.StatusDraft || current.RowVersion != req.ExpectedRowVersion {
			return api.ErrOvertimeConflict
		}
		if len(current.Rules) == 0 {
			return api.ErrOvertimeValidation
		}
		spec := workpolicy.OvertimePolicyVersionSpec{
			DailyCapMinutes: current.DailyCapMinutes,
			MonthlyCapHours: current.MonthlyCapHours,
			YearlyCapHours:  current.YearlyCapHours,
			Rules:           current.Rules,
		}
		if err := spec.Validate(); err != nil {
			return api.ErrOvertimeValidation
		}
		published, err = scanOvertimePolicyVersion(tx.QueryRow(ctx, `
			UPDATE overtime_policy_versions
			SET status = 'published',
			    published_by = $3,
			    published_at = CURRENT_TIMESTAMP,
			    row_version = row_version + 1,
			    updated_at = CURRENT_TIMESTAMP
			WHERE policy_id = $1 AND id = $2 AND status = 'draft' AND row_version = $4
			RETURNING `+overtimePolicyVersionColumns,
			policyID, versionID, actorID, req.ExpectedRowVersion,
		))
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrOvertimeConflict
		}
		return mapConstraintError(err)
	})
	return published, err
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
	db, err := requireDB(r.DB, "overtime repository")
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
			if err := db.QueryRow(ctx, `SELECT id FROM employment_segments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL AND start_date <= $2 AND (end_date IS NULL OR end_date >= $2) ORDER BY start_date DESC LIMIT 1`, employeeID, planned).Scan(&employmentID); err != nil {
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
	db, err := requireDB(r.DB, "overtime repository")
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
			return errNoRows
		}
		return submitWorkflowInstance(ctx, db, "ot_request", id)
	})
}

func (r OvertimeRepository) ConsentOvertimeRequest(ctx context.Context, requestID, employeeID uuid.UUID, req api.EmployeeConsentRequest) error {
	db, err := requireDB(r.DB, "overtime repository")
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE overtime_request_employees ore SET consent_status = $1, consent_at = NOW(), decline_reason = $2 WHERE ore.overtime_request_id = $3 AND ore.employment_id = (SELECT id FROM employment_segments WHERE employee_id = $4 ORDER BY start_date DESC LIMIT 1)`, req.Status, req.DeclineReason, requestID, employeeID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return errNoRows
	}
	return nil
}

func (r OvertimeRepository) GetEmployee(ctx context.Context, id uuid.UUID) (api.EmployeeRecord, error) {
	db, err := requireDB(r.DB, "overtime repository")
	if err != nil {
		return api.EmployeeRecord{}, err
	}
	var record api.EmployeeRecord
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, `SELECT e.id, tenure.user_id, e.full_name, COALESCE(tenure.employee_number, ''), e.tags, e.created_at, e.updated_at, NULL::text, (SELECT em.id FROM employment_segments em WHERE em.employee_id = e.id AND em.status = 'active' AND em.deleted_at IS NULL ORDER BY em.start_date DESC LIMIT 1), NULL::uuid, NULL::uuid FROM employees e `+sqlEffectiveTenureLateral+` WHERE e.id = $1 AND e.deleted_at IS NULL`, id).
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
	db, err := requireDB(r.DB, "overtime repository")
	if err != nil {
		return nil, err
	}
	var id *uuid.UUID
	err = db.QueryRow(ctx, `SELECT department_id FROM employment_segments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC LIMIT 1`, employeeID).Scan(&id)
	return id, err
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
		if err := db.QueryRow(ctx, `SELECT id FROM employment_segments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC LIMIT 1`, employeeID).Scan(&employmentID); err != nil {
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
	db, err := requireDB(r.DB, "overtime repository")
	if err != nil {
		return err
	}
	date, err := time.Parse("2006-01-02", req.Date)
	if err != nil {
		return err
	}
	var employmentID uuid.UUID
	if err := db.QueryRow(ctx, `SELECT id FROM employment_segments WHERE employee_id = $1 AND status = 'active' AND deleted_at IS NULL ORDER BY start_date DESC LIMIT 1`, employeeID).Scan(&employmentID); err != nil {
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
		return errNoRows
	}
	return nil
}

var _ api.OvertimeRepository = OvertimeRepository{}
