package api

import (
	"archive/zip"
	"bytes"
	"encoding/xml"
	"errors"
	"fmt"
	"io"
	"math"
	"path"
	"strconv"
	"strings"
	"time"
)

const maxEmployeeImportBytes int64 = 10 * 1024 * 1024

var errInvalidExcel = errors.New("invalid Excel workbook")
var errUnsupportedXLS = errors.New("binary XLS workbooks are not supported by this parser")

type excelCell struct {
	ref   string
	style int
	kind  string
	value string
}
type excelRow struct {
	number int
	cells  []excelCell
}
type excelSheet struct {
	name string
	rows []excelRow
}
type excelWorkbook struct {
	sheets     []excelSheet
	shared     []string
	dateStyles map[int]bool
}

type xmlWorkbook struct {
	Sheets []xmlSheet `xml:"sheets>sheet"`
}
type xmlSheet struct {
	Name string `xml:"name,attr"`
	RID  string `xml:"http://schemas.openxmlformats.org/officeDocument/2006/relationships id,attr"`
}
type xmlRels struct {
	Relationships []xmlRel `xml:"Relationship"`
}
type xmlRel struct {
	ID     string `xml:"Id,attr"`
	Target string `xml:"Target,attr"`
}
type xmlSST struct {
	Items []xmlSI `xml:"si"`
}
type xmlSI struct {
	Text []string `xml:"t"`
	Runs []xmlRun `xml:"r"`
}
type xmlRun struct {
	Text []string `xml:"t"`
}
type xmlStyle struct {
	XFs     []xmlXF     `xml:"cellXfs>xf"`
	NumFmts []xmlNumFmt `xml:"numFmts>numFmt"`
}
type xmlXF struct {
	NumFmtID int `xml:"numFmtId,attr"`
}
type xmlNumFmt struct {
	ID   int    `xml:"numFmtId,attr"`
	Code string `xml:"formatCode,attr"`
}
type xmlWorksheet struct {
	Rows []xmlWorksheetRow `xml:"sheetData>row"`
}
type xmlWorksheetRow struct {
	Number int                `xml:"r,attr"`
	Cells  []xmlWorksheetCell `xml:"c"`
}
type xmlWorksheetCell struct {
	Ref    string    `xml:"r,attr"`
	Type   string    `xml:"t,attr"`
	Style  int       `xml:"s,attr"`
	Value  string    `xml:"v"`
	Inline xmlInline `xml:"is"`
}
type xmlInline struct {
	Text []string `xml:"t"`
	Runs []xmlRun `xml:"r"`
}

func isXLSX(data []byte) bool { return len(data) >= 4 && bytes.Equal(data[:4], []byte("PK\x03\x04")) }
func isXLS(data []byte) bool {
	return len(data) >= 8 && bytes.Equal(data[:8], []byte{0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1})
}

func parseEmployeeWorkbook(data []byte, expected []string) ([]map[string]string, error) {
	workbook, err := parseXLSX(data)
	if err != nil {
		return nil, err
	}
	return mapExcelRows(workbook, expected)
}

