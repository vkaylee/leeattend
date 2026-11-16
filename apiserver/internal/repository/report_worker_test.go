package repository

import (
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
