package repository

import (
	"archive/zip"
	"context"
	"encoding/json"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"sort"
	"strings"
	"sync"
	"time"

	"github.com/google/uuid"
	goredis "github.com/redis/go-redis/v9"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	redisinfra "github.com/vkaylee/leeattend/apiserver/internal/infra/redis"
)

const reportWorkerQueue = "worker:jobs:high"

var (
	ErrReportWorkerUnavailable = errors.New("report worker dependencies unavailable")
	ErrReportJobMalformed      = errors.New("report worker job payload is malformed")
)

type ReportWorkerRepository interface {
	GetAttendanceReport(context.Context, api.AttendanceReportFilter) (api.PaginatedReportResponse, error)
}

type ReportWorker struct {
	redis        *redisinfra.Client
	repository   ReportWorkerRepository
	done         chan struct{}
	closed       chan struct{}
	startOnce    sync.Once
	shutdownOnce sync.Once
	cancel       context.CancelFunc
}

type reportWorkerEnvelope struct {
	TaskType    string                  `json:"task_type"`
	RequestID   string                  `json:"request_id"`
	ParentJobID string                  `json:"parent_job_id"`
	Request     api.ExportReportRequest `json:"request"`
}

type reportMetadata struct {
	JobID            string  `json:"job_id"`
	RequesterID      string  `json:"requester_id"`
	Status           string  `json:"status"`
	TotalBatches     int     `json:"total_batches"`
	CompletedBatches int     `json:"completed_batches"`
	FailedBatches    int     `json:"failed_batches"`
	ErrorMessage     *string `json:"error_message,omitempty"`
	CreatedAt        string  `json:"created_at"`
	CompletedAt      *string `json:"completed_at,omitempty"`
}

func NewReportWorker(redis *redisinfra.Client, repository ReportWorkerRepository) (*ReportWorker, error) {
	if redis == nil || repository == nil {
		return nil, ErrReportWorkerUnavailable
	}
	return &ReportWorker{redis: redis, repository: repository, done: make(chan struct{}), closed: make(chan struct{})}, nil
}

func (w *ReportWorker) Start(ctx context.Context) {
	if w == nil {
		return
	}
	w.startOnce.Do(func() {
		workerCtx, cancel := context.WithCancel(ctx)
		w.cancel = cancel
		go w.run(workerCtx)
	})
}

func (w *ReportWorker) Shutdown(ctx context.Context) error {
	if w == nil {
		return nil
	}
	w.shutdownOnce.Do(func() {
		close(w.done)
		if w.cancel != nil {
			w.cancel()
		}
	})
	select {
	case <-w.closed:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	}
}

func (w *ReportWorker) run(ctx context.Context) {
	defer close(w.closed)
	client, err := w.redis.Blocking()
	if err != nil || client == nil {
		return
	}
	for {
		result, err := client.BLPop(ctx, time.Second, reportWorkerQueue).Result()
		if err != nil {
			if errors.Is(err, goredis.Nil) || errors.Is(err, context.DeadlineExceeded) {
				select {
				case <-ctx.Done():
					return
				case <-w.done:
					return
				default:
					continue
				}
			}
			select {
			case <-ctx.Done():
				return
			case <-w.done:
				return
			default:
				continue
			}
		}
		if len(result) != 2 {
			continue
		}
		w.process(ctx, client, []byte(result[1]))
	}
}

