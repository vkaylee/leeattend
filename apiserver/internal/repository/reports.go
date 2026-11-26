package repository

import (
	"archive/zip"
	"context"
	"encoding/json"
	"encoding/xml"
	"fmt"
	"math"
	"sort"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
	"github.com/vkaylee/leeattend/apiserver/internal/domain/workpolicy"
)

type ReportRepository struct {
	DB DBTX
}

const officialScheduleCTE = `WITH latest_official_revisions AS (
		SELECT DISTINCT ON (employment_id, business_date)
			id, employment_id, business_date, state, timezone
		FROM schedule_day_revisions
		WHERE state IN ('published', 'explicit_off')
		ORDER BY employment_id, business_date, revision_number DESC, id DESC
	), official_schedule AS (
	SELECT sdr.id AS revision_id,
		sdr.employment_id,
		sdr.business_date,
		sdr.state,
		sdr.timezone,
		(ARRAY_AGG(ws.scheduled_start::text ORDER BY ws.scheduled_start_at ASC NULLS LAST, ws.sort_order ASC, ws.id ASC) FILTER (WHERE ws.id IS NOT NULL AND ws.segment_status <> 'cancelled'))[1] AS scheduled_start,
		(ARRAY_AGG(ws.scheduled_end::text ORDER BY ws.scheduled_end_at DESC NULLS LAST, ws.sort_order DESC, ws.id DESC) FILTER (WHERE ws.id IS NOT NULL AND ws.segment_status <> 'cancelled'))[1] AS scheduled_end,
		(ARRAY_AGG(st.name ORDER BY ws.scheduled_start_at ASC NULLS LAST, ws.sort_order ASC, ws.id ASC) FILTER (WHERE ws.id IS NOT NULL AND ws.segment_status <> 'cancelled'))[1] AS shift_name
	FROM latest_official_revisions sdr
	LEFT JOIN work_segments ws ON ws.schedule_revision_id = sdr.id
	LEFT JOIN shift_templates st ON st.id = ws.template_id
		GROUP BY sdr.id, sdr.employment_id, sdr.business_date, sdr.state, sdr.timezone
)`

func (r ReportRepository) SnapshotExportRequest(ctx context.Context, req api.ExportReportRequest) (api.ExportReportRequest, error) {
	if len(req.EmployeeIDs) == 0 {
		return req, nil
	}
	db, err := requireDB(r.DB, "report repository")
	if err != nil {
		return req, err
	}
	rows, err := db.Query(ctx, `SELECT t.employee_id, t.employee_number, t.start_date::text, t.end_date::text
		FROM employee_tenures t
		WHERE t.employee_id = ANY($1) AND t.deleted_at IS NULL
		  AND t.start_date <= $3::date AND (t.end_date IS NULL OR t.end_date >= $2::date)
		ORDER BY t.employee_id, t.start_date, t.id`, req.EmployeeIDs, req.StartDate, req.EndDate)
	if err != nil {
		return req, err
	}
	defer rows.Close()
	req.TenureSnapshots = nil
	for rows.Next() {
		var snapshot api.ExportTenureSnapshot
		if err := rows.Scan(&snapshot.EmployeeID, &snapshot.EmployeeNumber, &snapshot.StartDate, &snapshot.EndDate); err != nil {
			return req, err
		}
		req.TenureSnapshots = append(req.TenureSnapshots, snapshot)
	}
	return req, rows.Err()
}

const (
	reportPageSize     = 3000
	maxSyncExportRows  = 100000
	maxSyncExportBytes = 32 << 20
)

func (r ReportRepository) ExportAttendanceReport(ctx context.Context, req api.ExportReportRequest) ([]byte, string, error) {
	pageSize := reportPageSize
	allRows := make([]map[string]any, 0)
	reportType := req.ReportType
	if reportType == "detail" {
		reportType = "daily"
	}
	for page := 1; ; page++ {
		if err := ctx.Err(); err != nil {
			return nil, "", err
		}
		pageReport, err := r.GetAttendanceReport(ctx, api.AttendanceReportFilter{StartDate: req.StartDate, EndDate: req.EndDate, ReportType: reportType, EmployeeIDs: req.EmployeeIDs, Page: page, Limit: pageSize})
		if err != nil {
			return nil, "", err
		}
		var rows []map[string]any
		if err := json.Unmarshal(pageReport.Data, &rows); err != nil {
			return nil, "", err
		}
		allRows = append(allRows, rows...)
		if len(allRows) > maxSyncExportRows || pageReport.Total > maxSyncExportRows {
			return nil, "", api.ErrReportExportTooLarge
		}
		if len(rows) == 0 || int64(page*pageSize) >= pageReport.Total {
			break
		}
	}
	data, filename, err := encodeReportRowsSync(allRows, reportType)
	if err != nil {
		return nil, "", err
	}
	if len(data) > maxSyncExportBytes {
		return nil, "", api.ErrReportExportTooLarge
	}
	return data, filename, nil
}

