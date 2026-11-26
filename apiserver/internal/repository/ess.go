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
	BeginTx func(context.Context) (TxBeginner, error)
}

func (r ESSRepository) begin(ctx context.Context) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx)
	}
	db, err := requireDB(r.DB, "ess repository")
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("ess repository: database does not support transactions")
	}
	return beginner, nil
}

func (r ESSRepository) ResolveEmployeeForUser(ctx context.Context, userID uuid.UUID) (uuid.UUID, error) {
	db, err := requireDB(r.DB, "ess repository")
	if err != nil {
		return uuid.Nil, err
	}
	var empID uuid.UUID
	err = db.QueryRow(ctx, `SELECT t.employee_id FROM employee_tenures t JOIN employees e ON e.id = t.employee_id WHERE t.user_id = $1 AND t.deleted_at IS NULL AND t.start_date <= `+orgTodaySQL+` AND (t.end_date IS NULL OR t.end_date >= `+orgTodaySQL+`) AND e.deleted_at IS NULL ORDER BY t.start_date DESC, t.id DESC LIMIT 1`, userID).Scan(&empID)
	if errors.Is(err, pgx.ErrNoRows) {
		return uuid.Nil, api.ErrESSNoEmployeeRecord
	}
	return empID, err
}

func (r ESSRepository) GetMyProfile(ctx context.Context, employeeID uuid.UUID) (api.MyProfileResponse, error) {
	db, err := requireDB(r.DB, "ess repository")
	if err != nil {
		return api.MyProfileResponse{}, err
	}
	var emp api.EmployeeRecord
	var createdAt, updatedAt time.Time
	err = db.QueryRow(ctx, `
		SELECT
			e.id,
			tenure.user_id,
			e.full_name,
			COALESCE(tenure.employee_number, ''),
			e.tags,
			e.created_at,
			e.updated_at,
			d.name AS department_name,
			emp.id AS active_employment_id,
			emp.employer_id,
			emp.primary_location_id,
			tenure.id AS tenure_id,
			CASE WHEN tenure.id IS NOT NULL THEN 'active' END AS tenure_status,
			tenure.hire_type
		FROM employees e
		`+sqlEffectiveTenureLateral+`
		LEFT JOIN LATERAL (
			SELECT em.id, em.employer_id, em.department_id, em.primary_location_id
			FROM employments em
			JOIN employment_relationships rel ON rel.id = em.relationship_id AND rel.deleted_at IS NULL
			WHERE em.employee_id = e.id
			  AND em.deleted_at IS NULL
			  AND em.status = 'active'
			  AND (tenure.id IS NULL OR rel.tenure_id = tenure.id)
			ORDER BY
				CASE
					WHEN em.start_date <= `+orgTodaySQL+` AND (em.end_date IS NULL OR em.end_date >= `+orgTodaySQL+`) THEN 0
					WHEN em.start_date > `+orgTodaySQL+` AND em.end_date IS NULL THEN 1
					ELSE 2
				END ASC,
				em.start_date DESC,
				em.id DESC
			LIMIT 1
		) emp ON TRUE
		LEFT JOIN departments d ON d.id = emp.department_id
		WHERE e.id = $1 AND e.deleted_at IS NULL`, employeeID).Scan(
		&emp.ID,
		&emp.UserID,
		&emp.FullName,
		&emp.EmployeeIDNumber,
		&emp.Tags,
		&createdAt,
		&updatedAt,
		&emp.DepartmentName,
		&emp.ActiveEmploymentID,
		&emp.EmployerID,
		&emp.PrimaryLocationID,
		&emp.TenureID,
		&emp.TenureStatus,
		&emp.HireType,
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

	contacts, err := r.GetEmergencyContacts(ctx, employeeID)
	if err != nil {
		return api.MyProfileResponse{}, err
	}

	return api.MyProfileResponse{
		Employee:          emp,
		Profile:           profile,
		EmergencyContacts: contacts,
	}, nil
}

func (r ESSRepository) UpsertProfile(ctx context.Context, employeeID uuid.UUID, req api.UpdateEmployeeProfileRequest) (api.EmployeeProfileRecord, error) {
	db, err := requireDB(r.DB, "ess repository")
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

func (r ESSRepository) GetEmergencyContacts(ctx context.Context, employeeID uuid.UUID) ([]api.EmployeeEmergencyContactRecord, error) {
	db, err := requireDB(r.DB, "ess repository")
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

func (r ESSRepository) AddEmergencyContact(ctx context.Context, employeeID uuid.UUID, req api.CreateEmergencyContactRequest) (api.EmployeeEmergencyContactRecord, error) {
	tx, err := r.begin(ctx)
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

func (r ESSRepository) DeleteEmergencyContact(ctx context.Context, employeeID, contactID uuid.UUID) error {
	db, err := requireDB(r.DB, "ess repository")
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

func (r ESSRepository) GetPendingDocuments(ctx context.Context, employeeID uuid.UUID) ([]api.AcknowledgementRecord, error) {
	db, err := requireDB(r.DB, "ess repository")
	if err != nil {
		return nil, err
	}
	// Scope pending acknowledgements strictly to the employee's current effective tenure.
	// Legacy NULL tenure_id rows are bounded by the current tenure start_date; prior-tenure rows are never exposed.
	rows, err := db.Query(ctx, `
		SELECT fa.id, fa.document_id, fa.employee_id, fa.status
		FROM file_acknowledgements fa
		JOIN LATERAL (
			SELECT t.id, t.start_date, t.end_date
			FROM employee_tenures t
			WHERE t.employee_id = fa.employee_id
			  AND t.deleted_at IS NULL
			  AND t.start_date <= `+orgTodaySQL+`
			  AND (t.end_date IS NULL OR t.end_date >= `+orgTodaySQL+`)
			ORDER BY t.start_date DESC, t.id DESC
			LIMIT 1
		) cur ON TRUE
		WHERE fa.employee_id = $1
		  AND fa.status = 'pending'
		  AND (
		      fa.tenure_id = cur.id
		      OR (fa.tenure_id IS NULL AND fa.created_at::date >= cur.start_date)
		  )
		ORDER BY fa.created_at DESC
	`, employeeID)
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

func (r ESSRepository) AcknowledgeDocument(ctx context.Context, employeeID, docID uuid.UUID) error {
	db, err := requireDB(r.DB, "ess repository")
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `
		UPDATE file_acknowledgements fa
		SET status = 'acknowledged', acknowledged_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP
		FROM employee_tenures cur
		WHERE fa.id = $1
		  AND fa.employee_id = $2
		  AND fa.status = 'pending'
		  AND cur.employee_id = fa.employee_id
		  AND cur.deleted_at IS NULL
		  AND cur.start_date <= `+orgTodaySQL+`
		  AND (cur.end_date IS NULL OR cur.end_date >= `+orgTodaySQL+`)
		  AND (
		      fa.tenure_id = cur.id
		      OR (fa.tenure_id IS NULL AND fa.created_at::date >= cur.start_date)
		  )
	`, docID, employeeID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrESSNotFound
	}
	return nil
}

func (r ESSRepository) GetEmployeeDocuments(ctx context.Context, employeeID uuid.UUID) ([]api.EmployeeDocumentRecord, error) {
	db, err := requireDB(r.DB, "ess repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `
		SELECT d.id, d.employee_id, d.document_type, d.file_name, d.file_path, d.file_size, d.mime_type, d.expires_at::text, d.is_active, d.uploaded_by, d.created_at
		FROM employee_documents d
		JOIN LATERAL (
			SELECT t.id, t.start_date, t.end_date
			FROM employee_tenures t
			WHERE t.employee_id = d.employee_id
			  AND t.deleted_at IS NULL
			  AND t.start_date <= `+orgTodaySQL+`
			  AND (t.end_date IS NULL OR t.end_date >= `+orgTodaySQL+`)
			ORDER BY t.start_date DESC, t.id DESC
			LIMIT 1
		) cur ON TRUE
		WHERE d.employee_id = $1
		  AND (
		      d.tenure_id = cur.id
		      OR (d.tenure_id IS NULL AND d.created_at::date >= cur.start_date)
		  )
		ORDER BY d.created_at DESC
	`, employeeID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	items := make([]api.EmployeeDocumentRecord, 0)
	for rows.Next() {
		var rec api.EmployeeDocumentRecord
		var createdAt time.Time
		if err := rows.Scan(&rec.ID, &rec.EmployeeID, &rec.DocumentType, &rec.FileName, &rec.FilePath, &rec.FileSize, &rec.MimeType, &rec.ExpiresAt, &rec.IsActive, &rec.UploadedBy, &createdAt); err != nil {
			return nil, err
		}
		rec.CreatedAt = createdAt.UTC().Format(time.RFC3339)
		items = append(items, rec)
	}
	return items, rows.Err()
}
