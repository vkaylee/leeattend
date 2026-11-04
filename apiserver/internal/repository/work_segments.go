package repository

import (
	"context"
	"errors"
	"fmt"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/shopspring/decimal"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/service"
)

type WorkSegmentRepository struct {
	DB      DBTX
	BeginTx func(context.Context, uuid.UUID) (TxBeginner, error)
}

func (r WorkSegmentRepository) db(ctx context.Context) (DBTX, error) {
	if r.DB == nil {
		return nil, errors.New("work segment repository: database is not configured")
	}
	return r.DB, nil
}

func (r WorkSegmentRepository) begin(ctx context.Context, workspaceID uuid.UUID) (TxBeginner, error) {
	if r.BeginTx != nil {
		return r.BeginTx(ctx, workspaceID)
	}
	db, err := r.db(ctx)
	if err != nil {
		return nil, err
	}
	beginner, ok := db.(TxBeginner)
	if !ok {
		return nil, errors.New("work segment repository: tenant database does not support transactions")
	}
	return beginner, nil
}

const segmentSelect = `SELECT id, workspace_id, employment_id, date::text, template_id, interval_id, sort_order, scheduled_start::text, scheduled_end::text, segment_type::text, segment_status::text, compensation_rate::text, gap_compensation_before::text, gap_compensation_rate_before::text, break_config_override, ot_config_override, source, source_detail, notes, generation_metadata, created_at, updated_at FROM work_segments`

func (r WorkSegmentRepository) ListSegments(ctx context.Context, workspaceID uuid.UUID, filter api.WorkSegmentFilter, page, limit int) (api.WorkSegmentListResponse, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.WorkSegmentListResponse{}, err
	}
	where, args := segmentWhere(workspaceID, filter)
	var total int64
	if err := db.QueryRow(ctx, `SELECT count(*) FROM work_segments `+where, args...).Scan(&total); err != nil {
		return api.WorkSegmentListResponse{}, err
	}
	args = append(args, limit, (page-1)*limit)
	rows, err := db.Query(ctx, segmentSelect+` `+where+` ORDER BY date ASC, employment_id, sort_order ASC, id ASC LIMIT $`+fmt.Sprint(len(args)-1)+` OFFSET $`+fmt.Sprint(len(args)), args...)
	if err != nil {
		return api.WorkSegmentListResponse{}, err
	}
	defer rows.Close()
	segments := make([]api.WorkSegment, 0)
	for rows.Next() {
		segment, err := scanWorkSegment(rows)
		if err != nil {
			return api.WorkSegmentListResponse{}, err
		}
		segments = append(segments, segment)
	}
	return api.WorkSegmentListResponse{Segments: segments, TotalCount: total}, rows.Err()
}

func (r WorkSegmentRepository) GetSegment(ctx context.Context, workspaceID, segmentID uuid.UUID) (api.WorkSegment, error) {
	db, err := r.db(ctx)
	if err != nil {
		return api.WorkSegment{}, err
	}
	row := db.QueryRow(ctx, segmentSelect+` WHERE workspace_id = $1 AND id = $2`, workspaceID, segmentID)
	segment, err := scanWorkSegment(row)
	if errors.Is(err, pgx.ErrNoRows) {
		return api.WorkSegment{}, api.ErrWorkSegmentNotFound
	}
	return segment, err
}

