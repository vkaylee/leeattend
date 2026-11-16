package repository

import (
	"context"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/jackc/pgx/v5"
)

func TestDocumentTemplateVariablesResolveNumberThroughEmploymentTenure(t *testing.T) {
	db := &fakeDBTX{
		queryRowFn: func(_ context.Context, query string, _ ...any) pgx.Row {
			if strings.Contains(query, "FROM employments") {
				if !strings.Contains(query, "employee_tenures") || !strings.Contains(query, "em.start_date") {
					t.Fatalf("document variables must resolve tenure number by employment date: %s", query)
				}
				return fakeRow{scanFn: func(dest ...any) error {
					return pgx.ErrNoRows
				}}
			}
			return fakeRow{scanFn: func(dest ...any) error {
				return pgx.ErrNoRows
			}}
		},
	}
	if _, err := (DocumentRepository{DB: db}).GetTemplateVariables(context.Background(), uuid.New()); err != nil {
		t.Fatalf("get template variables: %v", err)
	}
}
