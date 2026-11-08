package repository

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type LifecycleRepository struct {
	DB      DBTX
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r LifecycleRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("lifecycle repository: database is not configured")
	}
	return r.DB, nil
}

func (r LifecycleRepository) begin(ctx context.Context) (TxBeginner, error) {
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

func scanLifecycleTemplate(row interface{ Scan(...any) error }) (api.LifecycleChecklistTemplate, error) {
	var t api.LifecycleChecklistTemplate
	var created, updated time.Time
	if err := row.Scan(&t.ID, &t.Name, &t.Description, &t.ChecklistType, &t.IsDefault, &t.IsActive, &created, &updated); err != nil {
		return api.LifecycleChecklistTemplate{}, err
	}
	t.CreatedAt = created.UTC().Format(time.RFC3339)
	t.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return t, nil
}

const lifecycleTemplateColumns = `id, name, description, checklist_type, is_default, is_active, created_at, updated_at`

func scanTemplateItem(row interface{ Scan(...any) error }) (api.LifecycleChecklistTemplateItem, error) {
	var i api.LifecycleChecklistTemplateItem
	if err := row.Scan(&i.ID, &i.TemplateID, &i.Title, &i.Description, &i.Category, &i.AssigneeType, &i.AssigneeUserID, &i.IsRequired, &i.DueDaysOffset, &i.DisplayOrder); err != nil {
		return api.LifecycleChecklistTemplateItem{}, err
	}
	return i, nil
}

const lifecycleTemplateItemColumns = `id, template_id, title, description, category, assignee_type, assignee_user_id, is_required, due_days_offset, display_order`

func scanChecklist(row interface{ Scan(...any) error }) (api.LifecycleChecklistResponse, error) {
	var c api.LifecycleChecklistResponse
	var started, created, updated time.Time
	var completed *time.Time
	if err := row.Scan(
		&c.ID, &c.EmploymentID, &c.TemplateID, &c.ChecklistType, &c.Status,
		&c.ProgressPercent, &started, &completed, &c.CreatedBy, &created, &updated,
	); err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	c.StartedAt = started.UTC().Format(time.RFC3339)
	c.CreatedAt = created.UTC().Format(time.RFC3339)
	c.UpdatedAt = updated.UTC().Format(time.RFC3339)
	if completed != nil {
		formatted := completed.UTC().Format(time.RFC3339)
		c.CompletedAt = &formatted
	}
	c.Items = []api.LifecycleChecklistItem{}
	return c, nil
}

const lifecycleChecklistColumns = `id, employment_id, template_id, checklist_type, status, progress_percent, started_at, completed_at, created_by, created_at, updated_at`

func scanChecklistItem(row interface{ Scan(...any) error }) (api.LifecycleChecklistItem, error) {
	var i api.LifecycleChecklistItem
	var created, updated time.Time
	var completed *time.Time
	var due *time.Time
	if err := row.Scan(
		&i.ID, &i.ChecklistID, &i.TemplateItemID, &i.Title, &i.Description, &i.Category,
		&i.AssigneeUserID, &i.IsRequired, &due, &i.Status, &completed, &i.CompletedBy, &i.Notes,
		&i.DisplayOrder, &created, &updated,
	); err != nil {
		return api.LifecycleChecklistItem{}, err
	}
	if due != nil {
		formatted := due.Format("2006-01-02")
		i.DueDate = &formatted
	}
	if completed != nil {
		formatted := completed.UTC().Format(time.RFC3339)
		i.CompletedAt = &formatted
	}
	i.CreatedAt = created.UTC().Format(time.RFC3339)
	i.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return i, nil
}

const lifecycleChecklistItemColumns = `id, checklist_id, template_item_id, title, description, category, assignee_user_id, is_required, due_date, status, completed_at, completed_by, notes, display_order, created_at, updated_at`

func (r LifecycleRepository) CreateTemplate(ctx context.Context, req api.CreateLifecycleChecklistTemplateRequest) (api.LifecycleChecklistTemplate, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.LifecycleChecklistTemplate{}, err
	}
	if txBeginner != nil {
		var created api.LifecycleChecklistTemplate
		err := service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
			res, txErr := r.createTemplateTx(ctx, tx, req)
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
		return api.LifecycleChecklistTemplate{}, err
	}
	return r.createTemplateTx(ctx, db, req)
}

