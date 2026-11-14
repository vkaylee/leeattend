package repository

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgconn"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestReportRepository_CreateExportJob(t *testing.T) {
	jobID := uuid.New()
	requesterID := uuid.New()
	var capturedSQL string
	var capturedArgs []any

	db := &fakeDBTX{
		execFn: func(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
			capturedSQL = sql
			capturedArgs = args
			return pgconn.NewCommandTag("INSERT 0 1"), nil
		},
	}

	repo := ReportRepository{DB: db}
	job := api.ReportJob{
		JobID:       jobID,
		RequesterID: requesterID,
		Status:      "pending",
	}
	req := api.ExportReportRequest{
		ReportType: "summary",
		StartDate:  "2026-01-01",
		EndDate:    "2026-01-31",
	}

	if err := repo.CreateExportJob(context.Background(), job, req); err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	if len(capturedArgs) != 2 {
		t.Fatalf("expected job id and payload args, got %d", len(capturedArgs))
	}
	if capturedArgs[0] != jobID {
		t.Errorf("expected job ID %s, got %v", jobID, capturedArgs[0])
	}

	rawPayload, ok := capturedArgs[1].([]byte)
	if !ok {
		t.Fatalf("expected []byte payload, got %T", capturedArgs[1])
	}
	var payload struct {
		JobID       string `json:"job_id"`
		RequesterID string `json:"requester_id"`
		Request     struct {
			ReportType  string   `json:"report_type"`
			StartDate   string   `json:"start_date"`
			EndDate     string   `json:"end_date"`
			EmployeeIDs []string `json:"employee_ids"`
		} `json:"request"`
	}
	if err := json.Unmarshal(rawPayload, &payload); err != nil {
		t.Fatalf("failed to decode payload: %v", err)
	}
	if payload.JobID != jobID.String() {
		t.Errorf("expected payload job_id %s, got %s", jobID, payload.JobID)
	}
	if payload.RequesterID != requesterID.String() {
		t.Errorf("expected requester_id %s, got %s", requesterID, payload.RequesterID)
	}
	if payload.Request.ReportType != "summary" || payload.Request.StartDate != "2026-01-01" || payload.Request.EndDate != "2026-01-31" {
		t.Errorf("export request not embedded in payload: %+v", payload.Request)
	}
	if capturedSQL == "" {
		t.Error("expected sql to be executed")
	}
}

func TestReportRepository_CreateExportJobPropagatesExecError(t *testing.T) {
	expectedErr := errors.New("insert failed")
	db := &fakeDBTX{
		execFn: func(ctx context.Context, sql string, args ...any) (pgconn.CommandTag, error) {
			return pgconn.CommandTag{}, expectedErr
		},
	}

	repo := ReportRepository{DB: db}
	if err := repo.CreateExportJob(context.Background(), api.ReportJob{JobID: uuid.New()}, api.ExportReportRequest{ReportType: "summary"}); !errors.Is(err, expectedErr) {
		t.Fatalf("expected %v, got %v", expectedErr, err)
	}
}

func TestReportRepository_RequiresDatabase(t *testing.T) {
	repo := ReportRepository{}
	if err := repo.CreateExportJob(context.Background(), api.ReportJob{}, api.ExportReportRequest{}); err == nil {
		t.Fatal("expected error when database is not configured")
	}
	if _, err := repo.GetAttendanceReport(context.Background(), api.AttendanceReportFilter{}); err == nil {
		t.Fatal("expected error when database is not configured")
	}
	if _, err := repo.GetNoShowReport(context.Background(), api.NoShowReportFilter{}); err == nil {
		t.Fatal("expected error when database is not configured")
	}
	if _, err := repo.ResolveBehalfEmployeeIDs(context.Background(), uuid.New()); err == nil {
		t.Fatal("expected error when database is not configured")
	}
}

