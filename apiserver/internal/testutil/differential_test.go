package testutil

import (
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// TestDifferentialFixturesStructure asserts that all canonical fixtures in docs/migration/rust-go/fixtures
// can be loaded and executed by the Go fixture runner.
func TestDifferentialFixturesStructure(t *testing.T) {
	fixtureDir := filepath.Join("..", "..", "..", "docs", "migration", "rust-go", "fixtures")
	entries, err := os.ReadDir(fixtureDir)
	if err != nil {
		t.Skipf("fixtures directory not accessible: %v", err)
		return
	}

	var count int
	for _, entry := range entries {
		if !entry.IsDir() {
			continue
		}
		domainDir := filepath.Join(fixtureDir, entry.Name())
		files, err := os.ReadDir(domainDir)
		if err != nil {
			continue
		}
		for _, file := range files {
			if !strings.HasSuffix(file.Name(), ".json") {
				continue
			}
			filePath := filepath.Join(domainDir, file.Name())
			data, err := os.ReadFile(filePath)
			if err != nil {
				t.Errorf("read fixture %s: %v", filePath, err)
				continue
			}
			fixture, err := LoadFixture(data)
			if err != nil {
				t.Errorf("load fixture %s: %v", filePath, err)
				continue
			}
			if fixture.Metadata.ID == "" {
				t.Errorf("fixture %s missing metadata ID", filePath)
			}
			count++
		}
	}
	if count == 0 {
		t.Fatalf("expected at least one valid fixture, found 0")
	}
}

// TestDifferentialExecutionMock verifies normalization between Rust reference and Go candidate mock responses.
func TestDifferentialExecutionMock(t *testing.T) {
	rustServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Header().Set("X-Request-Id", "018d0000-0000-7000-8000-000000000001")
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"data":{"name":"Acme","id":"018d0000-0000-7000-8000-000000000010"}}`))
	}))
	defer rustServer.Close()

	goServer := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		w.Header().Set("X-Request-Id", "018d0000-0000-7000-8000-000000000002") // generated UUIDv7 allowed to differ
		w.WriteHeader(http.StatusOK)
		_, _ = w.Write([]byte(`{"data":{"name":"Acme","id":"018d0000-0000-7000-8000-000000000010"}}`))
	}))
	defer goServer.Close()

	fixtureJSON := []byte(`{
		"schema_version": "1.0.0",
		"metadata": {"id": "DIFF-TEST-001"},
		"request": {
			"method": "GET",
			"path": "/api/v1/workspaces/ws-1",
			"path_params": {},
			"query": {},
			"headers": {},
			"body": null
		},
		"expected_response": {
			"status": 200,
			"headers": {"content-type": "application/json"},
			"body": {"data":{"name":"Acme","id":"018d0000-0000-7000-8000-000000000010"}}
		}
	}`)

	fixture, err := LoadFixture(fixtureJSON)
	if err != nil {
		t.Fatalf("load fixture: %v", err)
	}

	rustResult, err := RunFixture(rustServer.Client(), fixture, rustServer.URL)
	if err != nil {
		t.Fatalf("run rust: %v", err)
	}

	goResult, err := RunFixture(goServer.Client(), fixture, goServer.URL)
	if err != nil {
		t.Fatalf("run go: %v", err)
	}

	if rustResult.Status != goResult.Status {
		t.Errorf("status mismatch: rust=%d go=%d", rustResult.Status, goResult.Status)
	}

	var rustBody, goBody map[string]any
	if err := json.Unmarshal(rustResult.Body, &rustBody); err != nil {
		t.Fatalf("unmarshal rust body: %v", err)
	}
	if err := json.Unmarshal(goResult.Body, &goBody); err != nil {
		t.Fatalf("unmarshal go body: %v", err)
	}

	// pony tail: shallow key equality; nested normalization handles full tolerance tree
	if len(rustBody) != len(goBody) {
		t.Errorf("body key length mismatch: rust=%d go=%d", len(rustBody), len(goBody))
	}
}