func parseXLSX(data []byte) (excelWorkbook, error) {
	if isXLS(data) {
		return excelWorkbook{}, errUnsupportedXLS
	}
	if !isXLSX(data) {
		return excelWorkbook{}, errInvalidExcel
	}
	reader, err := zip.NewReader(bytes.NewReader(data), int64(len(data)))
	if err != nil {
		return excelWorkbook{}, fmt.Errorf("%w: %v", errInvalidExcel, err)
	}
	files := make(map[string][]byte, len(reader.File))
	for _, item := range reader.File {
		if item.FileInfo().IsDir() {
			continue
		}
		body, readErr := item.Open()
		if readErr != nil {
			return excelWorkbook{}, fmt.Errorf("%w: %v", errInvalidExcel, readErr)
		}
		content, readErr := io.ReadAll(io.LimitReader(body, 32<<20))
		_ = body.Close()
		if readErr != nil {
			return excelWorkbook{}, fmt.Errorf("%w: %v", errInvalidExcel, readErr)
		}
		files[item.Name] = content
	}
	var workbookXML xmlWorkbook
	if err = xml.Unmarshal(files["xl/workbook.xml"], &workbookXML); err != nil {
		return excelWorkbook{}, fmt.Errorf("%w: workbook XML", errInvalidExcel)
	}
	var rels xmlRels
	if err = xml.Unmarshal(files["xl/_rels/workbook.xml.rels"], &rels); err != nil {
		return excelWorkbook{}, fmt.Errorf("%w: workbook relationships", errInvalidExcel)
	}
	targetByID := make(map[string]string, len(rels.Relationships))
	for _, rel := range rels.Relationships {
		targetByID[rel.ID] = path.Join("xl", rel.Target)
	}
	var sst xmlSST
	if raw, ok := files["xl/sharedStrings.xml"]; ok {
		_ = xml.Unmarshal(raw, &sst)
	}
	shared := make([]string, len(sst.Items))
	for i, item := range sst.Items {
		shared[i] = strings.Join(item.Text, "") + joinRuns(item.Runs)
	}
	dateStyles := make(map[int]bool)
	if raw, ok := files["xl/styles.xml"]; ok {
		dateStyles = parseDateStyles(raw)
	}
	result := excelWorkbook{shared: shared, dateStyles: dateStyles}
	for _, sheet := range workbookXML.Sheets {
		target, ok := targetByID[sheet.RID]
		if !ok {
			continue
		}
		raw, ok := files[target]
		if !ok {
			continue
		}
		var ws xmlWorksheet
		if err = xml.Unmarshal(raw, &ws); err != nil {
			return excelWorkbook{}, fmt.Errorf("%w: worksheet XML", errInvalidExcel)
		}
		parsed := excelSheet{name: sheet.Name}
		for _, row := range ws.Rows {
			parsedRow := excelRow{number: row.Number}
			for _, cell := range row.Cells {
				value := cell.Value
				if cell.Type == "inlineStr" {
					value = strings.Join(cell.Inline.Text, "") + joinRuns(cell.Inline.Runs)
				}
				if cell.Type == "s" {
					if index, e := strconv.Atoi(strings.TrimSpace(value)); e == nil && index >= 0 && index < len(shared) {
						value = shared[index]
					}
				}
				if value != "" && cell.Type != "s" && dateStyles[cell.Style] {
					if serial, e := strconv.ParseFloat(value, 64); e == nil {
						value = excelDate(serial)
					}
				}
				parsedRow.cells = append(parsedRow.cells, excelCell{ref: cell.Ref, style: cell.Style, kind: cell.Type, value: strings.TrimSpace(value)})
			}
			parsed.rows = append(parsed.rows, parsedRow)
		}
		result.sheets = append(result.sheets, parsed)
	}
	if len(result.sheets) == 0 {
		return excelWorkbook{}, errInvalidExcel
	}
	return result, nil
}

func joinRuns(runs []xmlRun) string {
	var b strings.Builder
	for _, run := range runs {
		b.WriteString(strings.Join(run.Text, ""))
	}
	return b.String()
}

func parseDateStyles(raw []byte) map[int]bool {
	var style xmlStyle
	if xml.Unmarshal(raw, &style) != nil {
		return nil
	}
	custom := make(map[int]bool)
	for _, format := range style.NumFmts {
		code := strings.ToLower(format.Code)
		custom[format.ID] = strings.ContainsAny(code, "dmy")
	}
	result := make(map[int]bool)
	for i, xf := range style.XFs {
		result[i] = xf.NumFmtID >= 14 && xf.NumFmtID <= 22 || custom[xf.NumFmtID]
	}
	return result
}

func excelDate(serial float64) string {
	// Excel's 1900 date system includes its historical fictitious 1900-02-29.
	days := int64(math.Floor(serial))
	fraction := serial - float64(days)
	base := time.Date(1899, 12, 31, 0, 0, 0, 0, time.UTC)
	if days >= 60 {
		days--
	}
	date := base.AddDate(0, 0, int(days))
	if fraction >= 0.5 {
		date = date.AddDate(0, 0, 1)
	}
	return date.Format("2006-01-02")
}