func (r LifecycleRepository) createTemplateTx(ctx context.Context, db DBTX, req api.CreateLifecycleChecklistTemplateRequest) (api.LifecycleChecklistTemplate, error) {
	if req.IsDefault {
		if _, err := db.Exec(ctx, `UPDATE lifecycle_checklist_templates
			SET is_default = FALSE, updated_at = NOW()
			WHERE checklist_type = $1 AND deleted_at IS NULL`, req.ChecklistType); err != nil {
			return api.LifecycleChecklistTemplate{}, err
		}
	}
	return scanLifecycleTemplate(db.QueryRow(ctx, `INSERT INTO lifecycle_checklist_templates (
		id, name, description, checklist_type, is_default, is_active, deleted_at, created_at, updated_at
	) VALUES ($1, $2, $3, $4, $5, TRUE, NULL, NOW(), NOW())
	RETURNING `+lifecycleTemplateColumns,
		uuid.New(), req.Name, req.Description, req.ChecklistType, req.IsDefault))
}

func (r LifecycleRepository) GetTemplate(ctx context.Context, id uuid.UUID) (api.LifecycleChecklistTemplate, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LifecycleChecklistTemplate{}, err
	}
	t, err := scanLifecycleTemplate(db.QueryRow(ctx, `SELECT `+lifecycleTemplateColumns+`
		FROM lifecycle_checklist_templates
		WHERE id = $1 AND deleted_at IS NULL`, id))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistTemplate{}, api.ErrTemplateNotFound
	}
	return t, err
}

func (r LifecycleRepository) ListTemplates(ctx context.Context, checklistType *string) ([]api.LifecycleChecklistTemplate, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	query := `SELECT ` + lifecycleTemplateColumns + `
		FROM lifecycle_checklist_templates
		WHERE is_active = TRUE AND deleted_at IS NULL`
	var args []any
	if checklistType != nil {
		query += " AND checklist_type = $1"
		args = append(args, *checklistType)
	}
	query += " ORDER BY name ASC"
	rows, err := db.Query(ctx, query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.LifecycleChecklistTemplate, 0)
	for rows.Next() {
		t, err := scanLifecycleTemplate(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, t)
	}
	return result, rows.Err()
}

func (r LifecycleRepository) UpdateTemplate(ctx context.Context, id uuid.UUID, req api.UpdateLifecycleChecklistTemplateRequest) (api.LifecycleChecklistTemplate, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.LifecycleChecklistTemplate{}, err
	}
	if txBeginner != nil {
		var updated api.LifecycleChecklistTemplate
		err := service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
			res, txErr := r.updateTemplateTx(ctx, tx, id, req)
			if txErr != nil {
				return txErr
			}
			updated = res
			return nil
		})
		return updated, err
	}
	db, err := r.db(ctx)
	if err != nil {
		return api.LifecycleChecklistTemplate{}, err
	}
	return r.updateTemplateTx(ctx, db, id, req)
}