func encodeReportRowsSync(rows []map[string]any, reportType string) ([]byte, string, error) {
	headers := make([]string, 0)
	seen := map[string]struct{}{}
	for _, row := range rows {
		for key := range row {
			if _, ok := seen[key]; !ok {
				seen[key] = struct{}{}
				headers = append(headers, key)
			}
		}
	}
	sort.Strings(headers)
	records := make([][]string, 0, len(rows)+1)
	records = append(records, headers)
	for _, row := range rows {
		record := make([]string, len(headers))
		for i, header := range headers {
			if row[header] != nil {
				record[i] = fmt.Sprint(row[header])
			}
		}
		records = append(records, record)
	}
	return xlsxBytesSync(records), "attendance-" + reportType + ".xlsx", nil
}

func xlsxBytesSync(records [][]string) []byte {
	var output strings.Builder
	var sheet strings.Builder
	sheet.WriteString(`<?xml version="1.0" encoding="UTF-8"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>`)
	for rowIndex, row := range records {
		fmt.Fprintf(&sheet, `<row r="%d">`, rowIndex+1)
		for colIndex, value := range row {
			fmt.Fprintf(&sheet, `<c r="%s%d" t="inlineStr"><is><t>%s</t></is></c>`, columnNameSync(colIndex), rowIndex+1, xmlEscapeSync(value))
		}
		sheet.WriteString(`</row>`)
	}
	sheet.WriteString(`</sheetData></worksheet>`)
	files := map[string]string{
		"[Content_Types].xml":        `<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/><Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/></Types>`,
		"_rels/.rels":                `<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>`,
		"xl/_rels/workbook.xml.rels": `<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/></Relationships>`,
		"xl/workbook.xml":            `<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets><sheet name="Report" sheetId="1" r:id="rId1"/></sheets></workbook>`,
		"xl/worksheets/sheet1.xml":   sheet.String(),
	}
	zw := zip.NewWriter(&stringWriterSync{builder: &output})
	for name, content := range files {
		f, _ := zw.Create(name)
		_, _ = f.Write([]byte(content))
	}
	_ = zw.Close()
	return []byte(output.String())
}

func columnNameSync(index int) string {
	name := ""
	for index >= 0 {
		name = string(rune('A'+index%26)) + name
		index = index/26 - 1
	}
	return name
}

func xmlEscapeSync(value string) string {
	encoded, _ := xml.Marshal(value)
	text := string(encoded)
	return text[1 : len(text)-1]
}

type stringWriterSync struct{ builder *strings.Builder }

func (w *stringWriterSync) Write(p []byte) (int, error) { return w.builder.Write(p) }

