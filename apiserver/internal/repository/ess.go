package repository

import (
	"context"
	"errors"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type ESSRepository struct {
	DB      DBTX
	BeginTx func(context.Context, uuid.UUID) (TxBeginner, error)
}

func (r ESSRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("ess repository: database is not configured")
	}
	return r.DB, nil
}

func (r ESSRepository) begin(ctx context.Context, workspaceID uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, workspaceID)
	}
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("ess repository: tenant database does not support transactions")
	}
	return beginner, nil
}

func (r ESSRepository) ResolveEmployeeForUser(ctx context.Context, workspaceID, userID uuid.UUID) (uuid.UUID, error) {
	db, err := r.db(ctx)
	if err != nil {
		return uuid.Nil, err
	}
	var empID uuid.UUID
	err = db.QueryRow(ctx, `SELECT id FROM employees WHERE user_id = $1 AND workspace_id = $2 AND deleted_at IS NULL LIMIT 1`, userID, workspaceID).Scan(&empID)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, api.ErrESSNoEmployeeRecord
	}
	return empID, err
}

func (r ESSRepository) GetMyProfile(ctx context.Context, workspaceID, employeeID uuid.UUID) (api.MyProfileResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.MyProfileResponse{}, err
	}
	var emp api.EmployeeRecord
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, `SELECT id, workspace_id, user_id, full_name, employee_id_number, tags, created_at, updated_at FROM employees WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL`, employeeID, workspaceID).Scan(
		&emp.ID, &emp.WorkspaceID, &emp.UserID, &emp.FullName, &emp.EmployeeIDNumber, &emp.Tags, &createdAt, &updatedAt,
	)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.MyProfileResponse{}, api.ErrESSNotFound
	}
	if err != nil {
		return api.MyProfileResponse{}, err
	}
	emp.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	emp.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)

	var profile *api.EmployeeProfileRecord
	var prof api.EmployeeProfileRecord
	var profCreated, profUpdated time.Time
	err = db.QueryRow(ctx, `SELECT id, employee_id, phone, address, date_of_birth::text, gender, national_id, bank_name, bank_account_number, bank_branch, avatar_file_id, created_at, updated_at FROM employee_profiles WHERE employee_id = $1`, employeeID).Scan(
		&prof.ID, &prof.EmployeeID, &prof.Phone, &prof.Address, &prof.DateOfBirth, &prof.Gender, &prof.NationalID, &prof.BankName, &prof.BankAccountNumber, &prof.BankBranch, &prof.AvatarFileID, &profCreated, &profUpdated,
	)
	if err == nil {
		prof.CreatedAt = profCreated.UTC().Format(time.RFC3339)
		prof.UpdatedAt = profUpdated.UTC().Format(time.RFC3339)
		profile = &prof
	} else if !errors.Is(err, pgx.ErrNoRows) {
		return api.MyProfileResponse{}, err
	}

	contacts, err := r.GetEmergencyContacts(ctx, workspaceID, employeeID)
	if err != nil {
		return api.MyProfileResponse{}, err
	}

	return api.MyProfileResponse{
		Employee:          emp,
		Profile:           profile,
		EmergencyContacts: contacts,
	}, nil
}

func (r ESSRepository) UpsertProfile(ctx context.Context, workspaceID, employeeID uuid.UUID, req api.UpdateEmployeeProfileRequest) (api.EmployeeProfileRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployeeProfileRecord{}, err
	}
	var prof api.EmployeeProfileRecord
	var profCreated, profUpdated time.Time
	err = db.QueryRow(ctx, `INSERT INTO employee_profiles (id, employee_id, phone, address, date_of_birth, gender, national_id, bank_name, bank_account_number, bank_branch, avatar_file_id, created_at, updated_at) VALUES ($1, $2, $3, $4, $5::date, $6, $7, $8, $9, $10, $11, now(), now()) ON CONFLICT (employee_id) DO UPDATE SET phone = EXCLUDED.phone, address = EXCLUDED.address, date_of_birth = EXCLUDED.date_of_birth, gender = EXCLUDED.gender, national_id = EXCLUDED.national_id, bank_name = EXCLUDED.bank_name, bank_account_number = EXCLUDED.bank_account_number, bank_branch = EXCLUDED.bank_branch, avatar_file_id = EXCLUDED.avatar_file_id, updated_at = now() RETURNING id, employee_id, phone, address, date_of_birth::text, gender, national_id, bank_name, bank_account_number, bank_branch, avatar_file_id, created_at, updated_at`, uuid.New(), employeeID, req.Phone, req.Address, req.DateOfBirth, req.Gender, req.NationalID, req.BankName, req.BankAccountNumber, req.BankBranch, req.AvatarFileID).Scan(
		&prof.ID, &prof.EmployeeID, &prof.Phone, &prof.Address, &prof.DateOfBirth, &prof.Gender, &prof.NationalID, &prof.BankName, &prof.BankAccountNumber, &prof.BankBranch, &prof.AvatarFileID, &profCreated, &profUpdated,
	)
	if err != nil {
		return api.EmployeeProfileRecord{}, err
	}
	prof.CreatedAt = profCreated.UTC().Format(time.RFC3339)
	prof.UpdatedAt = profUpdated.UTC().Format(time.RFC3339)
	return prof, nil
}

