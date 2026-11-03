// Package license verifies LeeAttend licenses without network access.
package license

import (
	"crypto/ed25519"
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"strings"
	"time"
)

const (
	SchemaVersion   = 1
	Product         = "leeattend"
	maxLicenseBytes = 64 * 1024
)

var (
	ErrInvalidLicense     = errors.New("license: invalid license")
	ErrUnsupportedSchema  = errors.New("license: unsupported schema version")
	ErrWrongProduct       = errors.New("license: wrong product")
	ErrInvalidSignature   = errors.New("license: invalid signature")
	ErrMissingEntitlement = errors.New("license: missing entitlement")
)

// Payload is the signed, non-secret license content.
type Payload struct {
	SchemaVersion int      `json:"schema_version"`
	Product       string   `json:"product"`
	Edition       string   `json:"edition"`
	Customer      string   `json:"customer"`
	IssuedAt      string   `json:"issued_at"`
	Entitlements  []string `json:"entitlements"`
}

// Envelope is the on-disk license format. Signature is base64.RawURLEncoding
// over the canonical JSON representation of Payload.
type Envelope struct {
	Payload   Payload `json:"payload"`
	Signature string  `json:"signature"`
}

// Status is safe to return to an administrator or health endpoint. It excludes
// customer identity and signature material.
type Status struct {
	Valid         bool     `json:"valid"`
	Product       string   `json:"product"`
	Edition       string   `json:"edition"`
	Entitlements  []string `json:"entitlements"`
	LicenseDigest string   `json:"license_digest"`
}

// Verifier verifies licenses signed by the configured offline public key.
type Verifier struct {
	publicKey ed25519.PublicKey
	product   string
}

// NewVerifier creates a verifier. The public key is copied so callers cannot
// mutate verifier behavior after construction.
func NewVerifier(publicKey ed25519.PublicKey, product string) (*Verifier, error) {
	if len(publicKey) != ed25519.PublicKeySize {
		return nil, fmt.Errorf("%w: public key length", ErrInvalidLicense)
	}
	product = strings.TrimSpace(product)
	if product == "" {
		return nil, fmt.Errorf("%w: product is empty", ErrInvalidLicense)
	}
	key := make(ed25519.PublicKey, len(publicKey))
	copy(key, publicKey)
	return &Verifier{publicKey: key, product: product}, nil
}

// NewVerifierFromBase64 creates a verifier from a base64url-encoded Ed25519 public key.
func NewVerifierFromBase64(encoded, product string) (*Verifier, error) {
	key, err := base64.RawURLEncoding.DecodeString(strings.TrimSpace(encoded))
	if err != nil {
		return nil, fmt.Errorf("%w: public key encoding", ErrInvalidLicense)
	}
	return NewVerifier(ed25519.PublicKey(key), product)
}

// VerifyFile loads and verifies one local license file. It never performs I/O
// beyond the supplied path and never makes network calls.
func (v *Verifier) VerifyFile(path string) (Status, error) {
	if v == nil || len(v.publicKey) != ed25519.PublicKeySize {
		return Status{}, fmt.Errorf("%w: verifier is not configured", ErrInvalidLicense)
	}
	if strings.TrimSpace(path) == "" {
		return Status{}, fmt.Errorf("%w: license path is empty", ErrInvalidLicense)
	}
	data, err := os.ReadFile(path)
	if err != nil {
		return Status{}, fmt.Errorf("license: read file: %w", err)
	}
	if len(data) == 0 || len(data) > maxLicenseBytes {
		return Status{}, fmt.Errorf("%w: file size", ErrInvalidLicense)
	}
	return v.Verify(data)
}

// Verify validates an envelope encoded as JSON.
func (v *Verifier) Verify(data []byte) (Status, error) {
	if v == nil || len(v.publicKey) != ed25519.PublicKeySize {
		return Status{}, fmt.Errorf("%w: verifier is not configured", ErrInvalidLicense)
	}
	var envelope Envelope
	decoder := json.NewDecoder(strings.NewReader(string(data)))
	decoder.DisallowUnknownFields()
	if err := decoder.Decode(&envelope); err != nil {
		return Status{}, fmt.Errorf("%w: malformed json", ErrInvalidLicense)
	}
	var trailing any
	if err := decoder.Decode(&trailing); !errors.Is(err, io.EOF) {
		return Status{}, fmt.Errorf("%w: trailing json", ErrInvalidLicense)
	}
	if envelope.Payload.SchemaVersion != SchemaVersion {
		return Status{}, ErrUnsupportedSchema
	}
	if envelope.Payload.Product != v.product {
		return Status{}, ErrWrongProduct
	}
	if strings.TrimSpace(envelope.Payload.Edition) == "" || strings.TrimSpace(envelope.Payload.IssuedAt) == "" {
		return Status{}, fmt.Errorf("%w: required payload field", ErrInvalidLicense)
	}
	if _, err := time.Parse(time.RFC3339, envelope.Payload.IssuedAt); err != nil {
		return Status{}, fmt.Errorf("%w: issued_at", ErrInvalidLicense)
	}
	if len(envelope.Payload.Entitlements) == 0 {
		return Status{}, ErrMissingEntitlement
	}
	signature, err := base64.RawURLEncoding.DecodeString(envelope.Signature)
	if err != nil || len(signature) != ed25519.SignatureSize {
		return Status{}, ErrInvalidSignature
	}
	canonical, err := json.Marshal(envelope.Payload)
	if err != nil {
		return Status{}, fmt.Errorf("%w: canonical payload", ErrInvalidLicense)
	}
	if !ed25519.Verify(v.publicKey, canonical, signature) {
		return Status{}, ErrInvalidSignature
	}
	digest := sha256.Sum256(data)
	entitlements := append([]string(nil), envelope.Payload.Entitlements...)
	return Status{
		Valid:         true,
		Product:       v.product,
		Edition:       envelope.Payload.Edition,
		Entitlements:  entitlements,
		LicenseDigest: hex.EncodeToString(digest[:]),
	}, nil
}

// HasEntitlement reports whether a verified status includes an entitlement.
func HasEntitlement(status Status, entitlement string) bool {
	if !status.Valid || strings.TrimSpace(entitlement) == "" {
		return false
	}
	for _, value := range status.Entitlements {
		if value == entitlement {
			return true
		}
	}
	return false
}