func (r ReportRepository) ResolveBehalfEmployeeIDs(ctx context.Context, userID uuid.UUID) ([]uuid.UUID, error) {
	db, err := requireDB(r.DB, "report repository")
	if err != nil {
		return nil, err
	}
	rows, err := db.Query(ctx, `SELECT DISTINCT e.id
FROM behalf_assignments ba
JOIN employment_segments emp ON (
    (ba.target_employee_id IS NOT NULL AND emp.employee_id = ba.target_employee_id)
    OR (ba.target_department_id IS NOT NULL AND emp.department_id = ba.target_department_id)
)
JOIN employees e ON e.id = emp.employee_id
WHERE ba.assignee_user_id = $1
  AND ba.assignment_type = 'overtime_claim'`, userID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	ids := make([]uuid.UUID, 0)
	for rows.Next() {
		var id uuid.UUID
		if err := rows.Scan(&id); err != nil {
			return nil, err
		}
		ids = append(ids, id)
	}
	return ids, rows.Err()
}

func appendEmployeeStatusFilter(clause string, args []any, argIndex int, status *string) (string, []any, int) {
	if status == nil || *status == "" || *status == "all" {
		return clause, args, argIndex
	}
	return clause + fmt.Sprintf(" AND emp.status = $%d", argIndex), append(args, *status), argIndex + 1
}

func (r ReportRepository) GetAttendanceReport(ctx context.Context, params api.AttendanceReportFilter) (api.PaginatedReportResponse, error) {
	db, err := requireDB(r.DB, "report repository")
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	if params.EmploymentID != nil {
		var exists bool
		if err := db.QueryRow(ctx, `SELECT EXISTS (SELECT 1 FROM employment_segments WHERE id = $1 AND deleted_at IS NULL)`, *params.EmploymentID).Scan(&exists); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		if !exists {
			return api.PaginatedReportResponse{}, api.ErrReportNotFound
		}
	}

	offset := (params.Page - 1) * params.Limit

	if params.ReportType == "daily" {
		whereClause := `emp.deleted_at IS NULL AND e.deleted_at IS NULL AND ads.date >= $1::date AND ads.date <= $2::date`
		filterClause := ""
		args := []any{params.StartDate, params.EndDate}
		argIndex := 3

		if len(params.EmployeeIDs) > 0 {
			filterClause += fmt.Sprintf(" AND e.id = ANY($%d)", argIndex)
			args = append(args, params.EmployeeIDs)
			argIndex++
		}
		if params.DepartmentID != nil {
			filterClause += fmt.Sprintf(" AND emp.department_id = $%d", argIndex)
			args = append(args, *params.DepartmentID)
			argIndex++
		}
		if params.EmployerID != nil {
			filterClause += fmt.Sprintf(" AND emp.employer_id = $%d", argIndex)
			args = append(args, *params.EmployerID)
			argIndex++
		}
		if params.EmploymentID != nil {
			filterClause += fmt.Sprintf(" AND emp.id = $%d", argIndex)
			args = append(args, *params.EmploymentID)
			argIndex++
		}
		filterClause, args, argIndex = appendEmployeeStatusFilter(filterClause, args, argIndex, params.EmployeeStatus)

		var total int64
		countQuery := fmt.Sprintf(`SELECT count(*) FROM employment_segments emp JOIN employees e ON e.id = emp.employee_id JOIN attendance_daily_summaries ads ON ads.employment_id = emp.id LEFT JOIN employment_relationships er ON er.id = emp.relationship_id AND er.employee_id = emp.employee_id AND er.deleted_at IS NULL LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= ads.date AND (t.end_date IS NULL OR t.end_date >= ads.date) WHERE %s%s`, whereClause, filterClause)
		if err := db.QueryRow(ctx, countQuery, args...).Scan(&total); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		dataQuery := fmt.Sprintf(officialScheduleCTE+` SELECT ads.date::text, emp.id AS employment_id, e.id AS employee_id, e.full_name, COALESCE(t.employee_number, ''),
       os.shift_name, os.scheduled_start, os.scheduled_end,
       te.punch_in_time::text, te.punch_out_time::text,
       COALESCE(ads.actual_work_minutes, 0),
       COALESCE(ads.scheduled_break_minutes, 0),
       COALESCE(ads.actual_break_minutes, 0),
       COALESCE(ads.deducted_break_minutes, 0),
       COALESCE(ads.net_work_minutes, ads.actual_work_minutes, 0),
       COALESCE(ads.net_paid_minutes, ads.net_work_minutes, ads.actual_work_minutes, 0),
       COALESCE(ads.break_provisional, false),
       ads.break_exception_code,
       COALESCE(ads.standard_minutes, 0),
       COALESCE(ads.approved_ot_minutes, 0),
       COALESCE(ads.ot_status::text, 'none')
FROM employment_segments emp
JOIN employees e ON e.id = emp.employee_id
JOIN attendance_daily_summaries ads ON ads.employment_id = emp.id
LEFT JOIN employment_relationships er ON er.id = emp.relationship_id AND er.employee_id = emp.employee_id AND er.deleted_at IS NULL
LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= ads.date AND (t.end_date IS NULL OR t.end_date >= ads.date)
LEFT JOIN official_schedule os ON os.employment_id = emp.id AND os.business_date = ads.date
LEFT JOIN LATERAL (
       SELECT MIN(te.punch_in_time)::text AS punch_in_time, MAX(te.punch_out_time)::text AS punch_out_time
       FROM time_entries te
       LEFT JOIN work_segments te_ws ON te_ws.id = te.segment_id
       WHERE te.employment_id = emp.id
         AND te.entry_type = 'work'
         AND (
             (os.revision_id IS NOT NULL AND te_ws.schedule_revision_id = os.revision_id)
             OR (te.segment_id IS NOT NULL AND te_ws.date = ads.date AND (os.revision_id IS NULL OR te_ws.schedule_revision_id IS NULL))
             OR (te.segment_id IS NULL AND (te.punch_in_time AT TIME ZONE COALESCE(NULLIF(os.timezone, ''), 'UTC'))::date = ads.date)
         )
) te ON TRUE
WHERE %s%s
ORDER BY ads.date ASC, e.full_name ASC
LIMIT $%d OFFSET $%d`, whereClause, filterClause, argIndex, argIndex+1)

		args = append(args, params.Limit, offset)
		rows, err := db.Query(ctx, dataQuery, args...)
		if err != nil {
			return api.PaginatedReportResponse{}, err
		}
		defer rows.Close()

		type dailyPending struct {
			row     api.DailyAttendanceRow
			stdMins int
			otMins  int
		}
		pending := make([]dailyPending, 0)
		empSet := make(map[uuid.UUID]struct{})
		empIDs := make([]uuid.UUID, 0)
		var minDate, maxDate time.Time
		for rows.Next() {
			var row api.DailyAttendanceRow
			var actMins, scheduledBreakMins, actualBreakMins, deductedBreakMins, netWorkMins, netPaidMins, stdMins, otMins int
			var breakProvisional bool
			var breakExceptionCode *string
			var shiftName, schedStart, schedEnd, punchIn, punchOut *string
			var otStatus string
			if err := rows.Scan(&row.Date, &row.EmploymentID, &row.EmployeeID, &row.EmployeeName, &row.EmployeeIDNumber,
				&shiftName, &schedStart, &schedEnd, &punchIn, &punchOut,
				&actMins, &scheduledBreakMins, &actualBreakMins, &deductedBreakMins, &netWorkMins, &netPaidMins, &breakProvisional, &breakExceptionCode, &stdMins, &otMins, &otStatus); err != nil {
				return api.PaginatedReportResponse{}, err
			}
			row.ShiftName = shiftName
			row.ScheduledStart = schedStart
			row.ScheduledEnd = schedEnd
			row.ActualPunchIn = punchIn
			row.ActualPunchOut = punchOut
			row.ActualWorkMinutes = actMins
			row.ScheduledBreakMinutes = scheduledBreakMins
			row.ActualBreakMinutes = actualBreakMins
			row.DeductedBreakMinutes = deductedBreakMins
			row.NetWorkMinutes = netWorkMins
			row.NetPaidMinutes = netPaidMins
			row.BreakProvisional = breakProvisional
			row.BreakExceptionCode = breakExceptionCode
			row.ComputedStdHours = math.Round((float64(stdMins)/60.0)*100) / 100
			row.ComputedOTHours = math.Round((float64(otMins)/60.0)*100) / 100
			row.OTClaimStatus = otStatus
			if actMins > 0 || stdMins > 0 || (punchIn != nil && punchOut != nil) {
				row.Status = "present"
			} else {
				row.Status = "absent"
			}
			if _, seen := empSet[row.EmploymentID]; !seen {
				empSet[row.EmploymentID] = struct{}{}
				empIDs = append(empIDs, row.EmploymentID)
			}
			if d, parseErr := time.Parse("2006-01-02", row.Date); parseErr == nil {
				if minDate.IsZero() || d.Before(minDate) {
					minDate = d
				}
				if maxDate.IsZero() || d.After(maxDate) {
					maxDate = d
				}
			}
			pending = append(pending, dailyPending{row: row, stdMins: stdMins, otMins: otMins})
		}
		if err := rows.Err(); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		rows.Close()

		policiesByEmp := make(map[uuid.UUID][]workpolicy.ResolvedPolicy)
		if len(empIDs) > 0 && !minDate.IsZero() {
			policiesByEmp, err = (WorkPolicyResolver{}).ResolveRangeForEmployments(ctx, db, empIDs, minDate, maxDate)
			if err != nil {
				return api.PaginatedReportResponse{}, err
			}
		}
		results := make([]api.DailyAttendanceRow, 0, len(pending))
		for _, item := range pending {
			row := item.row
			denominator := 480.0
			minutesPerCredit := 0.0
			if d, parseErr := time.Parse("2006-01-02", row.Date); parseErr == nil {
				if policy, ok := FindResolvedPolicyForDate(policiesByEmp[row.EmploymentID], d); ok && policy.MinutesPerCredit > 0 {
					minutesPerCredit = float64(policy.MinutesPerCredit)
					denominator = minutesPerCredit
				}
			}
			row.StdWorkUnits = math.Round((float64(item.stdMins)/denominator)*100) / 100
			row.OTWorkUnits = math.Round((float64(item.otMins)/denominator)*100) / 100
			if minutesPerCredit > 0 {
				row.StandardCredits = math.Round((float64(item.stdMins)/minutesPerCredit)*100) / 100
			}
			results = append(results, row)
		}
		if err := rows.Err(); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		raw, err := json.Marshal(results)
		if err != nil {
			return api.PaginatedReportResponse{}, err
		}
		return api.PaginatedReportResponse{
			Data:  raw,
			Total: total,
			Page:  params.Page,
			Limit: params.Limit,
		}, nil
	}

	baseClause := `emp.deleted_at IS NULL AND e.deleted_at IS NULL
		AND emp.start_date <= $2::date
		AND (emp.end_date IS NULL OR emp.end_date >= $1::date)`
	countFilterClause := ""
	countArgs := []any{params.StartDate, params.EndDate}
	countArgIndex := 3
	dataFilterClause := ""
	dataArgs := []any{params.StartDate, params.EndDate}
	dataArgIndex := 3

	if len(params.EmployeeIDs) > 0 {
		countFilterClause += fmt.Sprintf(" AND e.id = ANY($%d)", countArgIndex)
		countArgs = append(countArgs, params.EmployeeIDs)
		countArgIndex++
		dataFilterClause += fmt.Sprintf(" AND e.id = ANY($%d)", dataArgIndex)
		dataArgs = append(dataArgs, params.EmployeeIDs)
		dataArgIndex++
	}
	if params.DepartmentID != nil {
		countFilterClause += fmt.Sprintf(" AND emp.department_id = $%d", countArgIndex)
		countArgs = append(countArgs, *params.DepartmentID)
		countArgIndex++
		dataFilterClause += fmt.Sprintf(" AND emp.department_id = $%d", dataArgIndex)
		dataArgs = append(dataArgs, *params.DepartmentID)
		dataArgIndex++
	}
	if params.EmployerID != nil {
		countFilterClause += fmt.Sprintf(" AND emp.employer_id = $%d", countArgIndex)
		countArgs = append(countArgs, *params.EmployerID)
		countArgIndex++
		dataFilterClause += fmt.Sprintf(" AND emp.employer_id = $%d", dataArgIndex)
		dataArgs = append(dataArgs, *params.EmployerID)
		dataArgIndex++
	}
	if params.EmploymentID != nil {
		countFilterClause += fmt.Sprintf(" AND emp.id = $%d", countArgIndex)
		countArgs = append(countArgs, *params.EmploymentID)
		dataFilterClause += fmt.Sprintf(" AND emp.id = $%d", dataArgIndex)
		dataArgs = append(dataArgs, *params.EmploymentID)
		dataArgIndex++
	}
	countFilterClause, countArgs, _ = appendEmployeeStatusFilter(countFilterClause, countArgs, countArgIndex, params.EmployeeStatus)
	dataFilterClause, dataArgs, dataArgIndex = appendEmployeeStatusFilter(dataFilterClause, dataArgs, dataArgIndex, params.EmployeeStatus)

	var total int64
	countQuery := fmt.Sprintf(`SELECT count(DISTINCT (emp.id, t.id)) FROM employment_segments emp JOIN employees e ON e.id = emp.employee_id LEFT JOIN employment_relationships er ON er.id = emp.relationship_id AND er.employee_id = emp.employee_id AND er.deleted_at IS NULL LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= $2::date AND (t.end_date IS NULL OR t.end_date >= $1::date) WHERE %s%s`, baseClause, countFilterClause)
	if err := db.QueryRow(ctx, countQuery, countArgs...).Scan(&total); err != nil {
		return api.PaginatedReportResponse{}, err
	}

	dataQuery := fmt.Sprintf(`SELECT emp.id AS employment_id, e.id AS employee_id, e.full_name, COALESCE(t.employee_number, ''),
       COALESCE(SUM(ads.standard_minutes), 0)::int AS total_std_minutes,
       COALESCE(SUM(ads.approved_ot_minutes), 0)::int AS total_ot_minutes,
       COALESCE(json_agg(json_build_object('date', ads.date::text, 'standard_minutes', ads.standard_minutes)) FILTER (WHERE ads.id IS NOT NULL), '[]'::json) AS daily_entries,
       emp.employer_id, em.name AS employer_name
FROM employment_segments emp
JOIN employees e ON e.id = emp.employee_id
LEFT JOIN employment_relationships er ON er.id = emp.relationship_id AND er.employee_id = emp.employee_id AND er.deleted_at IS NULL
LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= $2::date AND (t.end_date IS NULL OR t.end_date >= $1::date)
LEFT JOIN employers em ON em.id = emp.employer_id
LEFT JOIN attendance_daily_summaries ads ON ads.employment_id = emp.id AND ads.date >= $1::date AND ads.date <= $2::date
		WHERE %s%s
		GROUP BY emp.id, e.id, e.full_name, COALESCE(t.employee_number, ''), emp.employer_id, em.name
		ORDER BY e.full_name ASC
		LIMIT $%d OFFSET $%d`, baseClause, dataFilterClause, dataArgIndex, dataArgIndex+1)

	dataArgs = append(dataArgs, params.Limit, offset)
	rows, err := db.Query(ctx, dataQuery, dataArgs...)
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	defer rows.Close()

	type summaryDailyEntry struct {
		Date            string `json:"date"`
		StandardMinutes int    `json:"standard_minutes"`
	}
	type summaryPending struct {
		row          api.AttendanceSummaryRow
		totalStdMins int
		totalOTMins  int
		dailyRaw     []byte
	}
	pending := make([]summaryPending, 0)
	empIDs := make([]uuid.UUID, 0)
	for rows.Next() {
		var item summaryPending
		item.row.UnitType = "DAY"
		if err := rows.Scan(&item.row.EmploymentID, &item.row.EmployeeID, &item.row.EmployeeName, &item.row.EmployeeIDNumber,
			&item.totalStdMins, &item.totalOTMins, &item.dailyRaw,
			&item.row.EmployerID, &item.row.EmployerName); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		empIDs = append(empIDs, item.row.EmploymentID)
		pending = append(pending, item)
	}
	if err := rows.Err(); err != nil {
		return api.PaginatedReportResponse{}, err
	}
	rows.Close()

	startDate, startErr := time.Parse("2006-01-02", params.StartDate)
	endDate, endErr := time.Parse("2006-01-02", params.EndDate)
	policiesByEmp := make(map[uuid.UUID][]workpolicy.ResolvedPolicy)
	if len(empIDs) > 0 && startErr == nil && endErr == nil {
		policiesByEmp, err = (WorkPolicyResolver{}).ResolveRangeForEmployments(ctx, db, empIDs, startDate, endDate)
		if err != nil {
			return api.PaginatedReportResponse{}, err
		}
	}

	summaries := make([]api.AttendanceSummaryRow, 0, len(pending))
	for _, item := range pending {
		row := item.row
		empPolicies := policiesByEmp[row.EmploymentID]
		hoursPerDay := 8.0
		if startErr == nil {
			if p, ok := FindResolvedPolicyForDate(empPolicies, startDate); ok && p.MinutesPerCredit > 0 {
				hoursPerDay = math.Round((float64(p.MinutesPerCredit)/60.0)*100) / 100
			} else if len(empPolicies) > 0 && empPolicies[0].MinutesPerCredit > 0 {
				hoursPerDay = math.Round((float64(empPolicies[0].MinutesPerCredit)/60.0)*100) / 100
			}
		}
		denomMins := hoursPerDay * 60.0
		if denomMins <= 0 {
			denomMins = 480.0
		}
		row.UnitCoefficientHours = hoursPerDay
		row.TotalStdUnits = int(math.Floor(float64(item.totalStdMins) / denomMins))
		row.TotalOTUnits = int(math.Floor(float64(item.totalOTMins) / denomMins))
		row.ComputedStdHours = math.Round((float64(item.totalStdMins)/60.0)*100) / 100
		row.ComputedOTHours = math.Round((float64(item.totalOTMins)/60.0)*100) / 100

		var entries []summaryDailyEntry
		if len(item.dailyRaw) > 0 {
			_ = json.Unmarshal(item.dailyRaw, &entries)
		}
		creditsTotal := 0.0
		for _, entry := range entries {
			if d, parseErr := time.Parse("2006-01-02", entry.Date); parseErr == nil {
				if p, ok := FindResolvedPolicyForDate(empPolicies, d); ok && p.MinutesPerCredit > 0 {
					creditsTotal += float64(entry.StandardMinutes) / float64(p.MinutesPerCredit)
				}
			}
		}
		row.StandardCredits = math.Round(creditsTotal*100) / 100
		summaries = append(summaries, row)
	}
	if err := rows.Err(); err != nil {
		return api.PaginatedReportResponse{}, err
	}
	raw, err := json.Marshal(summaries)
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	return api.PaginatedReportResponse{
		Data:  raw,
		Total: total,
		Page:  params.Page,
		Limit: params.Limit,
	}, nil
}

func (r ReportRepository) GetNoShowReport(ctx context.Context, params api.NoShowReportFilter) (api.PaginatedReportResponse, error) {
	db, err := requireDB(r.DB, "report repository")
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	offset := (params.Page - 1) * params.Limit
	noShowCTE := officialScheduleCTE + `, missing_official_segments AS (
		SELECT sdr.id AS revision_id,
		       sdr.employment_id,
		       ws.id AS segment_id,
		       ws.sort_order,
		       ws.scheduled_start::text AS scheduled_start,
		       ws.scheduled_end::text AS scheduled_end,
		       st.name AS shift_name
		FROM latest_official_revisions sdr
		JOIN work_segments ws ON ws.schedule_revision_id = sdr.id
		LEFT JOIN shift_templates st ON st.id = ws.template_id
		WHERE sdr.business_date = $1::date
		  AND sdr.state = 'published'
		  AND ws.segment_status <> 'cancelled'
		  AND NOT EXISTS (
		      SELECT 1 FROM time_entries te
		      WHERE te.employment_id = sdr.employment_id
		        AND te.entry_type = 'work'
		        AND (
		            te.segment_id = ws.id
		            OR (te.segment_id IS NULL AND ws.scheduled_start_at IS NOT NULL AND ws.scheduled_end_at IS NOT NULL
		                AND te.punch_in_time >= ws.scheduled_start_at - INTERVAL '2 hours'
		                AND te.punch_in_time <= ws.scheduled_end_at)
		            OR (te.segment_id IS NULL AND (ws.scheduled_start_at IS NULL OR ws.scheduled_end_at IS NULL)
		                AND (te.punch_in_time AT TIME ZONE COALESCE(NULLIF(sdr.timezone, ''), 'UTC'))::date = $1::date)
		        )
		  )
		  AND NOT EXISTS (
		      SELECT 1 FROM leave_requests lr
		      WHERE lr.employment_id = sdr.employment_id
		        AND lr.status IN ('approved', 'pre_approved')
		        AND lr.start_date <= $1::date AND lr.end_date >= $1::date
		        AND (
		            lr.duration_type IS NULL
		            OR lr.duration_type = 'full_day'
		            OR (lr.duration_type = 'custom_hours' AND lr.start_time IS NOT NULL AND lr.end_time IS NOT NULL
		                AND ws.scheduled_start >= lr.start_time AND ws.scheduled_end <= lr.end_time)
		        )
		  )
	)`
	whereClause := `emp.start_date <= $1::date AND (emp.end_date IS NULL OR emp.end_date >= $1::date)
	AND emp.deleted_at IS NULL AND e.deleted_at IS NULL
	AND (os.state IS NULL OR os.state <> 'explicit_off')
	AND NOT EXISTS (
		SELECT 1 FROM leave_requests lr
		WHERE lr.employment_id = emp.id
		  AND lr.status IN ('approved', 'pre_approved')
		  AND lr.start_date <= $1::date AND lr.end_date >= $1::date
		  AND (lr.duration_type IS NULL OR lr.duration_type = 'full_day')
	)
	AND (
		(os.state = 'published' AND mos.segment_id IS NOT NULL)
		OR (
			os.state IS NULL
			AND NOT EXISTS (SELECT 1 FROM time_entries te WHERE te.employment_id = emp.id AND te.entry_type = 'work' AND (te.punch_in_time AT TIME ZONE COALESCE(NULLIF(wl.timezone, ''), 'UTC'))::date = $1::date)
		)
	)`
	args := []any{params.Date}
	idx := 2

	if len(params.EmployeeIDs) > 0 {
		whereClause += fmt.Sprintf(" AND e.id = ANY($%d)", idx)
		args = append(args, params.EmployeeIDs)
		idx++
	}
	if params.DepartmentID != nil {
		whereClause += fmt.Sprintf(" AND emp.department_id = $%d", idx)
		args = append(args, *params.DepartmentID)
		idx++
	}

	fromClause := noShowCTE + ` SELECT `
	joins := `
FROM employment_segments emp
JOIN employees e ON e.id = emp.employee_id
LEFT JOIN work_locations wl ON wl.id = emp.primary_location_id
LEFT JOIN employment_relationships er ON er.id = emp.relationship_id AND er.employee_id = emp.employee_id AND er.deleted_at IS NULL
	LEFT JOIN employee_tenures t ON t.id = er.tenure_id AND t.employee_id = e.id AND t.deleted_at IS NULL AND t.start_date <= $1::date AND (t.end_date IS NULL OR t.end_date >= $1::date)
	LEFT JOIN official_schedule os ON os.employment_id = emp.id AND os.business_date = $1::date
	LEFT JOIN missing_official_segments mos ON mos.revision_id = os.revision_id`

	var total int64
	countQuery := noShowCTE + ` SELECT count(*)` + joins + `
WHERE ` + whereClause
	if err := db.QueryRow(ctx, countQuery, args...).Scan(&total); err != nil {
		return api.PaginatedReportResponse{}, err
	}

	dataQuery := fromClause + `os.revision_id, emp.id, e.id, e.full_name, COALESCE(t.employee_number, ''),
       COALESCE(mos.shift_name, os.shift_name), COALESCE(mos.scheduled_start, os.scheduled_start), COALESCE(mos.scheduled_end, os.scheduled_end), $1::date::text, os.state` + joins + fmt.Sprintf(`
WHERE %s
ORDER BY e.full_name ASC, emp.id ASC, mos.sort_order ASC NULLS FIRST, mos.segment_id ASC NULLS FIRST
LIMIT $%d OFFSET $%d`, whereClause, idx, idx+1)

	args = append(args, params.Limit, offset)
	rows, err := db.Query(ctx, dataQuery, args...)
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	defer rows.Close()

	results := make([]api.NoShowReportRow, 0)
	for rows.Next() {
		var revisionID *uuid.UUID
		var row api.NoShowReportRow
		var shiftName, schedStart, schedEnd, scheduleState *string
		if err := rows.Scan(&revisionID, &row.EmploymentID, &row.EmployeeID, &row.EmployeeName, &row.EmployeeIDNumber,
			&shiftName, &schedStart, &schedEnd, &row.Date, &scheduleState); err != nil {
			return api.PaginatedReportResponse{}, err
		}
		row.ShiftName = shiftName
		row.ScheduledStart = schedStart
		row.ScheduledEnd = schedEnd
		row.Status = "absent"
		if scheduleState == nil {
			row.RecordingIssue = "missing_schedule"
		} else {
			row.RecordingIssue = "missing_punch_in"
		}
		results = append(results, row)
	}
	if err := rows.Err(); err != nil {
		return api.PaginatedReportResponse{}, err
	}
	raw, err := json.Marshal(results)
	if err != nil {
		return api.PaginatedReportResponse{}, err
	}
	return api.PaginatedReportResponse{
		Data:  raw,
		Total: total,
		Page:  params.Page,
		Limit: params.Limit,
	}, nil
}