func (r ESSRepository) GetEmergencyContacts(ctx context.Context, workspaceID, employeeID uuid.UUID) ([]api.EmployeeEmergencyContactRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, employee_id, contact_name, relationship, phone, is_primary, created_at FROM employee_emergency_contacts WHERE employee_id = $1 ORDER BY is_primary DESC, created_at ASC`, employeeID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	contacts := make([]api.EmployeeEmergencyContactRecord, 0)
	for rows.Next() {
		var c api.EmployeeEmergencyContactRecord
		var createdAt time.Time
		if err := rows.Scan(&c.ID, &c.EmployeeID, &c.ContactName, &c.Relationship, &c.Phone, &c.IsPrimary, &createdAt); err != nil {
			return nil, err
		}
		c.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		contacts = append(contacts, c)
	}
	return contacts, rows.Err()
}

func (r ESSRepository) AddEmergencyContact(ctx context.Context, workspaceID, employeeID uuid.UUID, req api.CreateEmergencyContactRequest) (api.EmployeeEmergencyContactRecord, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.EmployeeEmergencyContactRecord{}, err
	}
	isPrimary := false
	if req.IsPrimary != nil {
		isPrimary = *req.IsPrimary
	}
	var record api.EmployeeEmergencyContactRecord
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if isPrimary {
			if _, err := db.Exec(ctx, `UPDATE employee_emergency_contacts SET is_primary = false WHERE employee_id = $1`, employeeID); err != nil {
				return err
			}
		}
		var createdAt time.Time
		return db.QueryRow(ctx, `INSERT INTO employee_emergency_contacts (id, employee_id, contact_name, relationship, phone, is_primary, created_at) VALUES ($1, $2, $3, $4, $5, $6, now()) RETURNING id, employee_id, contact_name, relationship, phone, is_primary, created_at`, uuid.New(), employeeID, req.ContactName, req.Relationship, req.Phone, isPrimary).Scan(
			&record.ID, &record.EmployeeID, &record.ContactName, &record.Relationship, &record.Phone, &record.IsPrimary, &createdAt,
		)
	})
	if err != nil {
		return api.EmployeeEmergencyContactRecord{}, err
	}
	return record, nil
}

func (r ESSRepository) DeleteEmergencyContact(ctx context.Context, workspaceID, employeeID, contactID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	cmd, err := db.Exec(ctx, `DELETE FROM employee_emergency_contacts WHERE id = $1 AND employee_id = $2`, contactID, employeeID)
	if err != nil {
		return err
	}
	if cmd.RowsAffected() == 0 {
		return api.ErrESSNotFound
	}
	return nil
}

func (r ESSRepository) GetPendingDocuments(ctx context.Context, workspaceID, employeeID uuid.UUID) ([]api.AcknowledgementRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, document_id, employee_id, status FROM file_acknowledgements WHERE workspace_id = $1 AND employee_id = $2 AND status = 'pending' ORDER BY created_at DESC`, workspaceID, employeeID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	items := make([]api.AcknowledgementRecord, 0)
	for rows.Next() {
		var rec api.AcknowledgementRecord
		if err := rows.Scan(&rec.ID, &rec.DocumentID, &rec.EmployeeID, &rec.Status); err != nil {
			return nil, err
		}
		items = append(items, rec)
	}
	return items, rows.Err()
}

func (r ESSRepository) AcknowledgeDocument(ctx context.Context, workspaceID, employeeID, docID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	_, err = db.Exec(ctx, `UPDATE file_acknowledgements SET status = 'acknowledged', acknowledged_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND workspace_id = $2`, docID, workspaceID)
	return err
}

func (r ESSRepository) GetEmployeeDocuments(ctx context.Context, workspaceID, employeeID uuid.UUID) ([]api.EmployeeDocumentRecord, error) {
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT id, employee_id, workspace_id, document_type, file_name, file_path, file_size, mime_type, expires_at::text, is_active, uploaded_by, created_at FROM employee_documents WHERE employee_id = $1 AND workspace_id = $2 ORDER BY created_at DESC`, employeeID, workspaceID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	items := make([]api.EmployeeDocumentRecord, 0)
	for rows.Next() {
		var rec api.EmployeeDocumentRecord
		var createdAt time.Time
		if err := rows.Scan(&rec.ID, &rec.EmployeeID, &rec.WorkspaceID, &rec.DocumentType, &rec.FileName, &rec.FilePath, &rec.FileSize, &rec.MimeType, &rec.ExpiresAt, &rec.IsActive, &rec.UploadedBy, &createdAt); err != nil {
			return nil, err
		}
		rec.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		items = append(items, rec)
	}
	return items, rows.Err()
}