func (r LifecycleRepository) updateTemplateTx(ctx context.Context, db DBTX, id uuid.UUID, req api.UpdateLifecycleChecklistTemplateRequest) (api.LifecycleChecklistTemplate, error) {
	existing, err := scanLifecycleTemplate(db.QueryRow(ctx, `SELECT `+lifecycleTemplateColumns+`
		FROM lifecycle_checklist_templates
		WHERE id = $1 AND deleted_at IS NULL`, id))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistTemplate{}, api.ErrTemplateNotFound
	}
	if err != nil {
		return api.LifecycleChecklistTemplate{}, err
	}
	if req.Name != nil {
		existing.Name = *req.Name
	}
	if req.Description != nil {
		existing.Description = req.Description
	}
	if req.IsDefault != nil {
		existing.IsDefault = *req.IsDefault
	}
	if req.IsActive != nil {
		existing.IsActive = *req.IsActive
	}
	if existing.IsDefault {
		if _, err := db.Exec(ctx, `UPDATE lifecycle_checklist_templates
			SET is_default = FALSE, updated_at = NOW()
			WHERE checklist_type = $1 AND deleted_at IS NULL AND id <> $2`,
			existing.ChecklistType, id); err != nil {
			return api.LifecycleChecklistTemplate{}, err
		}
	}
	updated, err := scanLifecycleTemplate(db.QueryRow(ctx, `UPDATE lifecycle_checklist_templates SET
		name = $1, description = $2, is_default = $3, is_active = $4, updated_at = NOW()
		WHERE id = $5
		RETURNING `+lifecycleTemplateColumns,
		existing.Name, existing.Description, existing.IsDefault, existing.IsActive, id))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistTemplate{}, api.ErrTemplateNotFound
	}
	return updated, err
}

func (r LifecycleRepository) DeleteTemplate(ctx context.Context, id uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE lifecycle_checklist_templates SET deleted_at = NOW(), updated_at = NOW()
		WHERE id = $1`, id)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrTemplateNotFound
	}
	return nil
}

func (r LifecycleRepository) CreateTemplateItem(ctx context.Context, templateID uuid.UUID, req api.CreateLifecycleChecklistTemplateItemRequest) (api.LifecycleChecklistTemplateItem, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LifecycleChecklistTemplateItem{}, err
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM lifecycle_checklist_templates WHERE id = $1)`, templateID).Scan(&exists); err != nil {
		return api.LifecycleChecklistTemplateItem{}, err
	}
	if !exists {
		return api.LifecycleChecklistTemplateItem{}, api.ErrTemplateNotFound
	}
	item, err := scanTemplateItem(db.QueryRow(ctx, `INSERT INTO lifecycle_checklist_template_items (
		id, template_id, title, description, category, assignee_type, assignee_user_id,
		is_required, due_days_offset, display_order, created_at
	) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, NOW())
	RETURNING `+lifecycleTemplateItemColumns,
		uuid.New(), templateID, req.Title, req.Description, req.Category, req.AssigneeType, req.AssigneeUserID, req.IsRequired, req.DueDaysOffset, req.DisplayOrder))
	if err != nil {
		return api.LifecycleChecklistTemplateItem{}, err
	}
	return item, nil
}

const templateItemJoin = ` FROM lifecycle_checklist_template_items i
	INNER JOIN lifecycle_checklist_templates t ON t.id = i.template_id`

func (r LifecycleRepository) GetTemplateItem(ctx context.Context, itemID uuid.UUID) (api.LifecycleChecklistTemplateItem, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LifecycleChecklistTemplateItem{}, err
	}
	item, err := scanTemplateItem(db.QueryRow(ctx, `SELECT `+prefixColumns("i", lifecycleTemplateItemColumns)+templateItemJoin+`
		WHERE i.id = $1`, itemID))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistTemplateItem{}, api.ErrTemplateItemNotFound
	}
	return item, err
}

