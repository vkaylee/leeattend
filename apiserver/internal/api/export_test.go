package api

import (
	"bytes"
	"net/http/httptest"
	"testing"
)

func TestWriteCSV_HeadersAndContent(t *testing.T) {
	rec := httptest.NewRecorder()
	rows := [][]string{
		{"id", "name"},
		{"1", "Alice"},
	}
	if err := WriteCSV(rec, "users.csv", rows); err != nil {
		t.Fatal(err)
	}
	if rec.Header().Get("Content-Type") != "text/csv; charset=utf-8" {
		t.Fatalf("unexpected content type: %s", rec.Header().Get("Content-Type"))
	}
	if rec.Header().Get("Content-Disposition") != "attachment; filename=\"users.csv\"" {
		t.Fatalf("unexpected content disposition: %s", rec.Header().Get("Content-Disposition"))
	}
	if rec.Header().Get("X-Content-Type-Options") != "nosniff" {
		t.Fatalf("missing nosniff header")
	}
}

func TestReadCSV_SuccessAndLimit(t *testing.T) {
	input := "id,name\n1,Alice\n2,Bob\n"
	rows, err := ReadCSV(bytes.NewBufferString(input), 10)
	if err != nil || len(rows) != 3 {
		t.Fatalf("unexpected rows: %v, err=%v", rows, err)
	}

	_, err = ReadCSV(bytes.NewBufferString(input), 2)
	if err == nil {
		t.Fatal("expected max row error, got nil")
	}
}