func (w *ReportWorker) process(ctx context.Context, client *goredis.Client, payload []byte) {
	var envelope reportWorkerEnvelope
	if err := json.Unmarshal(payload, &envelope); err != nil || envelope.TaskType != "attendance_report" {
		return
	}
	jobID, err := uuid.Parse(envelope.RequestID)
	if err != nil || jobID == uuid.Nil {
		return
	}
	meta, err := readReportMetadata(ctx, client, jobID)
	if err != nil {
		return
	}
	if err := writeReportMetadata(ctx, client, jobID, meta, "processing", nil); err != nil {
		return
	}

	reportType := envelope.Request.ReportType
	if reportType == "detail" {
		reportType = "daily"
	}
	ids := chunkUUIDs(envelope.Request.EmployeeIDs, 50)
	if len(ids) == 0 {
		ids = [][]uuid.UUID{nil}
	}
	rows := make([]map[string]any, 0)
	var total int64
	for index, employeeIDs := range ids {
		batchRows, batchTotal, err := w.readReportPages(ctx, reportType, envelope.Request, employeeIDs)
		if err != nil {
			w.fail(ctx, client, jobID, meta, err)
			return
		}
		applyTenureSnapshots(batchRows, envelope.Request.TenureSnapshots)
		rows = append(rows, batchRows...)
		total += batchTotal
		meta.CompletedBatches = index + 1
		if err := writeReportMetadataValue(ctx, client, jobID, meta); err != nil {
			return
		}
		_ = client.Publish(ctx, redisinfra.ChannelReportStream(jobID), fmt.Sprintf(`{"type":"PROGRESS","batch_index":%d,"completed_batches":%d,"total_batches":%d}`, index, meta.CompletedBatches, meta.TotalBatches)).Err()
	}
	data, filename, err := encodeReportRows(rows, envelope.Request.Format, reportType)
	if err != nil {
		w.fail(ctx, client, jobID, meta, err)
		return
	}
	now := time.Now().UTC().Format(time.RFC3339)
	if err := client.Set(ctx, redisinfra.KeyReportResultData(jobID), data, 24*time.Hour).Err(); err != nil {
		w.fail(ctx, client, jobID, meta, err)
		return
	}
	if err := client.Set(ctx, redisinfra.KeyReportResultFilename(jobID), filename, 24*time.Hour).Err(); err != nil {
		w.fail(ctx, client, jobID, meta, err)
		return
	}
	meta.CompletedBatches = meta.TotalBatches
	meta.Status = "completed"
	meta.CompletedAt = &now
	if err := writeReportMetadataValue(ctx, client, jobID, meta); err != nil {
		return
	}
	_ = client.Publish(ctx, redisinfra.ChannelReportStream(jobID), fmt.Sprintf(`{"type":"DONE","total_batches":%d,"total_records":%d}`, meta.TotalBatches, total)).Err()
}

func (w *ReportWorker) readReportPages(ctx context.Context, reportType string, request api.ExportReportRequest, employeeIDs []uuid.UUID) ([]map[string]any, int64, error) {
	const pageSize = 3000
	rows := make([]map[string]any, 0)
	for page := 1; ; page++ {
		report, err := w.repository.GetAttendanceReport(ctx, api.AttendanceReportFilter{
			StartDate:   request.StartDate,
			EndDate:     request.EndDate,
			ReportType:  reportType,
			EmployeeIDs: employeeIDs,
			Page:        page,
			Limit:       pageSize,
		})
		if err != nil {
			return nil, 0, err
		}
		var pageRows []map[string]any
		if err := json.Unmarshal(report.Data, &pageRows); err != nil {
			return nil, 0, err
		}
		rows = append(rows, pageRows...)
		pageTotal := report.Total
		if pageTotal == 0 || len(pageRows) == 0 || int64(page*pageSize) >= pageTotal {
			return rows, int64(len(rows)), nil
		}
	}
}

func (w *ReportWorker) fail(ctx context.Context, client *goredis.Client, jobID uuid.UUID, meta reportMetadata, cause error) {
	message := "report generation failed"
	now := time.Now().UTC().Format(time.RFC3339)
	meta.Status = "failed"
	meta.FailedBatches = meta.TotalBatches
	meta.ErrorMessage = &message
	meta.CompletedAt = &now
	_ = writeReportMetadataValue(ctx, client, jobID, meta)
	_ = client.Publish(ctx, redisinfra.ChannelReportStream(jobID), fmt.Sprintf(`{"type":"ERROR","message":%q,"batch_index":null}`, message)).Err()
}

func readReportMetadata(ctx context.Context, client *goredis.Client, jobID uuid.UUID) (reportMetadata, error) {
	value, err := client.Get(ctx, redisinfra.KeyReportMeta(jobID)).Result()
	if errors.Is(err, goredis.Nil) {
		value, err = client.Get(ctx, "job:metadata:"+jobID.String()).Result()
	}
	if err != nil {
		return reportMetadata{}, err
	}
	var meta reportMetadata
	if err := json.Unmarshal([]byte(value), &meta); err != nil {
		return reportMetadata{}, err
	}
	return meta, nil
}

func writeReportMetadata(ctx context.Context, client *goredis.Client, jobID uuid.UUID, meta reportMetadata, status string, cause error) error {
	meta.Status = status
	if cause != nil {
		message := cause.Error()
		meta.ErrorMessage = &message
	}
	return writeReportMetadataValue(ctx, client, jobID, meta)
}

