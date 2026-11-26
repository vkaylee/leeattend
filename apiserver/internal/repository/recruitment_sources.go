package repository

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

type RecruitmentSourceRepository struct {
	DB DBTX
}

const recruitmentSourceColumns = `id, source_type, name, referrer_employee_id,
	referral_bonus_amount::text, referral_bonus_paid, agency_contact_name, agency_contact_email,
	agency_fee_amount::text, agency_fee_paid, notes, created_at, updated_at`

func scanRecruitmentSource(row interface{ Scan(...any) error }) (api.RecruitmentSource, error) {
	var source api.RecruitmentSource
	var createdAt, updatedAt time.Time
	var referralBonus, agencyFee *string
	if err := row.Scan(&source.ID, &source.SourceType, &source.Name, &source.ReferrerEmployeeID,
		&referralBonus, &source.ReferralBonusPaid, &source.AgencyContactName, &source.AgencyContactEmail,
		&agencyFee, &source.AgencyFeePaid, &source.Notes, &createdAt, &updatedAt); err != nil {
		return api.RecruitmentSource{}, err
	}
	if referralBonus != nil {
		value, err := decimal.NewFromString(*referralBonus)
		if err != nil {
			return api.RecruitmentSource{}, err
		}
		source.ReferralBonusAmount = &value
	}
	if agencyFee != nil {
		value, err := decimal.NewFromString(*agencyFee)
		if err != nil {
			return api.RecruitmentSource{}, err
		}
		source.AgencyFeeAmount = &value
	}
	source.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	source.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return source, nil
}

func (r RecruitmentSourceRepository) ListRecruitmentSources(ctx context.Context, page, limit int) (api.RecruitmentSourceListResponse, error) {
	db, err := requireDB(r.DB, "recruitment source repository")
	if err != nil {
		return api.RecruitmentSourceListResponse{}, err
	}
	result := api.RecruitmentSourceListResponse{Data: make([]api.RecruitmentSource, 0), Page: page, Limit: limit}
	if err := db.QueryRow(ctx, `SELECT count(*) FROM recruitment_sources`).Scan(&result.Total); err != nil {
		return api.RecruitmentSourceListResponse{}, err
	}
	rows, err := db.Query(ctx, `SELECT `+recruitmentSourceColumns+` FROM recruitment_sources ORDER BY created_at DESC LIMIT $1 OFFSET $2`, limit, (page-1)*limit)
	if err != nil {
		return api.RecruitmentSourceListResponse{}, err
	}
	defer rows.Close()
	for rows.Next() {
		item, scanErr := scanRecruitmentSource(rows)
		if scanErr != nil {
			return api.RecruitmentSourceListResponse{}, scanErr
		}
		result.Data = append(result.Data, item)
	}
	return result, rows.Err()
}

func (r RecruitmentSourceRepository) CreateRecruitmentSource(ctx context.Context, req api.CreateRecruitmentSourceRequest) (api.RecruitmentSource, error) {
	db, err := requireDB(r.DB, "recruitment source repository")
	if err != nil {
		return api.RecruitmentSource{}, err
	}
	id := uuid.New()
	row := db.QueryRow(ctx, `INSERT INTO recruitment_sources
		(id, source_type, name, referrer_employee_id, referral_bonus_amount, referral_bonus_paid, agency_contact_name, agency_contact_email, agency_fee_amount, agency_fee_paid, notes, created_at, updated_at)
		VALUES ($1, $2::recruitment_source_type, $3, $4, $5, false, $6, $7, $8, false, $9, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
		RETURNING `+recruitmentSourceColumns,
		id, req.SourceType, req.Name, req.ReferrerEmployeeID, req.ReferralBonusAmount, req.AgencyContactName, req.AgencyContactEmail, req.AgencyFeeAmount, req.Notes)
	return scanRecruitmentSource(row)
}

func (r RecruitmentSourceRepository) UpdateRecruitmentSource(ctx context.Context, id uuid.UUID, req api.UpdateRecruitmentSourceRequest) (api.RecruitmentSource, error) {
	db, err := requireDB(r.DB, "recruitment source repository")
	if err != nil {
		return api.RecruitmentSource{}, err
	}
	sets := []string{"updated_at = CURRENT_TIMESTAMP"}
	args := make([]any, 0, 10)
	add := func(column string, value any) {
		args = append(args, value)
		sets = append(sets, fmt.Sprintf("%s = $%d", column, len(args)))
	}
	if req.Name != nil {
		add("name", *req.Name)
	}
	if req.ReferrerEmployeeID != nil {
		add("referrer_employee_id", *req.ReferrerEmployeeID)
	}
	if req.ReferralBonusAmount != nil {
		add("referral_bonus_amount", *req.ReferralBonusAmount)
	}
	if req.ReferralBonusPaid != nil {
		add("referral_bonus_paid", *req.ReferralBonusPaid)
	}
	if req.AgencyContactName != nil {
		add("agency_contact_name", *req.AgencyContactName)
	}
	if req.AgencyContactEmail != nil {
		add("agency_contact_email", *req.AgencyContactEmail)
	}
	if req.AgencyFeeAmount != nil {
		add("agency_fee_amount", *req.AgencyFeeAmount)
	}
	if req.AgencyFeePaid != nil {
		add("agency_fee_paid", *req.AgencyFeePaid)
	}
	if req.Notes != nil {
		add("notes", *req.Notes)
	}
	args = append(args, id)
	query := `UPDATE recruitment_sources SET ` + strings.Join(sets, ", ") + fmt.Sprintf(` WHERE id = $%d RETURNING `+recruitmentSourceColumns, len(args))
	item, err := scanRecruitmentSource(db.QueryRow(ctx, query, args...))
	if errors.Is(err, pgx.ErrNoRows) {
		return api.RecruitmentSource{}, api.ErrRecruitmentSourceNotFound
	}
	return item, err
}

func (r RecruitmentSourceRepository) DeleteRecruitmentSource(ctx context.Context, id uuid.UUID) error {
	db, err := requireDB(r.DB, "recruitment source repository")
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `DELETE FROM recruitment_sources WHERE id = $1`, id)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrRecruitmentSourceNotFound
	}
	return nil
}

func (r RecruitmentSourceRepository) RecruitmentSourceBreakdown(ctx context.Context) ([]api.RecruitmentSourceBreakdown, error) {
	db, err := requireDB(r.DB, "recruitment source repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT COALESCE(rs.source_type::text, 'direct') AS source_type, COALESCE(rs.name, 'Direct Hire') AS source_name, COUNT(*) AS count
		FROM employees e LEFT JOIN recruitment_sources rs ON e.recruitment_source_id = rs.id
		WHERE EXISTS (SELECT 1 FROM employment_segments emp WHERE emp.employee_id = e.id AND emp.status = 'active' AND emp.deleted_at IS NULL)
		GROUP BY rs.source_type, rs.name ORDER BY count DESC`)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	result := make([]api.RecruitmentSourceBreakdown, 0)
	for rows.Next() {
		var item api.RecruitmentSourceBreakdown
		if err := rows.Scan(&item.SourceType, &item.SourceName, &item.Count); err != nil {
			return nil, err
		}
		result = append(result, item)
	}
	return result, rows.Err()
}