func (r WorkSegmentRepository) CreateSegment(ctx context.Context, workspaceID uuid.UUID, req api.CreateWorkSegmentRequest) (api.WorkSegment, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.WorkSegment{}, err
	}
	id := uuid.New()
	now := time.Now().UTC()
	applySegmentDefaults(&req)
	var segment api.WorkSegment
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if err := validateSegmentReferences(ctx, db, workspaceID, req.EmploymentID, req.TemplateID); err != nil {
			return err
		}
		_, err := db.Exec(ctx, `INSERT INTO work_segments (id, workspace_id, employment_id, date, template_id, interval_id, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, break_config_override, ot_config_override, source, source_detail, notes, generation_metadata, created_at, updated_at) VALUES ($1, $2, $3, $4::date, $5, $6, $7, $8::time, $9::time, $10, $11, $12, $13, $14, $15, $16, $17, COALESCE($18, '{}'::jsonb), $19, '{}'::jsonb, $20, $20)`, id, workspaceID, req.EmploymentID, req.Date, req.TemplateID, req.IntervalID, req.SortOrder, req.ScheduledStart, req.ScheduledEnd, req.SegmentType, req.SegmentStatus, req.CompensationRate, req.GapCompensationBefore, req.GapCompensationRateBefore, jsonbArg(req.BreakConfigOverride), jsonbArg(req.OTConfigOverride), req.Source, jsonbArg(req.SourceDetail), req.Notes, now)
		if err != nil {
			return mapSegmentDBError(err)
		}
		segment, err = r.getSegmentTx(ctx, db, workspaceID, id)
		return err
	})
	if err != nil {
		return api.WorkSegment{}, err
	}
	return segment, nil
}

func (r WorkSegmentRepository) UpdateSegment(ctx context.Context, workspaceID, segmentID uuid.UUID, req api.UpdateWorkSegmentRequest) (api.WorkSegment, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return api.WorkSegment{}, err
	}
	now := time.Now().UTC()
	var segment api.WorkSegment
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if req.TemplateID != nil {
			if err := validateSegmentReferences(ctx, db, workspaceID, uuid.Nil, *req.TemplateID); err != nil {
				return err
			}
		}
		set, args := updateSegmentSet(req, now)
		args = append(args, segmentID, workspaceID)
		if _, err := db.Exec(ctx, `UPDATE work_segments SET `+set+` WHERE id = $`+fmt.Sprint(len(args)-1)+` AND workspace_id = $`+fmt.Sprint(len(args)), args...); err != nil {
			return mapSegmentDBError(err)
		}
		segment, err = r.getSegmentTx(ctx, db, workspaceID, segmentID)
		if errors.Is(err, pgx.ErrNoRows) {
			return api.ErrWorkSegmentNotFound
		}
		return err
	})
	if err != nil {
		return api.WorkSegment{}, err
	}
	return segment, nil
}

func (r WorkSegmentRepository) DeleteSegment(ctx context.Context, workspaceID, segmentID uuid.UUID) error {
	db, err := r.db(ctx)
	if err != nil {
		return err
	}
	tag, err := db.Exec(ctx, `DELETE FROM work_segments WHERE workspace_id = $1 AND id = $2`, workspaceID, segmentID)
	if err != nil {
		return err
	}
	if tag.RowsAffected() == 0 {
		return api.ErrWorkSegmentNotFound
	}
	return nil
}