func (r LifecycleRepository) ListTemplateItems(ctx context.Context, templateID uuid.UUID) ([]api.LifecycleChecklistTemplateItem, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT `+prefixColumns("i", lifecycleTemplateItemColumns)+templateItemJoin+`
		WHERE i.template_id = $1
		ORDER BY i.display_order ASC`, templateID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.LifecycleChecklistTemplateItem, 0)
	for rows.Next() {
		item, err := scanTemplateItem(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (r LifecycleRepository) UpdateTemplateItem(ctx context.Context, itemID uuid.UUID, req api.UpdateLifecycleChecklistTemplateItemRequest) (api.LifecycleChecklistTemplateItem, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LifecycleChecklistTemplateItem{}, err
	}
	existing, err := scanTemplateItem(db.QueryRow(ctx, `SELECT `+prefixColumns("i", lifecycleTemplateItemColumns)+templateItemJoin+`
		WHERE i.id = $1`, itemID))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistTemplateItem{}, api.ErrTemplateItemNotFound
	}
	if err != nil {
		return api.LifecycleChecklistTemplateItem{}, err
	}
	if req.Title != nil {
		existing.Title = *req.Title
	}
	if req.Description != nil {
		existing.Description = req.Description
	}
	if req.Category != nil {
		existing.Category = *req.Category
	}
	if req.AssigneeType != nil {
		existing.AssigneeType = *req.AssigneeType
	}
	if req.AssigneeIDPresent {
		if req.AssigneeUserID == nil {
			existing.AssigneeUserID = nil
		} else {
			existing.AssigneeUserID = *req.AssigneeUserID
		}
	}
	if req.IsRequired != nil {
		existing.IsRequired = *req.IsRequired
	}
	if req.DueDaysOffset != nil {
		existing.DueDaysOffset = *req.DueDaysOffset
	}
	if req.DisplayOrder != nil {
		existing.DisplayOrder = *req.DisplayOrder
	}
	updated, err := scanTemplateItem(db.QueryRow(ctx, `UPDATE lifecycle_checklist_template_items i SET
		title = $1, description = $2, category = $3, assignee_type = $4,
		assignee_user_id = $5, is_required = $6, due_days_offset = $7, display_order = $8
		WHERE i.id = $9
		RETURNING `+prefixColumns("i", lifecycleTemplateItemColumns),
		existing.Title, existing.Description, existing.Category, existing.AssigneeType,
		existing.AssigneeUserID, existing.IsRequired, existing.DueDaysOffset, existing.DisplayOrder,
		itemID))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistTemplateItem{}, api.ErrTemplateItemNotFound
	}
	return updated, err
}

func (r LifecycleRepository) DeleteTemplateItem(ctx context.Context, itemID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `DELETE FROM lifecycle_checklist_template_items WHERE id = $1`, itemID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrTemplateItemNotFound
	}
	return nil
}

type lifecycleEmployment struct {
	EmployeeID          uuid.UUID
	StartDate           string
	EndDate             *string
	ManagerEmploymentID *uuid.UUID
}

func (r LifecycleRepository) CreateChecklistFromTemplate(ctx context.Context, employmentID, templateID uuid.UUID, actorID *uuid.UUID, usersResolver func(context.Context, string) []uuid.UUID) (api.LifecycleChecklistResponse, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	if txBeginner != nil {
		var created api.LifecycleChecklistResponse
		err := service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
			res, txErr := r.createChecklistTx(ctx, tx, employmentID, templateID, actorID, usersResolver)
			if txErr != nil {
				return txErr
			}
			created = res
			return nil
		})
		return created, err
	}
	d, err := r.db(ctx)
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	return r.createChecklistTx(ctx, d, employmentID, templateID, actorID, usersResolver)
}

func (r LifecycleRepository) createChecklistTx(ctx context.Context, db DBTX, employmentID, templateID uuid.UUID, actorID *uuid.UUID, usersResolver func(context.Context, string) []uuid.UUID) (api.LifecycleChecklistResponse, error) {
	tmpl, err := scanLifecycleTemplate(db.QueryRow(ctx, `SELECT `+lifecycleTemplateColumns+`
		FROM lifecycle_checklist_templates
		WHERE id = $1 AND deleted_at IS NULL`, templateID))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistResponse{}, api.ErrTemplateNotFound
	}
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}

	templateItems, err := r.listTemplateItemsTx(ctx, db, templateID)
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}

	var employment lifecycleEmployment
	err = db.QueryRow(ctx, `SELECT employee_id, start_date::text, end_date::text, manager_employment_id
		FROM employments
		WHERE id = $1 AND deleted_at IS NULL`, employmentID).Scan(
		&employment.EmployeeID, &employment.StartDate, &employment.EndDate, &employment.ManagerEmploymentID)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistResponse{}, api.ErrEmploymentNotFound
	}
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}

	var employeeUserID *uuid.UUID
	var employeeName string
	err = db.QueryRow(ctx, `SELECT user_id, full_name FROM employees
		WHERE id = $1 AND deleted_at IS NULL`, employment.EmployeeID).Scan(&employeeUserID, &employeeName)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistResponse{}, api.ErrEmployeeNotFound
	}
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	_ = employeeName

	baseDate := employment.StartDate
	if tmpl.ChecklistType == "offboarding" {
		if employment.EndDate != nil {
			baseDate = *employment.EndDate
		} else {
			baseDate = time.Now().UTC().Format("2006-01-02")
		}
	}
	base, err := time.Parse("2006-01-02", baseDate)
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}

	hasDirectManager := false
	hasHRManager := false
	hasITAdmin := false
	for _, item := range templateItems {
		switch item.AssigneeType {
		case "direct_manager":
			hasDirectManager = true
		case "hr_manager":
			hasHRManager = true
		case "it_admin":
			hasITAdmin = true
		}
	}

	var directManagerUserID *uuid.UUID
	if hasDirectManager && employment.ManagerEmploymentID != nil {
		var managerEmployeeID uuid.UUID
		if err := db.QueryRow(ctx, `SELECT employee_id FROM employments WHERE id = $1`, *employment.ManagerEmploymentID).Scan(&managerEmployeeID); err == nil {
			var managerUserID *uuid.UUID
			if err := db.QueryRow(ctx, `SELECT user_id FROM employees WHERE id = $1`, managerEmployeeID).Scan(&managerUserID); err == nil {
				directManagerUserID = managerUserID
			}
		}
	}

	firstUser := func(permission string) *uuid.UUID {
		if usersResolver == nil {
			return nil
		}
		users := usersResolver(ctx, permission)
		if len(users) == 0 {
			return nil
		}
		id := users[0]
		return &id
	}

	var hrManagerUserID *uuid.UUID
	if hasHRManager {
		hrManagerUserID = firstUser("manage_onboarding")
		if hrManagerUserID == nil {
			hrManagerUserID = firstUser("admin")
		}
	}
	var itAdminUserID *uuid.UUID
	if hasITAdmin {
		itAdminUserID = firstUser("manage_it")
		if itAdminUserID == nil {
			itAdminUserID = firstUser("admin")
		}
	}

	checklistID := uuid.New()
	checklist, err := scanChecklist(db.QueryRow(ctx, `INSERT INTO lifecycle_checklists (
		id, employment_id, template_id, checklist_type, status, progress_percent,
		started_at, completed_at, created_by, created_at, updated_at
	) VALUES ($1, $2, $3, $4, 'in_progress', 0, NOW(), NULL, $5, NOW(), NOW())
	RETURNING `+lifecycleChecklistColumns,
		checklistID, employmentID, templateID, tmpl.ChecklistType, actorID))
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}

	items := make([]api.LifecycleChecklistItem, 0, len(templateItems))
	for _, item := range templateItems {
		var assignee *uuid.UUID
		switch item.AssigneeType {
		case "specific_user":
			assignee = item.AssigneeUserID
		case "employee":
			assignee = employeeUserID
		case "direct_manager":
			assignee = directManagerUserID
		case "hr_manager":
			assignee = hrManagerUserID
		case "it_admin":
			assignee = itAdminUserID
		}
		dueDate := base.AddDate(0, 0, item.DueDaysOffset).Format("2006-01-02")
		created, err := scanChecklistItem(db.QueryRow(ctx, `INSERT INTO lifecycle_checklist_items (
			id, checklist_id, template_item_id, title, description, category, assignee_user_id,
			is_required, due_date, status, completed_at, completed_by, notes, display_order,
			created_at, updated_at
		) VALUES ($1, $2, $3, $4, $5, $6, $7,
			$8, $9::date, 'pending', NULL, NULL, NULL, $10,
			NOW(), NOW())
		RETURNING `+lifecycleChecklistItemColumns,
			uuid.New(), checklistID, item.ID, item.Title, item.Description, item.Category, assignee,
			item.IsRequired, dueDate, item.DisplayOrder))
		if err != nil {
			return api.LifecycleChecklistResponse{}, err
		}
		items = append(items, created)
	}

	if len(items) == 0 {
		checklist, err = scanChecklist(db.QueryRow(ctx, `UPDATE lifecycle_checklists SET
			status = 'completed', progress_percent = 100, completed_at = NOW(), updated_at = NOW()
			WHERE id = $1
			RETURNING `+lifecycleChecklistColumns, checklistID))
		if err != nil {
			return api.LifecycleChecklistResponse{}, err
		}
	}
	checklist.Items = items
	return checklist, nil
}

func (r LifecycleRepository) listTemplateItemsTx(ctx context.Context, db DBTX, templateID uuid.UUID) ([]api.LifecycleChecklistTemplateItem, error) {
	rows, err := db.Query(ctx, `SELECT `+prefixColumns("i", lifecycleTemplateItemColumns)+templateItemJoin+`
		WHERE i.template_id = $1
		ORDER BY i.display_order ASC`, templateID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.LifecycleChecklistTemplateItem, 0)
	for rows.Next() {
		item, err := scanTemplateItem(rows)
		if err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}

func (r LifecycleRepository) getChecklist(ctx context.Context, db DBTX, checklistID uuid.UUID) (api.LifecycleChecklistResponse, error) {
	checklist, err := scanChecklist(db.QueryRow(ctx, `SELECT `+lifecycleChecklistColumns+`
		FROM lifecycle_checklists WHERE id = $1`, checklistID))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistResponse{}, api.ErrChecklistNotFound
	}
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	rows, err := db.Query(ctx, `SELECT `+lifecycleChecklistItemColumns+`
		FROM lifecycle_checklist_items
		WHERE checklist_id = $1
		ORDER BY display_order ASC`, checklistID)
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	defer rows.Close()
	for rows.Next() {
		item, err := scanChecklistItem(rows)
		if err != nil {
			return api.LifecycleChecklistResponse{}, err
		}
		checklist.Items = append(checklist.Items, item)
	}
	if err := rows.Err(); err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	return checklist, nil
}

func (r LifecycleRepository) GetChecklist(ctx context.Context, checklistID uuid.UUID) (api.LifecycleChecklistResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	return r.getChecklist(ctx, db, checklistID)
}

func (r LifecycleRepository) ListChecklistsForEmployment(ctx context.Context, employmentID uuid.UUID) ([]api.LifecycleChecklistResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT `+lifecycleChecklistColumns+`
		FROM lifecycle_checklists
		WHERE employment_id = $1
		ORDER BY created_at DESC`, employmentID)
	if err != nil {
		return nil, err
	}
	checklists := make([]api.LifecycleChecklistResponse, 0)
	for rows.Next() {
		c, err := scanChecklist(rows)
		if err != nil {
			rows.Close()
			return nil, err
		}
		checklists = append(checklists, c)
	}
	rows.Close()
	if err := rows.Err(); err != nil {
		return nil, err
	}
	if len(checklists) == 0 {
		return checklists, nil
	}

	itemRows, err := db.Query(ctx, `SELECT `+lifecycleChecklistItemColumns+`
		FROM lifecycle_checklist_items i
		INNER JOIN lifecycle_checklists c ON c.id = i.checklist_id
		WHERE c.employment_id = $1
		ORDER BY i.display_order ASC`, employmentID)
	defer itemRows.Close()
	index := make(map[uuid.UUID]int, len(checklists))
	for i := range checklists {
		index[checklists[i].ID] = i
	}
	for itemRows.Next() {
		item, err := scanChecklistItem(itemRows)
		if err != nil {
			return nil, err
		}
		if i, ok := index[item.ChecklistID]; ok {
			checklists[i].Items = append(checklists[i].Items, item)
		}
	}
	return checklists, itemRows.Err()
}
func (r LifecycleRepository) ListChecklists(ctx context.Context) ([]api.LifecycleInstanceSummary, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	query := `SELECT
		c.id, c.employment_id, em.employee_id, e.full_name, e.employee_id_number, d.name,
		c.template_id, t.name, c.checklist_type, c.status, c.progress_percent,
		COALESCE(item_counts.total_items, 0), COALESCE(item_counts.completed_items, 0),
		c.started_at, c.completed_at, c.created_at, c.updated_at
	FROM lifecycle_checklists c
	INNER JOIN employments em ON em.id = c.employment_id
	INNER JOIN employees e ON e.id = em.employee_id
	LEFT JOIN departments d ON d.id = em.department_id
	LEFT JOIN lifecycle_checklist_templates t ON t.id = c.template_id
	LEFT JOIN (
		SELECT checklist_id, COUNT(*)::int AS total_items,
			COUNT(*) FILTER (WHERE status = 'completed')::int AS completed_items
		FROM lifecycle_checklist_items
		GROUP BY checklist_id
	) item_counts ON item_counts.checklist_id = c.id
	ORDER BY c.created_at DESC`
	rows, err := db.Query(ctx, query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	summaries := make([]api.LifecycleInstanceSummary, 0)
	for rows.Next() {
		var s api.LifecycleInstanceSummary
		var started, created, updated time.Time
		var completed *time.Time
		if err := rows.Scan(
			&s.ID, &s.EmploymentID, &s.EmployeeID, &s.EmployeeName, &s.EmployeeCode,
			&s.DepartmentName, &s.TemplateID, &s.TemplateName, &s.ChecklistType,
			&s.Status, &s.ProgressPercent, &s.TotalItems, &s.CompletedItems,
			&started, &completed, &created, &updated,
		); err != nil {
			return nil, err
		}
		s.StartedAt = started.UTC().Format(time.RFC3339)
		s.CreatedAt = created.UTC().Format(time.RFC3339)
		s.UpdatedAt = updated.UTC().Format(time.RFC3339)
		if completed != nil {
			formatted := completed.UTC().Format(time.RFC3339)
			s.CompletedAt = &formatted
		}
		summaries = append(summaries, s)
	}
	return summaries, rows.Err()
}

func (r LifecycleRepository) GetItemAssigneeAndChecklist(ctx context.Context, itemID uuid.UUID) (*uuid.UUID, uuid.UUID, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, uuid.Nil, err
	}
	var assignee *uuid.UUID
	var checklistID uuid.UUID
	err = db.QueryRow(ctx, `SELECT i.assignee_user_id, i.checklist_id
		FROM lifecycle_checklist_items i
		WHERE i.id = $1`, itemID).Scan(&assignee, &checklistID)
	if errors.Is(err, pgx.ErrNoRows) {
		return nil, uuid.Nil, api.ErrChecklistItemNotFound
	}
	return assignee, checklistID, err
}

