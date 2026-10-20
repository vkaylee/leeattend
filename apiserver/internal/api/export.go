package api

import (
	"encoding/csv"
	"errors"
	"fmt"
	"io"
	"net/http"
	"strings"
)

var ErrInvalidCSV = errors.New("invalid csv")

// WriteCSV streams CSV content with standard security and download headers.
func WriteCSV(w http.ResponseWriter, filename string, records [][]string) error {
	if w == nil {
		return errors.New("http response writer is required")
	}
	cleanName := strings.ReplaceAll(filename, "\"", "")
	w.Header().Set("Content-Type", "text/csv; charset=utf-8")
	w.Header().Set("Content-Disposition", fmt.Sprintf("attachment; filename=\"%s\"", cleanName))
	w.Header().Set("X-Content-Type-Options", "nosniff")
	w.WriteHeader(http.StatusOK)

	writer := csv.NewWriter(w)
	if err := writer.WriteAll(records); err != nil {
		return fmt.Errorf("write csv records: %w", err)
	}
	writer.Flush()
	return writer.Error()
}

// ReadCSV parses a CSV input body up to row and byte limits.
func ReadCSV(r io.Reader, maxRows int) ([][]string, error) {
	if r == nil {
		return nil, errors.New("reader is required")
	}
	reader := csv.NewReader(r)
	reader.ReuseRecord = false
	reader.TrimLeadingSpace = true

	var rows [][]string
	for {
		record, err := reader.Read()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return nil, fmt.Errorf("%w: %v", ErrInvalidCSV, err)
		}
		rows = append(rows, record)
		if maxRows > 0 && len(rows) > maxRows {
			return nil, fmt.Errorf("%w: row count exceeds maximum %d", ErrInvalidCSV, maxRows)
		}
	}
	return rows, nil
}