func (r WorkSegmentRepository) BulkCreateSegments(ctx context.Context, workspaceID uuid.UUID, req api.BulkCreateWorkSegmentsRequest) ([]api.WorkSegment, error) {
	tx, err := r.begin(ctx, workspaceID)
	if err != nil {
		return nil, err
	}
	now := time.Now().UTC()
	segments := make([]api.WorkSegment, 0, len(req.Segments))
	err = service.WithTx(ctx, tx, func(ctx context.Context, db pgx.Tx) error {
		if req.ReplaceExisting {
			for _, item := range req.Segments {
				if _, err := db.Exec(ctx, `DELETE FROM work_segments WHERE workspace_id = $1 AND employment_id = $2 AND date = $3::date`, workspaceID, item.EmploymentID, item.Date); err != nil {
					return err
				}
			}
		}
		for _, item := range req.Segments {
			applySegmentDefaults(&item)
			if err := validateSegmentReferences(ctx, db, workspaceID, item.EmploymentID, item.TemplateID); err != nil {
				return err
			}
			id := uuid.New()
			if _, err := db.Exec(ctx, `INSERT INTO work_segments (id, workspace_id, employment_id, date, template_id, interval_id, sort_order, scheduled_start, scheduled_end, segment_type, segment_status, compensation_rate, gap_compensation_before, gap_compensation_rate_before, break_config_override, ot_config_override, source, source_detail, notes, generation_metadata, created_at, updated_at) VALUES ($1, $2, $3, $4::date, $5, $6, $7, $8::time, $9::time, $10, $11, $12, $13, $14, $15, $16, $17, COALESCE($18, '{}'::jsonb), $19, '{}'::jsonb, $20, $20)`, id, workspaceID, item.EmploymentID, item.Date, item.TemplateID, item.IntervalID, item.SortOrder, item.ScheduledStart, item.ScheduledEnd, item.SegmentType, item.SegmentStatus, item.CompensationRate, item.GapCompensationBefore, item.GapCompensationRateBefore, jsonbArg(item.BreakConfigOverride), jsonbArg(item.OTConfigOverride), item.Source, jsonbArg(item.SourceDetail), item.Notes, now); err != nil {
				return mapSegmentDBError(err)
			}
			segment, err := r.getSegmentTx(ctx, db, workspaceID, id)
			if err != nil {
				return err
			}
			segments = append(segments, segment)
		}
		return nil
	})
	if err != nil {
		return nil, err
	}
	return segments, nil
}

func (r WorkSegmentRepository) getSegmentTx(ctx context.Context, db DBTX, workspaceID, segmentID uuid.UUID) (api.WorkSegment, error) {
	return scanWorkSegment(db.QueryRow(ctx, segmentSelect+` WHERE workspace_id = $1 AND id = $2`, workspaceID, segmentID))
}

func validateSegmentReferences(ctx context.Context, db DBTX, workspaceID, employmentID, templateID uuid.UUID) error {
	if employmentID != uuid.Nil {
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employments WHERE id = $1 AND workspace_id = $2 AND deleted_at IS NULL)`, employmentID, workspaceID).Scan(&exists); err != nil {
			return err
		}
		if !exists {
			return api.ErrEmploymentNotFound
		}
	}
	var exists bool
	if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM shift_templates WHERE id = $1 AND workspace_id = $2)`, templateID, workspaceID).Scan(&exists); err != nil {
		return err
	}
	if !exists {
		return api.ErrShiftNotFound
	}
	return nil
}

func scanWorkSegment(row pgx.Row) (api.WorkSegment, error) {
	var s api.WorkSegment
	var compensation, gapRate string
	var createdAt, updatedAt time.Time
	if err := row.Scan(&s.ID, &s.WorkspaceID, &s.EmploymentID, &s.Date, &s.TemplateID, &s.IntervalID, &s.SortOrder, &s.ScheduledStart, &s.ScheduledEnd, &s.SegmentType, &s.SegmentStatus, &compensation, &s.GapCompensationBefore, &gapRate, &s.BreakConfigOverride, &s.OTConfigOverride, &s.Source, &s.SourceDetail, &s.Notes, &s.GenerationMetadata, &createdAt, &updatedAt); err != nil {
		return api.WorkSegment{}, err
	}
	var err error
	s.CompensationRate, err = decimal.NewFromString(compensation)
	if err != nil {
		return api.WorkSegment{}, err
	}
	s.GapCompensationRateBefore, err = decimal.NewFromString(gapRate)
	if err != nil {
		return api.WorkSegment{}, err
	}
	s.CreatedAt = createdAt.UTC().Format(time.RFC3339)
	s.UpdatedAt = updatedAt.UTC().Format(time.RFC3339)
	return s, nil
}