func writeReportMetadataValue(ctx context.Context, client *goredis.Client, jobID uuid.UUID, meta reportMetadata) error {
	encoded, err := json.Marshal(meta)
	if err != nil {
		return err
	}
	pipe := client.TxPipeline()
	pipe.Set(ctx, redisinfra.KeyReportMeta(jobID), encoded, 24*time.Hour)
	pipe.Set(ctx, "job:metadata:"+jobID.String(), encoded, 24*time.Hour)
	_, err = pipe.Exec(ctx)
	return err
}

func encodeReportRows(rows []map[string]any, format, reportType string) ([]byte, string, error) {
	headers := reportHeaders(rows)
	records := make([][]string, 0, len(rows)+1)
	records = append(records, headers)
	for _, row := range rows {
		record := make([]string, len(headers))
		for i, header := range headers {
			record[i] = fmt.Sprint(row[header])
			if row[header] == nil {
				record[i] = ""
			}
		}
		records = append(records, record)
	}
	return xlsxBytes(records), "attendance-" + reportType + ".xlsx", nil
}

func chunkUUIDs(ids []uuid.UUID, size int) [][]uuid.UUID {
	if len(ids) == 0 {
		return nil
	}
	chunks := make([][]uuid.UUID, 0, (len(ids)+size-1)/size)
	for start := 0; start < len(ids); start += size {
		end := start + size
		if end > len(ids) {
			end = len(ids)
		}
		chunks = append(chunks, ids[start:end])
	}
	return chunks
}

func applyTenureSnapshots(rows []map[string]any, snapshots []api.ExportTenureSnapshot) {
	for _, row := range rows {
		employeeID, ok := row["employee_id"].(string)
		if !ok {
			continue
		}
		date, hasDate := row["date"].(string)
		if !hasDate {
			continue
		}
		bestStart := ""
		bestNumber := ""
		for _, snapshot := range snapshots {
			if snapshot.EmployeeID.String() != employeeID || snapshot.StartDate > date {
				continue
			}
			if snapshot.EndDate != nil && *snapshot.EndDate < date {
				continue
			}
			if snapshot.StartDate >= bestStart {
				bestStart = snapshot.StartDate
				bestNumber = snapshot.EmployeeNumber
			}
		}
		if bestNumber != "" {
			row["employee_id_number"] = bestNumber
		}
	}
}

func reportHeaders(rows []map[string]any) []string {
	set := map[string]struct{}{}
	for _, row := range rows {
		for key := range row {
			set[key] = struct{}{}
		}
	}
	headers := make([]string, 0, len(set))
	for key := range set {
		headers = append(headers, key)
	}
	sort.Strings(headers)
	return headers
}

func xlsxBytes(records [][]string) []byte {
	var output strings.Builder
	files := map[string]string{
		"[Content_Types].xml":        `<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>`,
		"_rels/.rels":                `<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>`,
		"xl/_rels/workbook.xml.rels": `<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>`,
		"xl/workbook.xml":            `<?xml version="1.0" encoding="UTF-8"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Report" sheetId="1" r:id="rId1"/></sheets></workbook>`,
	}
	var sheet strings.Builder
	sheet.WriteString(`<?xml version="1.0" encoding="UTF-8"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>`)
	for rowIndex, row := range records {
		fmt.Fprintf(&sheet, `<row r="%d">`, rowIndex+1)
		for colIndex, value := range row {
			fmt.Fprintf(&sheet, `<c r="%s%d" t="inlineStr"><is><t>%s</t></is></c>`, columnName(colIndex), rowIndex+1, xmlEscape(value))
		}
		sheet.WriteString(`</row>`)
	}
	sheet.WriteString(`</sheetData></worksheet>`)
	files["xl/worksheets/sheet1.xml"] = sheet.String()
	writer := zip.NewWriter(&stringWriter{builder: &output})
	for name, content := range files {
		entry, err := writer.Create(name)
		if err != nil {
			return nil
		}
		_, _ = io.WriteString(entry, content)
	}
	if err := writer.Close(); err != nil {
		return nil
	}
	return []byte(output.String())
}

type stringWriter struct{ builder *strings.Builder }

func (w *stringWriter) Write(data []byte) (int, error) { return w.builder.Write(data) }

func columnName(index int) string {
	name := ""
	for index >= 0 {
		name = string(rune('A'+index%26)) + name
		index = index/26 - 1
	}
	return name
}

func xmlEscape(value string) string {
	encoded, _ := xml.Marshal(value)
	text := string(encoded)
	return strings.TrimPrefix(strings.TrimSuffix(text, `</string>`), `<string>`)
}
