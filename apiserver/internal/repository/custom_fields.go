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

type CustomFieldRepository struct {
	DB DBTX
}

func (r CustomFieldRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("custom field repository: database is not configured")
	}
	return r.DB, nil
}

func (r CustomFieldRepository) CreateDefinition(ctx context.Context, record api.CustomFieldDefinitionRecord) (api.CustomFieldDefinitionRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.CustomFieldDefinitionRecord{}, err
	}
	_, err = db.Exec(ctx, `INSERT INTO custom_field_definitions (
		id, workspace_id, entity_type, field_key, field_label, field_type,
		options, default_value, is_required, is_visible_to_employee, is_filterable,
		is_exportable, display_order, section_name, validation_rules, created_at, updated_at
	) VALUES (
		$1, $2, $3, $4, $5, $6,
		$7, $8, $9, $10, $11,
		$12, $13, $14, $15, NOW(), NOW()
	)`, record.ID, record.WorkspaceID, record.EntityType, record.FieldKey, record.FieldLabel, record.FieldType,
		record.Options, record.DefaultValue, record.IsRequired, record.IsVisibleToEmployee, record.IsFilterable,
		record.IsExportable, record.DisplayOrder, record.SectionName, record.ValidationRules)
	if err != nil {
		if strings.Contains(err.Error(), "unique") || strings.Contains(err.Error(), "duplicate key") {
			return api.CustomFieldDefinitionRecord{}, api.ErrCustomFieldConflict
		}
		return api.CustomFieldDefinitionRecord{}, err
	}
	return r.GetDefinition(ctx, record.WorkspaceID, record.ID)
}

func (r CustomFieldRepository) GetDefinition(ctx context.Context, wsID, id uuid.UUID) (api.CustomFieldDefinitionRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.CustomFieldDefinitionRecord{}, err
	}
	var rec api.CustomFieldDefinitionRecord
	var created, updated time.Time
	err = db.QueryRow(ctx, `SELECT
		id, workspace_id, entity_type, field_key, field_label, field_type,
		options, default_value, is_required, is_visible_to_employee, is_filterable,
		is_exportable, display_order, section_name, validation_rules, created_at, updated_at
	FROM custom_field_definitions
	WHERE workspace_id = $1 AND id = $2 AND deleted_at IS NULL`, wsID, id).Scan(
		&rec.ID, &rec.WorkspaceID, &rec.EntityType, &rec.FieldKey, &rec.FieldLabel, &rec.FieldType,
		&rec.Options, &rec.DefaultValue, &rec.IsRequired, &rec.IsVisibleToEmployee, &rec.IsFilterable,
		&rec.IsExportable, &rec.DisplayOrder, &rec.SectionName, &rec.ValidationRules, &created, &updated,
	)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.CustomFieldDefinitionRecord{}, api.ErrCustomFieldNotFound
		}
		return api.CustomFieldDefinitionRecord{}, err
	}
	rec.CreatedAt = created.UTC().Format(time.RFC3339)
	rec.UpdatedAt = updated.UTC().Format(time.RFC3339)
	return rec, nil
}

func (r CustomFieldRepository) ListDefinitions(ctx context.Context, wsID uuid.UUID, entityType *string) ([]api.CustomFieldDefinitionRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	query := `SELECT
		id, workspace_id, entity_type, field_key, field_label, field_type,
		options, default_value, is_required, is_visible_to_employee, is_filterable,
		is_exportable, display_order, section_name, validation_rules, created_at, updated_at
	FROM custom_field_definitions
	WHERE workspace_id = $1 AND deleted_at IS NULL`
	var args []any
	args = append(args, wsID)
	if entityType != nil && *entityType != "" {
		query += " AND entity_type = $2"
		args = append(args, *entityType)
	}
	query += " ORDER BY display_order ASC, field_key ASC"
	rows, err := db.Query(ctx, query, args...)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.CustomFieldDefinitionRecord, 0)
	for rows.Next() {
		var rec api.CustomFieldDefinitionRecord
		var created, updated time.Time
		if err := rows.Scan(
			&rec.ID, &rec.WorkspaceID, &rec.EntityType, &rec.FieldKey, &rec.FieldLabel, &rec.FieldType,
			&rec.Options, &rec.DefaultValue, &rec.IsRequired, &rec.IsVisibleToEmployee, &rec.IsFilterable,
			&rec.IsExportable, &rec.DisplayOrder, &rec.SectionName, &rec.ValidationRules, &created, &updated,
		); err != nil {
			return nil, err
		}
		rec.CreatedAt = created.UTC().Format(time.RFC3339)
		rec.UpdatedAt = updated.UTC().Format(time.RFC3339)
		result = append(result, rec)
	}
	return result, rows.Err()
}