func TestReportRepository_GetAttendanceReportEmploymentNotFound(t *testing.T) {
	employmentID := uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(ctx context.Context, sql string, args ...any) pgx.Row {
			return fakeRow{
				scanFn: func(dest ...any) error {
					*(dest[0].(*bool)) = false
					return nil
				},
			}
		},
	}

	repo := ReportRepository{DB: db}
	_, err := repo.GetAttendanceReport(context.Background(), api.AttendanceReportFilter{
		EmploymentID: &employmentID,
		Page:         1,
		Limit:        10,
	})
	if !errors.Is(err, api.ErrReportNotFound) {
		t.Fatalf("expected ErrReportNotFound, got %v", err)
	}
}

func TestReportRepository_GetAttendanceReportDailyMapsOTClaimStatus(t *testing.T) {
	employmentID, employeeID := uuid.New(), uuid.New()
	statuses := []string{"none", "pending_employee", "pending_manager", "approved", "rejected"}
	rows := make([][]any, 0, len(statuses))
	for _, status := range statuses {
		rows = append(rows, []any{
			"2026-01-01", employmentID, employeeID, "Alice", "EMP01",
			"", "", "", "", "",
			480, 480, 0, status, 8.0,
		})
	}

	var dataQuery string
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = int64(len(rows))
				return nil
			}}
		},
		queryFn: func(_ context.Context, query string, _ ...any) (pgx.Rows, error) {
			dataQuery = query
			return &fakeRows{rows: rows}, nil
		},
	}

	response, err := (ReportRepository{DB: db}).GetAttendanceReport(context.Background(), api.AttendanceReportFilter{
		StartDate: "2026-01-01", EndDate: "2026-01-31", ReportType: "daily", Page: 1, Limit: 20,
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(dataQuery, "COALESCE(ads.ot_status::text, 'none')") {
		t.Fatalf("daily query must normalize missing status to none: %s", dataQuery)
	}

	var got []api.DailyAttendanceRow
	if err := json.Unmarshal(response.Data, &got); err != nil {
		t.Fatalf("failed to decode daily rows: %v", err)
	}
	if len(got) != len(statuses) {
		t.Fatalf("expected %d daily rows, got %d", len(statuses), len(got))
	}
	for i, status := range statuses {
		if got[i].OTClaimStatus != status {
			t.Errorf("row %d: expected OT claim status %q, got %q", i, status, got[i].OTClaimStatus)
		}
	}
}

func TestReportRepository_GetNoShowReportUsesLatestOfficialRevision(t *testing.T) {
	var countQuery, dataQuery string
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			countQuery = query
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 0
				return nil
			}}
		},
		queryFn: func(_ context.Context, query string, _ ...any) (pgx.Rows, error) {
			dataQuery = query
			return &fakeRows{}, nil
		},
	}

	response, err := (ReportRepository{DB: db}).GetNoShowReport(context.Background(), api.NoShowReportFilter{
		Date: "2026-01-01", Page: 1, Limit: 20,
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if response.Total != 0 {
		t.Fatalf("expected empty result, got total %d", response.Total)
	}
	for name, query := range map[string]string{"count": countQuery, "data": dataQuery} {
		if !strings.Contains(query, "latest_official_revisions") || !strings.Contains(query, "DISTINCT ON (employment_id, business_date)") || !strings.Contains(query, "revision_number DESC") {
			t.Fatalf("%s query must select latest official revision: %s", name, query)
		}
		if !strings.Contains(query, "schedule_day_revisions") || !strings.Contains(query, "work_segments") {
			t.Fatalf("%s query must use official schedule tables: %s", name, query)
		}
		if strings.Contains(query, "shift_rosters") || strings.Contains(query, "09:00:00") || strings.Contains(query, "17:00:00") {
			t.Fatalf("%s query must not synthesize roster hours: %s", name, query)
		}
		if !strings.Contains(query, "explicit_off") {
			t.Fatalf("%s query must exclude explicit off: %s", name, query)
		}
	}
}

type reportRows struct {
	rows [][]any
	idx  int
}

func (r *reportRows) Close()                                       {}
func (r *reportRows) Err() error                                   { return nil }
func (r *reportRows) CommandTag() pgconn.CommandTag                { return pgconn.CommandTag{} }
func (r *reportRows) FieldDescriptions() []pgconn.FieldDescription { return nil }
func (r *reportRows) Values() ([]any, error)                       { return nil, nil }
func (r *reportRows) RawValues() [][]byte                          { return nil }
func (r *reportRows) Conn() *pgx.Conn                              { return nil }
func (r *reportRows) Next() bool {
	if r.idx >= len(r.rows) {
		return false
	}
	r.idx++
	return true
}
func (r *reportRows) Scan(dest ...any) error {
	row := r.rows[r.idx-1]
	for i, value := range row {
		switch target := dest[i].(type) {
		case **uuid.UUID:
			if value == nil {
				*target = nil
				continue
			}
			id := value.(uuid.UUID)
			*target = &id
		case *uuid.UUID:
			*target = value.(uuid.UUID)
		case **string:
			if value == nil {
				*target = nil
				continue
			}
			text := value.(string)
			*target = &text
		case *string:
			*target = value.(string)
		}
	}
	return nil
}

func TestReportRepository_GetNoShowReportClassifiesMissingCoverage(t *testing.T) {
	employmentID, employeeID, revisionID := uuid.New(), uuid.New(), uuid.New()
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 2
				return nil
			}}
		},
		queryFn: func(_ context.Context, _ string, _ ...any) (pgx.Rows, error) {
			return &reportRows{rows: [][]any{
				{nil, employmentID, employeeID, "Missing", "EMP-M", nil, nil, nil, "2026-01-01", nil},
				{revisionID, employmentID, employeeID, "Absent", "EMP-A", "Day", "08:00:00", "17:00:00", "2026-01-01", "published"},
			}}, nil
		},
	}

	response, err := (ReportRepository{DB: db}).GetNoShowReport(context.Background(), api.NoShowReportFilter{
		Date: "2026-01-01", Page: 1, Limit: 20,
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	var rows []api.NoShowReportRow
	if err := json.Unmarshal(response.Data, &rows); err != nil {
		t.Fatalf("failed to decode no-show rows: %v", err)
	}
	if len(rows) != 2 {
		t.Fatalf("expected two rows, got %d", len(rows))
	}
	if rows[0].Status != "absent" || rows[0].RecordingIssue != "missing_schedule" {
		t.Fatalf("missing schedule must preserve absent status and report missing coverage: %+v", rows[0])
	}
	if rows[1].Status != "absent" || rows[1].RecordingIssue != "missing_punch_in" {
		t.Fatalf("published schedule must preserve absent compatibility: %+v", rows[1])
	}
}

func TestReportRepository_GetNoShowReportAggregatesOfficialIntervals(t *testing.T) {
	var dataQuery string
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, _ string, _ ...any) pgx.Row {
			return fakeRow{scanFn: func(dest ...any) error {
				*(dest[0].(*int64)) = 0
				return nil
			}}
		},
		queryFn: func(_ context.Context, query string, _ ...any) (pgx.Rows, error) {
			dataQuery = query
			return &fakeRows{}, nil
		},
	}

	_, err := (ReportRepository{DB: db}).GetNoShowReport(context.Background(), api.NoShowReportFilter{
		Date: "2026-01-01", Page: 1, Limit: 20,
	})
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if !strings.Contains(dataQuery, "GROUP BY sdr.id, sdr.employment_id, sdr.business_date, sdr.state") {
		t.Fatalf("official intervals must be collapsed per revision: %s", dataQuery)
	}
	if !strings.Contains(dataQuery, "NOT EXISTS (SELECT 1 FROM time_entries") {
		t.Fatalf("no-show query must check punches without interval multiplication: %s", dataQuery)
	}
}

