package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type EmployerRepository struct {
	DB DBTX
}

func (r EmployerRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("employer repository: database is not configured")
	}
	return r.DB, nil
}

const employerColumns = `id, name, tax_code, legal_address, created_at, updated_at,
	parent_id, relationship_type, business_type, registration_number, phone, email, website,
	incorporation_date::text, country_code, currency_code`

func scanEmployer(row interface{ Scan(...any) error }) (api.Employer, error) {
	var employer api.Employer
	var createdAt, updatedAt time.Time
	if err := row.Scan(&employer.ID, &employer.Name, &employer.TaxCode, &employer.LegalAddress, &createdAt, &updatedAt,
		&employer.ParentID, &employer.RelationshipType, &employer.BusinessType, &employer.RegistrationNumber, &employer.Phone, &employer.Email, &employer.Website,
		&employer.IncorporationDate, &employer.CountryCode, &employer.CurrencyCode); err != nil {
		return api.Employer{}, err
	}
	employer.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	employer.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return employer, nil
}

func (r EmployerRepository) ListEmployers(ctx context.Context, page, limit int) (api.EmployerListResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.EmployerListResponse{}, err
	}
	result := api.EmployerListResponse{Data: make([]api.Employer, 0), Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM employers WHERE deleted_at IS NULL`).Scan(&result.Total); err != nil {
		return api.EmployerListResponse{}, err
	}
	rows, err := db.Query(ctx, `SELECT `+employerColumns+` FROM employers WHERE deleted_at IS NULL ORDER BY name ASC LIMIT $1 OFFSET $2`, limit, (page-1)*limit)
	if err != nil {
		return api.EmployerListResponse{}, err
	}
	defer rows.Close()
	for rows.Next() {
		item, scanErr := scanEmployer(rows)
		if scanErr != nil {
			return api.EmployerListResponse{}, scanErr
		}
		result.Data = append(result.Data, item)
	}
	return result, rows.Err()
}

func (r EmployerRepository) CreateEmployer(ctx context.Context, req api.CreateEmployerRequest) (api.Employer, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.Employer{}, err
	}
	id := uuid.New()
	row := db.QueryRow(ctx, `INSERT INTO employers
		(id, name, tax_code, legal_address, parent_id, relationship_type, business_type, registration_number, phone, email, website, incorporation_date, country_code, currency_code, created_at, updated_at)
		VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::date, $13, $14, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
		RETURNING `+employerColumns,
		id, req.Name, req.TaxCode, req.LegalAddress, req.ParentID, req.RelationshipType, req.BusinessType, req.RegistrationNumber, req.Phone, req.Email, req.Website, req.IncorporationDate, req.CountryCode, req.CurrencyCode)
	return scanEmployer(row)
}

func (r EmployerRepository) GetEmployer(ctx context.Context, id uuid.UUID) (api.Employer, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.Employer{}, err
	}
	item, err := scanEmployer(db.QueryRow(ctx, `SELECT `+employerColumns+` FROM employers WHERE id = $1 AND deleted_at IS NULL`, id))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.Employer{}, api.ErrEmployerNotFound
	}
	return item, err
}

func (r EmployerRepository) UpdateEmployer(ctx context.Context, id uuid.UUID, req api.UpdateEmployerRequest) (api.Employer, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.Employer{}, err
	}
	sets := []string{"updated_at = CURRENT_TIMESTAMP"}
	args := make([]any, 0, 13)
	add := func(column string, value any) {
		args = append(args, value)
		sets = append(sets, fmt.Sprintf("%s = $%d", column, len(args)))
	}
	if req.Name != nil {
		add("name", *req.Name)
	}
	if req.TaxCode != nil {
		add("tax_code", *req.TaxCode)
	}
	if req.LegalAddress != nil {
		add("legal_address", *req.LegalAddress)
	}
	if req.ParentIDPresent {
		if req.ParentID == nil {
			add("parent_id", nil)
		} else {
			add("parent_id", *req.ParentID)
		}
	}
	if req.RelationshipTypePresent {
		if req.RelationshipType == nil {
			add("relationship_type", nil)
		} else {
			add("relationship_type", *req.RelationshipType)
		}
	}
	if req.BusinessType != nil {
		add("business_type", *req.BusinessType)
	}
	if req.RegistrationNumber != nil {
		add("registration_number", *req.RegistrationNumber)
	}
	if req.Phone != nil {
		add("phone", *req.Phone)
	}
	if req.Email != nil {
		add("email", *req.Email)
	}
	if req.Website != nil {
		add("website", *req.Website)
	}
	if req.IncorporationDate != nil {
		add("incorporation_date", *req.IncorporationDate)
	}
	if req.CountryCode != nil {
		add("country_code", *req.CountryCode)
	}
	if req.CurrencyCode != nil {
		add("currency_code", *req.CurrencyCode)
	}
	args = append(args, id)
	query := `UPDATE employers SET ` + strings.Join(sets, ", ") + fmt.Sprintf(` WHERE id = $%d AND deleted_at IS NULL RETURNING `+employerColumns, len(args))
	item, err := scanEmployer(db.QueryRow(ctx, query, args...))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.Employer{}, api.ErrEmployerNotFound
	}
	return item, err
}

func (r EmployerRepository) DeleteEmployer(ctx context.Context, id uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	var count int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM employments WHERE employer_id = $1 AND status = 'active' AND deleted_at IS NULL`, id).Scan(&count); err != nil {
		return err
	}
	if count > 0 {
		return api.ErrEmployerInUse
	}
	tag, err := db.Exec(ctx, `UPDATE employers SET deleted_at = CURRENT_TIMESTAMP, updated_at = CURRENT_TIMESTAMP WHERE id = $1 AND deleted_at IS NULL`, id)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrEmployerNotFound
	}
	return nil
}