func (r CustomFieldRepository) UpdateDefinition(ctx context.Context, wsID, id uuid.UUID, req api.UpdateCustomFieldDefinitionRequest) (api.CustomFieldDefinitionRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.CustomFieldDefinitionRecord{}, err
	}
	existing, err := r.GetDefinition(ctx, wsID, id)
	if err != nil {
		return api.CustomFieldDefinitionRecord{}, err
	}
	if req.FieldLabel != nil {
		existing.FieldLabel = *req.FieldLabel
	}
	if req.Options != nil {
		existing.Options = *req.Options
	}
	if req.DefaultValue != nil {
		existing.DefaultValue = req.DefaultValue
	}
	if req.IsRequired != nil {
		existing.IsRequired = *req.IsRequired
	}
	if req.IsVisibleToEmployee != nil {
		existing.IsVisibleToEmployee = *req.IsVisibleToEmployee
	}
	if req.IsFilterable != nil {
		existing.IsFilterable = *req.IsFilterable
	}
	if req.IsExportable != nil {
		existing.IsExportable = *req.IsExportable
	}
	if req.DisplayOrder != nil {
		existing.DisplayOrder = *req.DisplayOrder
	}
	if req.SectionName != nil {
		existing.SectionName = req.SectionName
	}
	if req.ValidationRules != nil {
		existing.ValidationRules = *req.ValidationRules
	}
	_, err = db.Exec(ctx, `UPDATE custom_field_definitions SET
		field_label = $1, options = $2, default_value = $3, is_required = $4,
		is_visible_to_employee = $5, is_filterable = $6, is_exportable = $7,
		display_order = $8, section_name = $9, validation_rules = $10, updated_at = NOW()
	WHERE workspace_id = $11 AND id = $12 AND deleted_at IS NULL`,
		existing.FieldLabel, existing.Options, existing.DefaultValue, existing.IsRequired,
		existing.IsVisibleToEmployee, existing.IsFilterable, existing.IsExportable,
		existing.DisplayOrder, existing.SectionName, existing.ValidationRules, wsID, id)
	if err != nil {
		return api.CustomFieldDefinitionRecord{}, err
	}
	return r.GetDefinition(ctx, wsID, id)
}

func (r CustomFieldRepository) DeleteDefinition(ctx context.Context, wsID, id uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `UPDATE custom_field_definitions SET deleted_at = NOW(), updated_at = NOW() WHERE workspace_id = $1 AND id = $2 AND deleted_at IS NULL`, wsID, id)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrCustomFieldNotFound
	}
	return nil
}

func (r CustomFieldRepository) GetValues(ctx context.Context, wsID, entityID uuid.UUID) ([]api.CustomFieldValueResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT
		d.id, d.field_key, d.field_label, d.field_type,
		v.value_text, v.value_number::text, v.value_date::text, v.value_bool, v.value_json
	FROM custom_field_definitions d
	LEFT JOIN custom_field_values v ON v.definition_id = d.id AND v.entity_id = $2
	WHERE d.workspace_id = $1 AND d.deleted_at IS NULL
	ORDER BY d.display_order ASC, d.field_key ASC`, wsID, entityID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.CustomFieldValueResponse, 0)
	for rows.Next() {
		var defID uuid.UUID
		var key, label, fType string
		var valText, valNum, valDate *string
		var valBool *bool
		var valJSON []byte
		if err := rows.Scan(&defID, &key, &label, &fType, &valText, &valNum, &valDate, &valBool, &valJSON); err != nil {
			return nil, err
		}
		var finalVal json.RawMessage
		if valText != nil {
			finalVal, _ = json.Marshal(*valText)
		} else if valNum != nil {
			finalVal = json.RawMessage(*valNum)
		} else if valDate != nil {
			finalVal, _ = json.Marshal(*valDate)
		} else if valBool != nil {
			finalVal, _ = json.Marshal(*valBool)
		} else if len(valJSON) > 0 {
			finalVal = valJSON
		}
		result = append(result, api.CustomFieldValueResponse{
			DefinitionID: defID,
			FieldKey:     key,
			FieldLabel:   label,
			FieldType:    fType,
			Value:        finalVal,
		})
	}
	return result, rows.Err()
}

func (r CustomFieldRepository) SetValues(ctx context.Context, wsID, entityID uuid.UUID, entityType string, values []api.SetCustomFieldValueItem) ([]api.CustomFieldValueResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	for _, item := range values {
		var fType string
		err := db.QueryRow(ctx, `SELECT field_type FROM custom_field_definitions WHERE workspace_id = $1 AND id = $2 AND deleted_at IS NULL`, wsID, item.DefinitionID).Scan(&fType)
		if err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return nil, fmt.Errorf("%w: definition %s", api.ErrCustomFieldNotFound, item.DefinitionID)
			}
			return nil, err
		}
		var valText, valNum, valDate *string
		var valBool *bool
		var valJSON []byte
		raw := string(item.Value)
		switch fType {
		case "number":
			trimmed := strings.Trim(raw, `"`)
			valNum = &trimmed
		case "boolean":
			b := raw == "true"
			valBool = &b
		case "date":
			trimmed := strings.Trim(raw, `"`)
			valDate = &trimmed
		case "select", "multiselect":
			valJSON = item.Value
		default:
			trimmed := strings.Trim(raw, `"`)
			valText = &trimmed
		}
		_, err = db.Exec(ctx, `INSERT INTO custom_field_values (
			id, definition_id, entity_id, value_text, value_number, value_date, value_bool, value_json, created_at, updated_at
		) VALUES (
			$1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW()
		) ON CONFLICT (definition_id, entity_id) DO UPDATE SET
			value_text = EXCLUDED.value_text,
			value_number = EXCLUDED.value_number,
			value_date = EXCLUDED.value_date,
			value_bool = EXCLUDED.value_bool,
			value_json = EXCLUDED.value_json,
			updated_at = NOW()`,
			uuid.New(), item.DefinitionID, entityID, valText, valNum, valDate, valBool, valJSON)
		if err != nil {
			return nil, err
		}
	}
	return r.GetValues(ctx, wsID, entityID)
}
