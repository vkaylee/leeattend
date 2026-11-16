//go:build integration

package repository

import (
	"context"
	"encoding/json"
	"os"
	"testing"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	redisinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
)

type reportWorkerIntegrationRepository struct{}

func (reportWorkerIntegrationRepository) GetAttendanceReport(context.Context, api.AttendanceReportFilter) (api.PaginatedReportResponse, error) {
	return api.PaginatedReportResponse{
		Data:  json.RawMessage(`[ {"date":"2026-06-15","employee_id":"00000000-0000-0000-0000-000000000101","employee_id_number":"current-1541","status":"present"} ]`),
		Total: 1,
	}, nil
}

func TestIntegrationReportWorkerConsumesRedisJobAndStoresResult(t *testing.T) {
	if os.Getenv("LEEATTEND_DB_INTEGRATION") != "1" {
		t.Fatal("LEEATTEND_DB_INTEGRATION=1 is required for integration tests")
	}
	url := os.Getenv("REDIS_URL")
	if url == "" {
		url = "redis://redis:6379"
	}
	client, err := redisinfra.NewClient(redisinfra.Config{URL: url})
	if err != nil {
		t.Fatalf("create redis client: %v", err)
	}
	defer func() { _ = client.Close() }()
	shared, err := client.Shared()
	if err != nil {
		t.Fatalf("get redis client: %v", err)
	}

	jobID := uuid.New()
	meta := reportMetadata{JobID: jobID.String(), RequesterID: uuid.New().String(), Status: "pending", TotalBatches: 1, CreatedAt: time.Now().UTC().Format(time.RFC3339)}
	encoded, err := json.Marshal(meta)
	if err != nil {
		t.Fatal(err)
	}
	if err := shared.Set(context.Background(), redisinfra.KeyReportMeta(jobID), encoded, time.Minute).Err(); err != nil {
		t.Fatalf("seed metadata: %v", err)
	}

	worker, err := NewReportWorker(client, reportWorkerIntegrationRepository{})
	if err != nil {
		t.Fatalf("create report worker: %v", err)
	}
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	worker.Start(ctx)

	payload, err := json.Marshal(reportWorkerEnvelope{
		TaskType:  "attendance_report",
		RequestID: jobID.String(),
		Request: api.ExportReportRequest{
			ReportType: "detail",
			StartDate:  "2026-06-01",
			EndDate:    "2026-06-30",
			Format:     "csv",
			TenureSnapshots: []api.ExportTenureSnapshot{{
				EmployeeID: uuid.MustParse("00000000-0000-0000-0000-000000000101"), EmployeeNumber: "old-1541", StartDate: "2026-01-01",
			}},
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	if err := shared.RPush(context.Background(), reportWorkerQueue, payload).Err(); err != nil {
		t.Fatalf("enqueue redis job: %v", err)
	}

	deadline := time.Now().Add(10 * time.Second)
	for time.Now().Before(deadline) {
		value, getErr := shared.Get(context.Background(), redisinfra.KeyReportMeta(jobID)).Result()
		if getErr == nil {
			var got reportMetadata
			if json.Unmarshal([]byte(value), &got) == nil && got.Status == "completed" {
				data, resultErr := shared.Get(context.Background(), redisinfra.KeyReportResultData(jobID)).Result()
				if resultErr != nil || !contains(data, "old-1541") {
					t.Fatalf("result = %q, err=%v", data, resultErr)
				}
				shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), 3*time.Second)
				defer shutdownCancel()
				if err := worker.Shutdown(shutdownCtx); err != nil {
					t.Fatalf("shutdown worker: %v", err)
				}
				return
			}
		}
		time.Sleep(50 * time.Millisecond)
	}
	t.Fatal("report worker did not complete redis job")
}

func contains(value, fragment string) bool {
	for i := 0; i+len(fragment) <= len(value); i++ {
		if value[i:i+len(fragment)] == fragment {
			return true
		}
	}
	return false
}
