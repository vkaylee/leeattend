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
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type ApprovalWorkflowRepository struct {
	TenantDB func(context.Context, uuid.UUID) (DBTX, error)
	BeginTx  func(context.Context, uuid.UUID) (TxBeginner, error)
}

func (r ApprovalWorkflowRepository) db(ctx context.Context, workspaceID uuid.UUID) (DBTX, error) {
	if r.TenantDB == nil {
		return nil, errors.New("approval workflow repository: tenant database is not configured")
	}
	return r.TenantDB(ctx, workspaceID)
}

func (r ApprovalWorkflowRepository) begin(ctx context.Context, workspaceID uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, workspaceID)
	}
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("approval workflow repository: tenant database does not support transactions")
	}
	return beginner, nil
}

func (r ApprovalWorkflowRepository) ListWorkflows(ctx context.Context, workspaceID uuid.UUID, entityType *string, page, limit int) (api.ApprovalWorkflowListResponse, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.ApprovalWorkflowListResponse{}, err
	}
	response := api.ApprovalWorkflowListResponse{Data: make([]api.ApprovalWorkflowRecord, 0), Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM approval_workflows WHERE workspace_id = $1 AND ($2::text IS NULL OR entity_type = $2)`, workspaceID, entityType).Scan(&response.Total); err != nil {
		return api.ApprovalWorkflowListResponse{}, err
	}
	offset := (page - 1) * limit
	rows, err := db.Query(ctx, `SELECT id, workspace_id, entity_type, name, description, total_steps, is_active, created_by, created_at, updated_at FROM approval_workflows WHERE workspace_id = $1 AND ($2::text IS NULL OR entity_type = $2) ORDER BY created_at DESC LIMIT $3 OFFSET $4`, workspaceID, entityType, limit, offset)
	if err != nil {
		return api.ApprovalWorkflowListResponse{}, err
	}
	defer rows.Close()
	for rows.Next() {
		var item api.ApprovalWorkflowRecord
		var createdAt, updatedAt time.Time
		if err := rows.Scan(&item.ID, &item.WorkspaceID, &item.EntityType, &item.Name, &item.Description, &item.TotalSteps, &item.IsActive, &item.CreatedBy, &createdAt, &updatedAt); err != nil {
			return api.ApprovalWorkflowListResponse{}, err
		}
		item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		item.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
		response.Data = append(response.Data, item)
	}
	return response, rows.Err()
}

func (r ApprovalWorkflowRepository) CreateWorkflow(ctx context.Context, workspaceID, creatorID uuid.UUID, req api.CreateWorkflowRequest) (api.ApprovalWorkflowRecord, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.ApprovalWorkflowRecord{}, err
	}
	workflowID := uuid.New()
	now := time.Now().UTC()
	result := api.ApprovalWorkflowRecord{
		ID:          workflowID,
		WorkspaceID: workspaceID,
		EntityType:  req.EntityType,
		Name:        req.Name,
		Description: req.Description,
		TotalSteps:  len(req.Steps),
		IsActive:    req.IsActive,
		CreatedBy:   &creatorID,
		Steps:       make([]api.WorkflowStepRecord, 0, len(req.Steps)),
	}
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if _, err := db.Exec(ctx, `INSERT INTO approval_workflows (id, workspace_id, entity_type, name, description, total_steps, is_active, created_by, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $9)`, workflowID, workspaceID, req.EntityType, req.Name, req.Description, len(req.Steps), req.IsActive, creatorID, now); err != nil {
			return err
		}
		for _, step := range req.Steps {
			if step.StepOrder < 1 || step.StepName == "" || step.ApproverType == "" {
				return api.ErrWorkflowValidation
			}
			config := step.ApproverConfig
			if len(config) == 0 {
				config = json.RawMessage(`{}`)
			}
			stepID := uuid.New()
			if _, err := db.Exec(ctx, `INSERT INTO approval_workflow_steps (id, workflow_id, step_order, step_name, approver_type, approver_config, action_required, can_skip, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)`, stepID, workflowID, step.StepOrder, step.StepName, step.ApproverType, config, step.ActionRequired, step.CanSkip, now); err != nil {
				return err
			}
			result.Steps = append(result.Steps, api.WorkflowStepRecord{
				ID:             stepID,
				WorkflowID:     workflowID,
				StepOrder:      step.StepOrder,
				StepName:       step.StepName,
				ApproverType:   step.ApproverType,
				ApproverConfig: config,
				ActionRequired: step.ActionRequired,
				CanSkip:        step.CanSkip,
			})
		}
		result.CreatedAt = now.Format(time.RFC3339)
		result.UpdatedAt = result.CreatedAt
		return nil
	})
	if err != nil {
		return api.ApprovalWorkflowRecord{}, err
	}
	return result, nil
}

func (r ApprovalWorkflowRepository) GetWorkflow(ctx context.Context, workspaceID, workflowID uuid.UUID) (api.ApprovalWorkflowRecord, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.ApprovalWorkflowRecord{}, err
	}
	var item api.ApprovalWorkflowRecord
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, `SELECT id, workspace_id, entity_type, name, description, total_steps, is_active, created_by, created_at, updated_at FROM approval_workflows WHERE id = $1 AND workspace_id = $2`, workflowID, workspaceID).
		Scan(&item.ID, &item.WorkspaceID, &item.EntityType, &item.Name, &item.Description, &item.TotalSteps, &item.IsActive, &item.CreatedBy, &createdAt, &updatedAt)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ApprovalWorkflowRecord{}, api.ErrWorkflowNotFound
		}
		return api.ApprovalWorkflowRecord{}, err
	}
	item.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	item.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	rows, err := db.Query(ctx, `SELECT id, workflow_id, step_order, step_name, approver_type, approver_config, action_required, can_skip FROM approval_workflow_steps WHERE workflow_id = $1 ORDER BY step_order ASC`, workflowID)
	if err != nil {
		return api.ApprovalWorkflowRecord{}, err
	}
	defer rows.Close()
	item.Steps = make([]api.WorkflowStepRecord, 0)
	for rows.Next() {
		var step api.WorkflowStepRecord
		if err := rows.Scan(&step.ID, &step.WorkflowID, &step.StepOrder, &step.StepName, &step.ApproverType, &step.ApproverConfig, &step.ActionRequired, &step.CanSkip); err != nil {
			return api.ApprovalWorkflowRecord{}, err
		}
		item.Steps = append(item.Steps, step)
	}
	if err := rows.Err(); err != nil {
		return api.ApprovalWorkflowRecord{}, err
	}
	return item, nil
}

func (r ApprovalWorkflowRepository) UpdateWorkflow(ctx context.Context, workspaceID, workflowID uuid.UUID, req api.UpdateWorkflowRequest) (api.ApprovalWorkflowRecord, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.ApprovalWorkflowRecord{}, err
	}
	sets := []string{"updated_at = CURRENT_TIMESTAMP"}
	args := []any{}
	add := func(column string, value any) {
		args = append(args, value)
		sets = append(sets, fmt.Sprintf("%s = $%d", column, len(args)))
	}
	if req.Name != nil {
		add("name", *req.Name)
	}
	if req.Description != nil {
		add("description", *req.Description)
	}
	if req.IsActive != nil {
		add("is_active", *req.IsActive)
	}
	args = append(args, workflowID, workspaceID)
	tag, err := db.Exec(ctx, fmt.Sprintf(`UPDATE approval_workflows SET %s WHERE id = $%d AND workspace_id = $%d`, strings.Join(sets, ", "), len(args)-1, len(args)), args...)
	if err != nil {
		return api.ApprovalWorkflowRecord{}, err
	}
	if tag.RowsAffected() == 0 {
		return api.ApprovalWorkflowRecord{}, api.ErrWorkflowNotFound
	}
	return r.GetWorkflow(ctx, workspaceID, workflowID)
}

func (r ApprovalWorkflowRepository) DeleteWorkflow(ctx context.Context, workspaceID, workflowID uuid.UUID) error {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `DELETE FROM approval_workflows WHERE id = $1 AND workspace_id = $2`, workflowID, workspaceID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrWorkflowNotFound
	}
	return nil
}

func (r ApprovalWorkflowRepository) ProcessApproval(ctx context.Context, workspaceID, actorID uuid.UUID, req api.ProcessApprovalRequest) error {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return err
	}
	return service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		var instanceID uuid.UUID
		var stepOrder int
		var stepName *string
		err := db.QueryRow(ctx, `SELECT i.id, i.current_step_order, s.step_name FROM approval_workflow_instances i LEFT JOIN approval_workflow_steps s ON s.workflow_id = i.workflow_id AND s.step_order = i.current_step_order WHERE i.workspace_id = $1 AND i.entity_type = $2 AND i.entity_id = $3 AND i.status = 'pending' FOR UPDATE`, workspaceID, req.EntityType, req.EntityID).Scan(&instanceID, &stepOrder, &stepName)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrWorkflowNotFound
		}
		if err != nil {
			return err
		}
		action := normalizeApprovalAction(req.Action)
		if _, err := db.Exec(ctx, `INSERT INTO approval_action_log (id, workspace_id, entity_type, entity_id, step_order, step_name, acted_by_user_id, action, note, acted_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now())`, uuid.New(), workspaceID, req.EntityType, req.EntityID, stepOrder, stepName, actorID, action, req.Note); err != nil {
			return err
		}
		status := "pending"
		switch action {
		case "approved":
			status = "approved"
		case "rejected":
			status = "rejected"
		}
		_, err = db.Exec(ctx, `UPDATE approval_workflow_instances SET status = $1, updated_at = now() WHERE id = $2`, status, instanceID)
		return err
	})
}

func normalizeApprovalAction(action string) string {
	switch action {
	case "approve", "approved":
		return "approved"
	case "reject", "rejected":
		return "rejected"
	case "return", "returned":
		return "returned"
	case "reassign", "reassigned":
		return "reassigned"
	default:
		return "skipped"
	}
}

func (r ApprovalWorkflowRepository) ListHistory(ctx context.Context, workspaceID uuid.UUID, entityType string, entityID uuid.UUID) ([]api.ApprovalActionLogRecord, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, workspace_id, entity_type, entity_id, step_order, step_name, acted_by_user_id, action, note, acted_at FROM approval_action_log WHERE workspace_id = $1 AND entity_type = $2 AND entity_id = $3 ORDER BY acted_at ASC`, workspaceID, entityType, entityID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	logs := make([]api.ApprovalActionLogRecord, 0)
	for rows.Next() {
		var item api.ApprovalActionLogRecord
		var actedAt time.Time
		if err := rows.Scan(&item.ID, &item.WorkspaceID, &item.EntityType, &item.EntityID, &item.StepOrder, &item.StepName, &item.ActedByUserID, &item.Action, &item.Note, &actedAt); err != nil {
			return nil, err
		}
		item.ActedAt = actedAt.UTC().Format(time.RFC3339)
		logs = append(logs, item)
	}
	return logs, rows.Err()
}

