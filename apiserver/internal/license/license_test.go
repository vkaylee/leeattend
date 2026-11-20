package license

import (
	"crypto/ed25519"
	"encoding/base64"
	"encoding/json"
	"errors"
	"os"
	"path/filepath"
	"strings"
	"testing"
	"time"
)

func signedEnvelope(t *testing.T, privateKey ed25519.PrivateKey, payload Payload) []byte {
	t.Helper()
	canonical, err := json.Marshal(payload)
	if err != nil {
		t.Fatalf("marshal payload: %v", err)
	}
	envelope := Envelope{Payload: payload, Signature: base64.RawURLEncoding.EncodeToString(ed25519.Sign(privateKey, canonical))}
	data, err := json.Marshal(envelope)
	if err != nil {
		t.Fatalf("marshal envelope: %v", err)
	}
	return data
}

func testPayload() Payload {
	return Payload{SchemaVersion: SchemaVersion, Product: Product, Edition: "professional", Customer: "Example Corp", IssuedAt: time.Now().UTC().Format(time.RFC3339), Entitlements: []string{"attendance", "payroll"}}
}

func newTestVerifier(t *testing.T) (ed25519.PrivateKey, *Verifier) {
	t.Helper()
	publicKey, privateKey, err := ed25519.GenerateKey(nil)
	if err != nil {
		t.Fatalf("generate key: %v", err)
	}
	verifier, err := NewVerifier(publicKey, Product)
	if err != nil {
		t.Fatalf("new verifier: %v", err)
	}
	return privateKey, verifier
}

func TestVerifyAcceptsValidPerpetualLicense(t *testing.T) {
	privateKey, verifier := newTestVerifier(t)
	status, err := verifier.Verify(signedEnvelope(t, privateKey, testPayload()))
	if err != nil {
		t.Fatalf("verify: %v", err)
	}
	if !status.Valid || !HasEntitlement(status, "payroll") {
		t.Fatalf("unexpected status: %+v", status)
	}
	if status.LicenseDigest == "" || status.Product != Product {
		t.Fatalf("missing redacted status fields: %+v", status)
	}
}

func TestVerifyRejectsTamperedPayload(t *testing.T) {
	privateKey, verifier := newTestVerifier(t)
	data := signedEnvelope(t, privateKey, testPayload())
	marker := []byte(`"signature":"`)
	start := strings.Index(string(data), string(marker)) + len(marker)
	if start < len(marker) {
		t.Fatal("signature marker missing")
	}
	if data[start] == 'A' {
		data[start] = 'B'
	} else {
		data[start] = 'A'
	}
	if _, err := verifier.Verify(data); !errors.Is(err, ErrInvalidSignature) {
		t.Fatalf("expected invalid signature, got %v", err)
	}
}

func TestVerifyRejectsWrongProduct(t *testing.T) {
	privateKey, verifier := newTestVerifier(t)
	payload := testPayload()
	payload.Product = "other-product"
	if _, err := verifier.Verify(signedEnvelope(t, privateKey, payload)); !errors.Is(err, ErrWrongProduct) {
		t.Fatalf("expected wrong product, got %v", err)
	}
}

func TestVerifyRejectsMissingEntitlement(t *testing.T) {
	privateKey, verifier := newTestVerifier(t)
	payload := testPayload()
	payload.Entitlements = nil
	if _, err := verifier.Verify(signedEnvelope(t, privateKey, payload)); !errors.Is(err, ErrMissingEntitlement) {
		t.Fatalf("expected missing entitlement, got %v", err)
	}
}

func TestVerifyFileRejectsMalformedJSON(t *testing.T) {
	_, verifier := newTestVerifier(t)
	dir := t.TempDir()
	path := filepath.Join(dir, "license.json")
	if err := os.WriteFile(path, []byte("{"), 0o600); err != nil {
		t.Fatalf("write license: %v", err)
	}
	if _, err := verifier.VerifyFile(path); !errors.Is(err, ErrInvalidLicense) {
		t.Fatalf("expected invalid license, got %v", err)
	}
}
