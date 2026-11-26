package api

import (
	"go/parser"
	"go/token"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

// TestAPIPackageDoesNotImportInfraDrivers locks in DIP: HTTP handlers in internal/api
// must depend on repository/infra interfaces, never concrete pgx or go-redis drivers.
func TestAPIPackageDoesNotImportInfraDrivers(t *testing.T) {
	forbidden := []string{
		"github.com/jackc/pgx",
		"github.com/redis/go-redis",
	}

	entries, err := os.ReadDir(".")
	if err != nil {
		t.Fatalf("readdir: %v", err)
	}

	fset := token.NewFileSet()
	for _, entry := range entries {
		name := entry.Name()
		if entry.IsDir() || !strings.HasSuffix(name, ".go") || strings.HasSuffix(name, "_test.go") {
			continue
		}
		file, err := parser.ParseFile(fset, filepath.Join(".", name), nil, parser.ImportsOnly)
		if err != nil {
			t.Fatalf("parse %s: %v", name, err)
		}
		for _, imp := range file.Imports {
			path := strings.Trim(imp.Path.Value, `"`)
			for _, prefix := range forbidden {
				if strings.HasPrefix(path, prefix) {
					t.Errorf("%s imports forbidden driver package %q; translate driver types in repository/infra", name, path)
				}
			}
		}
	}
}