// GetPendingTasksForUser returns pending approval workflow tasks matching the caller.
func (r ApprovalWorkflowRepository) GetPendingTasksForUser(ctx context.Context, workspaceID, userID uuid.UUID, page, limit int) (api.PaginatedTasksResponse, error) {
	db, err := r.db(ctx, workspaceID)
	if err != nil {
		return api.PaginatedTasksResponse{}, err
	}
	if page < 1 {
		page = 1
	}
	if limit < 1 || limit > 3000 {
		limit = 50
	}
	offset := (page - 1) * limit

	var roleIDs []uuid.UUID
	roleRows, _ := db.Query(ctx, `SELECT role_id FROM workspace_member_roles WHERE member_id IN (SELECT id FROM workspace_members WHERE user_id = $1 AND workspace_id = $2)`, userID, workspaceID)
	if roleRows != nil {
		for roleRows.Next() {
			var rid uuid.UUID
			if err := roleRows.Scan(&rid); err == nil {
				roleIDs = append(roleIDs, rid)
			}
		}
		roleRows.Close()
	}

	countQuery := `SELECT COUNT(*) FROM approval_workflow_tasks t JOIN approval_workflow_instances i ON t.instance_id = i.id WHERE i.workspace_id = $1 AND t.status = 'pending' AND ( (t.assignee_type = 'specific_user' AND t.assignee_target_id = $2) OR (t.assignee_type = 'role' AND t.assignee_target_id = ANY($3)) OR (t.assignee_type = 'department_manager') OR (t.assignee_type = 'permission') )`
	var total int64
	if err := db.QueryRow(ctx, countQuery, workspaceID, userID, roleIDs).Scan(&total); err != nil {
		return api.PaginatedTasksResponse{}, err
	}

	dataQuery := `SELECT t.id, t.instance_id, t.step_order, t.status, t.assignee_type, t.assignee_target_id, t.assignee_target_name, t.created_at, t.completed_at, t.completed_by FROM approval_workflow_tasks t JOIN approval_workflow_instances i ON t.instance_id = i.id WHERE i.workspace_id = $1 AND t.status = 'pending' AND ( (t.assignee_type = 'specific_user' AND t.assignee_target_id = $2) OR (t.assignee_type = 'role' AND t.assignee_target_id = ANY($3)) OR (t.assignee_type = 'department_manager') OR (t.assignee_type = 'permission') ) ORDER BY t.created_at DESC LIMIT $4 OFFSET $5`
	rows, err := db.Query(ctx, dataQuery, workspaceID, userID, roleIDs, limit, offset)
	if err != nil {
		return api.PaginatedTasksResponse{}, err
	}
	defer rows.Close()

	var data []api.ApprovalWorkflowTaskRecord
	for rows.Next() {
		var rec api.ApprovalWorkflowTaskRecord
		if err := rows.Scan(&rec.ID, &rec.InstanceID, &rec.StepOrder, &rec.Status, &rec.AssigneeType, &rec.AssigneeTargetID, &rec.AssigneeTargetName, &rec.CreatedAt, &rec.CompletedAt, &rec.CompletedBy); err != nil {
			return api.PaginatedTasksResponse{}, err
		}
		data = append(data, rec)
	}
	if data == nil {
		data = []api.ApprovalWorkflowTaskRecord{}
	}
	return api.PaginatedTasksResponse{
		Data:  data,
		Total: total,
		Page:  page,
		Limit: limit,
	}, nil
}
