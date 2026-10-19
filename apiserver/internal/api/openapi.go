package api

import (
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"fmt"
	"os"
)

var ErrMissingOpenAPIArtifact = errors.New("openapi artifact is missing")
var ErrOpenAPIDrift = errors.New("openapi artifact differs from frozen baseline")

// OpenAPIContract describes one generated artifact checked against frozen evidence.
type OpenAPIContract struct {
	ArtifactPath string
	ExpectedHash string
}

// ValidateOpenAPIArtifact hashes artifact and compares it to expected Rust/frontend baseline.
// Missing artifacts and hashes fail closed; no schema is silently regenerated.
func ValidateOpenAPIArtifact(contract OpenAPIContract) error {
	if contract.ArtifactPath == "" || contract.ExpectedHash == "" {
		return fmt.Errorf("%w: path and expected hash are required", ErrMissingOpenAPIArtifact)
	}
	data, err := os.ReadFile(contract.ArtifactPath)
	if err != nil {
		return fmt.Errorf("%w: %s", ErrMissingOpenAPIArtifact, contract.ArtifactPath)
	}
	digest := sha256.Sum256(data)
	actual := hex.EncodeToString(digest[:])
	if actual != contract.ExpectedHash {
		return fmt.Errorf("%w: expected %s, got %s", ErrOpenAPIDrift, contract.ExpectedHash, actual)
	}
	return nil
}
