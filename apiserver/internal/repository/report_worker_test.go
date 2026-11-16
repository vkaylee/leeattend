package repository

import (
	"context"
	"encoding/json"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestReportWorkerChunkUUIDs(t *testing.T) {
	ids := make([]uuid.UUID, 51)
	for i := range ids {
		ids[i] = uuid.New()
	}
	chunks := chunkUUIDs(ids, 50)
	if len(chunks) != 2 || len(chunks[0]) != 50 || len(chunks[1]) != 1 {
		t.Fatalf("chunks = %d/%d/%d, want 2/50/1", len(chunks), len(chunks[0]), len(chunks[1]))
	}
}

func TestReportWorkerAppliesDateEffectiveTenureSnapshot(t *testing.T) {
	employeeID := uuid.New()
	end := "2026-06-30"
	rows := []map[string]any{
		{"employee_id": employeeID.String(), "date": "2026-06-15", "employee_id_number": "current"},
		{"employee_id": employeeID.String(), "date": "2026-07-15", "employee_id_number": "current"},
	}
	applyTenureSnapshots(rows, []api.ExportTenureSnapshot{
		{EmployeeID: employeeID, EmployeeNumber: "old-1541", StartDate: "2026-01-01", EndDate: &end},
		{EmployeeID: employeeID, EmployeeNumber: "new-1541", StartDate: "2026-07-01"},
	})
	if rows[0]["employee_id_number"] != "old-1541" || rows[1]["employee_id_number"] != "new-1541" {
		t.Fatalf("numbers = %v/%v, want old-1541/new-1541", rows[0]["employee_id_number"], rows[1]["employee_id_number"])
	}
}

func TestReportWorkerReadsAllReportPages(t *testing.T) {
	calls := 0
	worker := &ReportWorker{repository: reportWorkerPageRepository{calls: &calls}}
	rows, total, err := worker.readReportPages(context.Background(), "daily", api.ExportReportRequest{StartDate: "2026-01-01", EndDate: "2026-01-31"}, nil)
	if err != nil {
		t.Fatalf("read report pages: %v", err)
	}
	if calls != 2 || len(rows) != 3001 || total != 3001 {
		t.Fatalf("calls/rows/total = %d/%d/%d, want 2/3001/3001", calls, len(rows), total)
	}
}

type reportWorkerPageRepository struct {
	calls *int
}

func (r reportWorkerPageRepository) GetAttendanceReport(_ context.Context, filter api.AttendanceReportFilter) (api.PaginatedReportResponse, error) {
	(*r.calls)++
	count := 3000
	if filter.Page == 2 {
		count = 1
	}
	rows := make([]map[string]any, count)
	for i := range rows {
		rows[i] = map[string]any{"row": i}
	}
	data, err := json.Marshal(rows)
	return api.PaginatedReportResponse{Data: data, Total: 3001, Page: filter.Page, Limit: filter.Limit}, err
}

func TestReportWorkerCSVOutputPreservesRows(t *testing.T) {
	data, filename, err := encodeReportRows([]map[string]any{{"employee_id_number": "old-1541", "status": "present"}}, "csv", "daily")
	if err != nil {
		t.Fatalf("encode report: %v", err)
	}
	if filename != "attendance-daily.csv" || !strings.Contains(string(data), "old-1541") {
		t.Fatalf("filename/data = %q/%q", filename, data)
	}
	var rows []map[string]any
	if err := json.Unmarshal([]byte(`[{"employee_id_number":"old-1541"}]`), &rows); err != nil {
		t.Fatal(err)
	}
}
