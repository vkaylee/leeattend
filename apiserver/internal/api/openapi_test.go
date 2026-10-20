package api

import (
	"errors"
	"os"
	"path/filepath"
	"testing"
)

func TestValidateOpenAPIMissingArtifactFailsClosed(t *testing.T) {
	err := ValidateOpenAPIArtifact(OpenAPIContract{
		ArtifactPath: filepath.Join(t.TempDir(), "missing.json"),
		ExpectedHash: "deadbeef",
	})
	if !errors.Is(err, ErrMissingOpenAPIArtifact) {
		t.Fatalf("expected missing-artifact error, got %v", err)
	}
}
func TestValidateOpenAPIMatchingArtifact(t *testing.T) {
	path := filepath.Join(t.TempDir(), "openapi.json")
	if err := os.WriteFile(path, []byte(`{"openapi":"3.0.3"}`), 0o600); err != nil {
		t.Fatal(err)
	}
	const hash = "fdf4aaaa0affbe5384df2618b7801c719a16d0057ae1956a4fce07202ed4d28f"
	if err := ValidateOpenAPIArtifact(OpenAPIContract{ArtifactPath: path, ExpectedHash: hash}); err != nil {
		t.Fatalf("expected matching artifact, got %v", err)
	}
}