func segmentWhere(workspaceID uuid.UUID, filter api.WorkSegmentFilter) (string, []any) {
	clauses := []string{"workspace_id = $1"}
	args := []any{workspaceID}
	if len(filter.EmploymentIDs) > 0 {
		args = append(args, filter.EmploymentIDs)
		clauses = append(clauses, fmt.Sprintf("employment_id = ANY($%d)", len(args)))
	}
	if filter.StartDate != nil {
		args = append(args, *filter.StartDate)
		clauses = append(clauses, fmt.Sprintf("date >= $%d::date", len(args)))
	}
	if filter.EndDate != nil {
		args = append(args, *filter.EndDate)
		clauses = append(clauses, fmt.Sprintf("date <= $%d::date", len(args)))
	}
	if filter.SegmentType != "" {
		args = append(args, filter.SegmentType)
		clauses = append(clauses, fmt.Sprintf("segment_type::text = $%d", len(args)))
	}
	if filter.SegmentStatus != "" {
		args = append(args, filter.SegmentStatus)
		clauses = append(clauses, fmt.Sprintf("segment_status::text = $%d", len(args)))
	}
	if filter.Source != "" {
		args = append(args, filter.Source)
		clauses = append(clauses, fmt.Sprintf("source = $%d", len(args)))
	}
	return "WHERE " + joinStrings(clauses, " AND "), args
}

func updateSegmentSet(req api.UpdateWorkSegmentRequest, now time.Time) (string, []any) {
	sets := make([]string, 0, 10)
	args := make([]any, 0, 10)
	add := func(column string, value any) {
		args = append(args, value)
		sets = append(sets, fmt.Sprintf("%s = $%d", column, len(args)))
	}
	if req.TemplateID != nil {
		add("template_id", *req.TemplateID)
	}
	if req.IntervalID != nil {
		add("interval_id", *req.IntervalID)
	}
	if req.SortOrder != nil {
		add("sort_order", *req.SortOrder)
	}
	if req.ScheduledStart != nil {
		add("scheduled_start", *req.ScheduledStart)
	}
	if req.ScheduledEnd != nil {
		add("scheduled_end", *req.ScheduledEnd)
	}
	if req.SegmentType != nil {
		add("segment_type", *req.SegmentType)
	}
	if req.SegmentStatus != nil {
		add("segment_status", *req.SegmentStatus)
	}
	if req.CompensationRate != nil {
		add("compensation_rate", *req.CompensationRate)
	}
	if req.GapCompensationBefore != nil {
		add("gap_compensation_before", *req.GapCompensationBefore)
	}
	if req.GapCompensationRateBefore != nil {
		add("gap_compensation_rate_before", *req.GapCompensationRateBefore)
	}
	if req.BreakConfigOverride != nil {
		add("break_config_override", req.BreakConfigOverride)
	}
	if req.OTConfigOverride != nil {
		add("ot_config_override", req.OTConfigOverride)
	}
	if req.Source != nil {
		add("source", *req.Source)
	}
	if req.SourceDetail != nil {
		add("source_detail", req.SourceDetail)
	}
	if req.Notes != nil {
		add("notes", *req.Notes)
	}
	add("updated_at", now)
	return joinStrings(sets, ", "), args
}

func applySegmentDefaults(req *api.CreateWorkSegmentRequest) {
	if req.SegmentType == "" {
		req.SegmentType = "work"
	}
	if req.SegmentStatus == "" {
		req.SegmentStatus = "scheduled"
	}
	if req.GapCompensationBefore == "" {
		req.GapCompensationBefore = "unpaid"
	}
	if req.Source == "" {
		req.Source = "manual"
	}
	if len(req.SourceDetail) == 0 {
		req.SourceDetail = []byte(`{}`)
	}
	if req.CompensationRate.IsZero() {
		req.CompensationRate = decimal.NewFromInt(1)
	}
}

func jsonbArg(raw []byte) any {
	if len(raw) == 0 {
		return nil
	}
	return raw
}

func mapSegmentDBError(err error) error {
	var pgErr *pgconn.PgError
	if errors.As(err, &pgErr) && pgErr.Code == "23505" {
		return api.ErrWorkSegmentConflict
	}
	return err
}

func joinStrings(values []string, separator string) string {
	result := ""
	for index, value := range values {
		if index > 0 {
			result += separator
		}
		result += value
	}
	return result
}
