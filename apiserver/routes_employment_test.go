package main

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func TestRegisterEmploymentRoutesIncludesEmployeeRelationshipList(t *testing.T) {
	router := chi.NewRouter()
	registerEmploymentRoutes(router, &api.ContractEmploymentHandler{}, &api.AssignmentHandler{}, &api.CustomFieldHandler{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/employees/00000000-0000-0000-0000-000000000001/employment-relationships", nil)
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, req)

	if rec.Code == http.StatusNotFound {
		t.Fatalf("employee relationship list route is not registered: got %d", rec.Code)
	}
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("employee relationship list route = %d, want %d before authentication", rec.Code, http.StatusUnauthorized)
	}
}

func TestRegisterEmploymentRoutesIncludesEmployeeContractList(t *testing.T) {
	router := chi.NewRouter()
	registerEmploymentRoutes(router, &api.ContractEmploymentHandler{}, &api.AssignmentHandler{}, &api.CustomFieldHandler{})

	req := httptest.NewRequest(http.MethodGet, "/api/v1/employees/00000000-0000-0000-0000-000000000001/contracts", nil)
	rec := httptest.NewRecorder()
	router.ServeHTTP(rec, req)

	if rec.Code == http.StatusNotFound {
		t.Fatalf("employee contract list route is not registered: got %d", rec.Code)
	}
	if rec.Code != http.StatusUnauthorized {
		t.Fatalf("employee contract list route = %d, want %d before authentication", rec.Code, http.StatusUnauthorized)
	}
}