func mapExcelRows(workbook excelWorkbook, expected []string) ([]map[string]string, error) {
	if len(expected) == 0 {
		return nil, errInvalidExcel
	}
	best := -1
	bestCount := -1
	for index, sheet := range workbook.sheets {
		if len(sheet.rows) == 0 {
			continue
		}
		count := 0
		for _, cell := range sheet.rows[0].cells {
			for _, wanted := range expected {
				if normalizeExcelHeader(cell.value) == normalizeExcelHeader(wanted) {
					count++
					break
				}
			}
		}
		if count > bestCount {
			best, bestCount = index, count
		}
	}
	if best < 0 {
		return nil, errInvalidExcel
	}
	sheet := workbook.sheets[best]
	indexes := make(map[string]int, len(expected))
	for index, cell := range sheet.rows[0].cells {
		name := normalizeExcelHeader(cell.value)
		for _, wanted := range expected {
			if name == normalizeExcelHeader(wanted) {
				if _, exists := indexes[wanted]; exists {
					return nil, fmt.Errorf("duplicate header %s", wanted)
				}
				indexes[wanted] = index
			}
		}
	}
	for _, wanted := range expected[:3] {
		if _, ok := indexes[wanted]; !ok {
			return nil, fmt.Errorf("missing mandatory header %s", wanted)
		}
	}
	rows := make([]map[string]string, 0, len(sheet.rows)-1)
	for _, row := range sheet.rows[1:] {
		mapped := make(map[string]string, len(expected))
		for _, wanted := range expected {
			if index, ok := indexes[wanted]; ok {
				mapped[wanted] = cellAt(row.cells, index)
			}
		}
		empty := true
		for _, value := range mapped {
			if strings.TrimSpace(value) != "" {
				empty = false
				break
			}
		}
		if !empty {
			mapped["_row_number"] = strconv.Itoa(row.number)
			rows = append(rows, mapped)
		}
	}
	return rows, nil
}
func cellAt(cells []excelCell, index int) string {
	if index < 0 || index >= len(cells) {
		return ""
	}
	return cells[index].value
}
func normalizeExcelHeader(value string) string {
	return strings.ToLower(strings.TrimSpace(strings.TrimPrefix(value, "\ufeff")))
}

func generateTerminationTemplate(lang string) ([]byte, error) {
	messages := terminationTemplateMessages(lang)
	sheets := []xlsxSheet{
		{Name: messages.templateName, Widths: []float64{20, 20, 25, 40}, Rows: [][]string{{"employee_code", "termination_date", "termination_reason", "termination_notes"}, {"EMP001", "2026-06-30", "voluntary_resignation", messages.exampleNotes}}},
		{Name: messages.instructionsName, Widths: []float64{25, 20, 80}, Rows: instructionRows(messages)},
	}
	return writeXLSX(sheets)
}

type terminationMessages struct {
	templateName, instructionsName, title, column, required, description, employeeCode, terminationDate, terminationReason, terminationNotes, reasonsTitle, reasonKey, reasonDescription, yes, no, exampleNotes string
	reasons                                                                                                                                                                                                     [8]string
}

func terminationTemplateMessages(lang string) terminationMessages {
	if lang == "vi" {
		return terminationMessages{"Mẫu nhập", "Hướng dẫn", "Hướng dẫn & Lý do hợp lệ", "Cột", "Bắt buộc?", "Mô tả", "Mã số nhân viên duy nhất", "Ngày làm việc cuối cùng (định dạng YYYY-MM-DD hoặc DD/MM/YYYY)", "Mã lý do nghỉ việc (Phải nhập chính xác từ bảng bên dưới)", "Ghi chú thêm (không bắt buộc)", "Mã lý do hợp lệ", "Giá trị của termination_reason", "Mô tả lý do", "Có", "Không", "Chuyển nhà sang thành phố khác", [8]string{"Nghỉ tự nguyện", "Sa thải", "Hết hạn hợp đồng", "Nghỉ hưu", "Thỏa thuận chấm dứt", "Bỏ việc không phép", "Từ trần", "Lý do khác"}}
	}
	if lang == "zh" {
		return terminationMessages{"输入模板", "使用说明", "使用说明与有效离职原因", "列名", "是否必填?", "说明", "唯一的员工编号", "最后工作日 (YYYY-MM-DD 或 DD/MM/YYYY)", "离职原因代码 (必须从下方表格选择)", "可选备注/说明", "有效离职原因代码", "termination_reason 值", "原因描述", "是", "否", "搬迁至其他城市", [8]string{"自愿辞职", "解雇", "合同到期", "退休", "协商一致", "擅自离职", "因公/因病去世", "其他"}}
	}
	return terminationMessages{"Template", "Instructions", "Instructions & Valid Reasons", "Column", "Required?", "Description", "Unique employee identifier code", "Last day of work (YYYY-MM-DD or DD/MM/YYYY)", "Reason code (Must select from table below)", "Optional comments/notes", "Valid Reason Keys", "termination_reason Value", "Description", "Yes", "No", "Relocation to another city", [8]string{"Voluntary Resignation", "Involuntary Termination", "End of Contract", "Retirement", "Mutual Agreement", "Job Abandonment", "Death", "Other"}}
}
func instructionRows(m terminationMessages) [][]string {
	rows := [][]string{{m.title}, {}, {m.column, m.required, m.description}, {"employee_code", m.yes, m.employeeCode}, {"termination_date", m.yes, m.terminationDate}, {"termination_reason", m.yes, m.terminationReason}, {"termination_notes", m.no, m.terminationNotes}, {}, {}, {m.reasonsTitle}, {}, {m.reasonKey, m.reasonDescription}}
	keys := [...]string{"voluntary_resignation", "involuntary_termination", "end_of_contract", "retirement", "mutual_agreement", "abandonment", "death", "other"}
	for i, key := range keys {
		rows = append(rows, []string{key, m.reasons[i]})
	}
	return rows
}

