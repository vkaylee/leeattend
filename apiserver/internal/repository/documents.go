package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type DocumentRepository struct {
	DB DBTX
}

func (r DocumentRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("document repository: database is not configured")
	}
	return r.DB, nil
}

func (r DocumentRepository) ListFolders(ctx context.Context) ([]api.DocumentFolderDto, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, name, parent_folder_id FROM document_folders ORDER BY name ASC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.DocumentFolderDto, 0)
	for rows.Next() {
		var f api.DocumentFolderDto
		if err := rows.Scan(&f.ID, &f.Name, &f.ParentFolderID); err != nil {
			return nil, err
		}
		result = append(result, f)
	}
	return result, rows.Err()
}

func (r DocumentRepository) CreateFolder(ctx context.Context, req api.CreateFolderRequest) (api.DocumentFolderDto, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.DocumentFolderDto{}, err
	}
	if req.ParentFolderID != nil {
		var exists int
		err := db.QueryRow(ctx, `SELECT 1 FROM document_folders WHERE id = $1`, *req.ParentFolderID).Scan(&exists)
		if err != nil {
			if errors.Is(err, pgx.ErrNoRows) {
				return api.DocumentFolderDto{}, api.ErrInvalidParent
			}
			return api.DocumentFolderDto{}, err
		}
	}
	var f api.DocumentFolderDto
	folderID := uuid.New()
	err = db.QueryRow(ctx, `INSERT INTO document_folders (
		id, name, parent_folder_id, created_at, updated_at
	) VALUES ($1, $2, $3, NOW(), NOW())
	RETURNING id, name, parent_folder_id`, folderID, req.Name, req.ParentFolderID).Scan(&f.ID, &f.Name, &f.ParentFolderID)
	if err != nil {
		return api.DocumentFolderDto{}, err
	}
	return f, nil
}

func (r DocumentRepository) GetFolder(ctx context.Context, folderID uuid.UUID) (api.DocumentFolderDto, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.DocumentFolderDto{}, err
	}
	var f api.DocumentFolderDto
	err = db.QueryRow(ctx, `SELECT id, name, parent_folder_id FROM document_folders WHERE id = $1`, folderID).Scan(&f.ID, &f.Name, &f.ParentFolderID)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.DocumentFolderDto{}, api.ErrFolderNotFound
		}
		return api.DocumentFolderDto{}, err
	}
	return f, nil
}

func (r DocumentRepository) RenameFolder(ctx context.Context, folderID uuid.UUID, name string) (api.DocumentFolderDto, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.DocumentFolderDto{}, err
	}
	var f api.DocumentFolderDto
	err = db.QueryRow(ctx, `UPDATE document_folders SET name = $1, updated_at = NOW() WHERE id = $2 RETURNING id, name, parent_folder_id`, name, folderID).Scan(&f.ID, &f.Name, &f.ParentFolderID)
	if err != nil {
		if errors.Is(err, pgx.ErrNoRows) {
			return api.DocumentFolderDto{}, api.ErrFolderNotFound
		}
		return api.DocumentFolderDto{}, err
	}
	return f, nil
}

func (r DocumentRepository) DeleteFolder(ctx context.Context, folderID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `DELETE FROM document_folders WHERE id = $1`, folderID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrFolderNotFound
	}
	return nil
}

func (r DocumentRepository) DistributeDocument(ctx context.Context, req api.DistributeDocumentRequest) (int, error) {
	db, err := r.db(ctx)
	if err != nil {
		return 0, err
	}
	count := 0
	for _, empID := range req.TargetEmployeeIDs {
		_, err := db.Exec(ctx, `INSERT INTO file_acknowledgements (
			id, document_id, employee_id, status, created_at, updated_at
		) VALUES (gen_random_uuid(), $1, $2, 'pending', NOW(), NOW())
		ON CONFLICT DO NOTHING`, req.DocumentID, empID)
		if err == nil {
			count++
		}
	}
	return count, nil
}

func (r DocumentRepository) GetTemplateVariables(ctx context.Context, employeeID uuid.UUID) (map[string]string, error) {
	vars := make(map[string]string)
	vars["COMPANY_NAME"] = "LeeAttend Corp"
	if r.DB != nil {
		var orgName string
		if err := r.DB.QueryRow(ctx, `SELECT name FROM organization_settings LIMIT 1`).Scan(&orgName); err == nil && orgName != "" {
			vars["COMPANY_NAME"] = orgName
		}
	}

	vars["DOCUMENT_DATE"] = time.Now().UTC().Format("02/01/2006")

	db, err := r.db(ctx)
	if err != nil {
		return vars, nil
	}

	var fullName, empNum string
	if err := db.QueryRow(ctx, `SELECT full_name, employee_id_number FROM employees WHERE id = $1 AND deleted_at IS NULL`, employeeID).Scan(&fullName, &empNum); err == nil {
		vars["EMPLOYEE_NAME"] = fullName
		vars["EMPLOYEE_ID_NUMBER"] = empNum
	}

	var empID uuid.UUID
	var jobTitle *string
	var startDate time.Time
	if err := db.QueryRow(ctx, `SELECT id, job_title, start_date FROM employments WHERE employee_id = $1 AND deleted_at IS NULL ORDER BY (CASE WHEN status = 'active' THEN 0 ELSE 1 END), start_date DESC LIMIT 1`, employeeID).Scan(&empID, &jobTitle, &startDate); err == nil {
		vars["HIRE_DATE"] = startDate.Format("02/01/2006")
		if jobTitle != nil && *jobTitle != "" {
			vars["JOB_TITLE"] = *jobTitle
		}

		var contractNum *string
		if err := db.QueryRow(ctx, `SELECT contract_number FROM employee_contracts WHERE employment_id = $1 AND deleted_at IS NULL ORDER BY created_at ASC LIMIT 1`, empID).Scan(&contractNum); err == nil {
			if contractNum != nil && *contractNum != "" {
				vars["CONTRACT_NUMBER"] = *contractNum
			}
		}
	}

	return vars, nil
}