func (r LifecycleRepository) UpdateChecklistItemStatus(ctx context.Context, itemID uuid.UUID, status string, notes *string, completedBy *uuid.UUID) (api.LifecycleChecklistResponse, error) {
	txBeginner, err := r.begin(ctx)
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	if txBeginner != nil {
		var updated api.LifecycleChecklistResponse
		err := service.WithTx(ctx, txBeginner, func(ctx context.Context, tx pgx.Tx) error {
			res, txErr := r.updateItemStatusTx(ctx, tx, itemID, status, notes, completedBy)
			if txErr != nil {
				return txErr
			}
			updated = res
			return nil
		})
		return updated, err
	}
	d, err := r.db(ctx)
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	return r.updateItemStatusTx(ctx, d, itemID, status, notes, completedBy)
}



func (r LifecycleRepository) updateItemStatusTx(ctx context.Context, db DBTX, itemID uuid.UUID, status string, notes *string, completedBy *uuid.UUID) (api.LifecycleChecklistResponse, error) {
	var completedAt *time.Time
	if status == "completed" {
		now := time.Now().UTC()
		completedAt = &now
	}
	var checklistID uuid.UUID
	err := db.QueryRow(ctx, `UPDATE lifecycle_checklist_items
		SET status = $1, completed_at = $2, completed_by = $3, notes = $4, updated_at = NOW()
		WHERE id = $5
		RETURNING checklist_id`, status, completedAt, completedBy, notes, itemID).Scan(&checklistID)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.LifecycleChecklistResponse{}, api.ErrChecklistItemNotFound
	}
	if err != nil {
		return api.LifecycleChecklistResponse{}, err
	}

	var total, done int
	if err := db.QueryRow(ctx, `SELECT COUNT(*), COUNT(*) FILTER (WHERE status IN ('completed', 'skipped'))
		FROM lifecycle_checklist_items WHERE checklist_id = $1`, checklistID).Scan(&total, &done); err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	progress := 100
	if total > 0 {
		progress = int(float64(done)/float64(total)*100 + 0.5)
	}
	newStatus := "in_progress"
	var completed *time.Time
	if progress == 100 {
		newStatus = "completed"
		now := time.Now().UTC()
		completed = &now
	}
	if _, err := db.Exec(ctx, `UPDATE lifecycle_checklists SET
		status = $1, progress_percent = $2, completed_at = $3, updated_at = NOW()
		WHERE id = $4`, newStatus, progress, completed, checklistID); err != nil {
		return api.LifecycleChecklistResponse{}, err
	}
	return r.getChecklist(ctx, db, checklistID)
}