func TestReportRepository_ResolveBehalfEmployeeIDs(t *testing.T) {
	userID := uuid.New()
	employeeID := uuid.New()
	var capturedSQL string
	var capturedArgs []any

	db := &fakeDBTX{
		queryFn: func(ctx context.Context, sql string, args ...any) (pgx.Rows, error) {
			capturedSQL = sql
			capturedArgs = args
			return &fakeRows{rows: [][]any{{employeeID}}}, nil
		},
	}

	repo := ReportRepository{DB: db}
	ids, err := repo.ResolveBehalfEmployeeIDs(context.Background(), userID)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if len(ids) != 1 || ids[0] != employeeID {
		t.Fatalf("unexpected employee ids: %v", ids)
	}
	if len(capturedArgs) != 1 || capturedArgs[0] != userID {
		t.Fatalf("expected only the assignee user argument, got %v", capturedArgs)
	}
	if capturedSQL == "" {
		t.Error("expected sql to be executed")
	}
}

func TestRedisReportQueue_Lifecycle(t *testing.T) {
	queue := &RedisReportQueue{}
	jobID := uuid.New()
	requesterID := uuid.New()

	job := api.ReportJob{
		JobID:        jobID,
		RequesterID:  requesterID,
		Status:       "pending",
		TotalBatches: 3,
		CreatedAt:    time.Now().UTC(),
	}

	if err := queue.EnqueueReport(context.Background(), job, api.ExportReportRequest{ReportType: "summary"}); err != nil {
		t.Fatalf("failed to enqueue: %v", err)
	}

	got, err := queue.GetReport(context.Background(), jobID)
	if err != nil {
		t.Fatalf("failed to get report: %v", err)
	}
	if got.JobID != jobID || got.RequesterID != requesterID || got.Status != "pending" {
		t.Fatalf("unexpected job recovered: %+v", got)
	}
	if got.TotalBatches != 3 {
		t.Fatalf("expected 3 batches, got %d", got.TotalBatches)
	}

	if err := queue.CancelReport(context.Background(), jobID); err != nil {
		t.Fatalf("failed to cancel: %v", err)
	}
	cancelled, err := queue.GetReport(context.Background(), jobID)
	if err != nil {
		t.Fatalf("failed to get report: %v", err)
	}
	if cancelled.Status != "failed" || cancelled.Error == nil || *cancelled.Error != "Cancelled by user" {
		t.Fatalf("job not marked failed on cancel: %+v", cancelled)
	}
	if cancelled.CompletedAt == nil {
		t.Fatal("expected completed_at to be set on cancel")
	}

	if err := queue.CancelReport(context.Background(), jobID); !errors.Is(err, api.ErrReportCannotCancel) {
		t.Fatalf("expected ErrReportCannotCancel, got %v", err)
	}

	queue.StoreResultForTest(jobID, []byte("sheet-content"), "export.xlsx")
	data, filename, err := queue.GetResultData(context.Background(), jobID)
	if err != nil {
		t.Fatalf("unexpected error getting result: %v", err)
	}
	if filename != "export.xlsx" || string(data) != "sheet-content" {
		t.Fatalf("unexpected result: filename=%s data=%s", filename, string(data))
	}
}

func TestRedisReportQueue_UnknownJob(t *testing.T) {
	queue := &RedisReportQueue{}
	missing := uuid.New()

	if _, err := queue.GetReport(context.Background(), missing); !errors.Is(err, api.ErrReportNotFound) {
		t.Fatalf("expected ErrReportNotFound, got %v", err)
	}
	if err := queue.CancelReport(context.Background(), missing); !errors.Is(err, api.ErrReportNotFound) {
		t.Fatalf("expected ErrReportNotFound, got %v", err)
	}
	if _, _, err := queue.GetResultData(context.Background(), missing); !errors.Is(err, api.ErrReportResultNotFound) {
		t.Fatalf("expected ErrReportResultNotFound, got %v", err)
	}
}

func TestRedisReportQueue_CancelRejectsCompletedJob(t *testing.T) {
	queue := &RedisReportQueue{}
	jobID := uuid.New()
	queue.Fallback.Store(jobID.String(), api.ReportJob{JobID: jobID, RequesterID: uuid.New(), Status: "completed"})

	if err := queue.CancelReport(context.Background(), jobID); !errors.Is(err, api.ErrReportCannotCancel) {
		t.Fatalf("expected ErrReportCannotCancel, got %v", err)
	}
}