type xlsxSheet struct {
	Name   string
	Widths []float64
	Rows   [][]string
}

func writeXLSX(sheets []xlsxSheet) ([]byte, error) {
	var output bytes.Buffer
	archive := zip.NewWriter(&output)
	files := map[string]string{"[Content_Types].xml": contentTypesXML(len(sheets)), "_rels/.rels": rootRelsXML(), "xl/workbook.xml": workbookXML(sheets), "xl/_rels/workbook.xml.rels": workbookRelsXML(len(sheets))}
	for i, sheet := range sheets {
		files[fmt.Sprintf("xl/worksheets/sheet%d.xml", i+1)] = worksheetXML(sheet)
	}
	for name, body := range files {
		entry, err := archive.Create(name)
		if err != nil {
			return nil, err
		}
		if _, err = entry.Write([]byte(body)); err != nil {
			return nil, err
		}
	}
	if err := archive.Close(); err != nil {
		return nil, err
	}
	return output.Bytes(), nil
}
func xmlEscape(value string) string {
	var b strings.Builder
	_ = xml.EscapeText(&b, []byte(value))
	return b.String()
}
func contentTypesXML(count int) string {
	var b strings.Builder
	b.WriteString(`<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>`)
	for i := 1; i <= count; i++ {
		fmt.Fprintf(&b, `<Override PartName="/xl/worksheets/sheet%d.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>`, i)
	}
	b.WriteString(`</Types>`)
	return b.String()
}
func rootRelsXML() string {
	return `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>`
}
func workbookXML(sheets []xlsxSheet) string {
	var b strings.Builder
	b.WriteString(`<?xml version="1.0" encoding="UTF-8" standalone="yes"?><workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><sheets>`)
	for i, sheet := range sheets {
		fmt.Fprintf(&b, `<sheet name="%s" sheetId="%d" r:id="rId%d"/>`, xmlEscape(sheet.Name), i+1, i+1)
	}
	b.WriteString(`</sheets></workbook>`)
	return b.String()
}
func workbookRelsXML(count int) string {
	var b strings.Builder
	b.WriteString(`<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">`)
	for i := 1; i <= count; i++ {
		fmt.Fprintf(&b, `<Relationship Id="rId%d" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet%d.xml"/>`, i, i)
	}
	b.WriteString(`</Relationships>`)
	return b.String()
}
func worksheetXML(sheet xlsxSheet) string {
	var b strings.Builder
	fmt.Fprintf(&b, `<?xml version="1.0" encoding="UTF-8" standalone="yes"?><worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><cols>`)
	for i, width := range sheet.Widths {
		fmt.Fprintf(&b, `<col min="%d" max="%d" width="%g" customWidth="1"/>`, i+1, i+1, width)
	}
	b.WriteString(`</cols><sheetData>`)
	for r, row := range sheet.Rows {
		fmt.Fprintf(&b, `<row r="%d">`, r+1)
		for c, value := range row {
			if value == "" {
				continue
			}
			ref := excelRef(c, r)
			fmt.Fprintf(&b, `<c r="%s" t="inlineStr"><is><t>%s</t></is></c>`, ref, xmlEscape(value))
		}
		b.WriteString(`</row>`)
	}
	b.WriteString(`</sheetData></worksheet>`)
	return b.String()
}
func excelRef(column, row int) string {
	var letters [8]byte
	n := column + 1
	i := len(letters)
	for n > 0 {
		i--
		letters[i] = byte('A' + (n-1)%26)
		n = (n - 1) / 26
	}
	return string(letters[i:]) + strconv.Itoa(row+1)
}
