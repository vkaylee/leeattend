package testutil

import (
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestRunFixtureSubstitutesPathAndQuery(t *testing.T) {
	server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path != "/api/v1/organization/settings" {
			t.Errorf("path = %q", r.URL.Path)
		}
		if r.URL.Query().Get("page") != "2" {
			t.Errorf("page = %q", r.URL.Query().Get("page"))
		}
		w.Header().Set("Content-Type", "application/json")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"ok":true}`))
	}))
	defer server.Close()

	fixture, err := LoadFixture([]byte(`{
		"schema_version":"1.0.0",
		"metadata":{"id":"TEST-001"},
		"request":{"method":"GET","path":"/api/v1/organization/settings","path_params":{},"query":{"page":2},"headers":{},"body":null},
		"expected_response":{"status":200}
	}`))
	if err != nil {
		t.Fatal(err)
	}
	result, err := RunFixture(server.Client(), fixture, server.URL)
	if err != nil {
		t.Fatal(err)
	}
	if result.Status != http.StatusOK || result.FixtureID != "TEST-001" {
		t.Fatalf("unexpected result: %+v", result)
	}
}

func TestBaseURL(t *testing.T) {
	if got := BaseURL(false); got != "http://127.0.0.1:3000" {
		t.Errorf("Rust base URL = %q", got)
	}
	if got := BaseURL(true); got != "http://127.0.0.1:3001" {
		t.Errorf("Go base URL = %q", got)
	}
}