func (r LifecycleRepository) GetPendingTasksForUser(ctx context.Context, userID uuid.UUID) ([]api.PendingTaskResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		i.id, i.checklist_id, e.full_name, c.checklist_type, i.title, i.description,
		i.category, i.due_date::text, i.is_required
	FROM lifecycle_checklist_items i
	INNER JOIN lifecycle_checklists c ON c.id = i.checklist_id
	INNER JOIN employments em ON em.id = c.employment_id
	INNER JOIN employees e ON e.id = em.employee_id
	WHERE i.assignee_user_id = $1
	  AND i.status = 'pending'
	ORDER BY i.due_date ASC NULLS LAST`, userID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.PendingTaskResponse, 0)
	for rows.Next() {
		var task api.PendingTaskResponse
		if err := rows.Scan(
			&task.ID, &task.ChecklistID, &task.EmployeeName, &task.ChecklistType,
			&task.Title, &task.Description, &task.Category, &task.DueDate, &task.IsRequired,
		); err != nil {
			return nil, err
		}
		result = append(result, task)
	}
	return result, rows.Err()
}

func prefixColumns(prefix, columns string) string {
	return replaceColumnList(columns, prefix)
}

func replaceColumnList(columns, prefix string) string {
	out := ""
	start := 0
	for i := 0; i <= len(columns); i++ {
		if i == len(columns) || columns[i] == ',' {
			part := columns[start:i]
			for len(part) > 0 && part[0] == ' ' {
				part = part[1:]
			}
			if part != "" {
				if out != "" {
					out += ", "
				}
				out += fmt.Sprintf("%s.%s", prefix, part)
			}
			start = i + 1
		}
	}
	return out
}
