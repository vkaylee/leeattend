package identity

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"errors"
	"strings"
	"time"
	"unicode"

	"golang.org/x/text/cases"
	"golang.org/x/text/unicode/norm"
)

const (
	MinHMACSecretLen = 32

	nationalIDDomainPrefix = "employee_profile:national_id:v1:"
	phoneDomainPrefix      = "employee_profile:phone:v1:"
)

var (
	ErrIdentityHashSecretMissing = errors.New("identity lookup hmac secret must be configured and at least 32 bytes")
	ErrInvalidDOB                = errors.New("date_of_birth must be a valid ISO YYYY-MM-DD date")
	ErrInvalidPhone              = errors.New("phone must be valid E.164 with explicit country calling code")
)

// NormalizeName applies Unicode NFC, trims and collapses Unicode whitespace runs,
// and Unicode case-folds while preserving punctuation.
func NormalizeName(raw string) string {
	collapsed := strings.Join(strings.Fields(norm.NFC.String(raw)), " ")
	if collapsed == "" {
		return ""
	}
	return norm.NFC.String(cases.Fold().String(collapsed))
}

// NormalizeDOB validates strict ISO YYYY-MM-DD dates without timezone conversion.
// Empty/whitespace-only input returns "", nil.
func NormalizeDOB(raw string) (string, error) {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		return "", nil
	}
	if len(trimmed) != 10 {
		return "", ErrInvalidDOB
	}
	parsed, err := time.Parse("2006-01-02", trimmed)
	if err != nil || parsed.Format("2006-01-02") != trimmed {
		return "", ErrInvalidDOB
	}
	return trimmed, nil
}

// NormalizeNationalID applies Unicode NFKC, trims outer whitespace, uppercases,
// and removes documented visual separators while keeping leading zeroes.
func NormalizeNationalID(raw string) string {
	canonical := strings.ToUpper(strings.TrimSpace(norm.NFKC.String(raw)))
	if canonical == "" {
		return ""
	}
	var b strings.Builder
	b.Grow(len(canonical))
	for _, r := range canonical {
		if isNationalIDVisualSeparator(r) {
			continue
		}
		b.WriteRune(r)
	}
	return b.String()
}

func isNationalIDVisualSeparator(r rune) bool {
	if unicode.IsSpace(r) {
		return true
	}
	switch r {
	case '-', '.', '/', '_', '−':
		return true
	default:
		return r >= '‐' && r <= '―'
	}
}

// NormalizePhone validates and formats an E.164 phone number with an explicit leading '+'.
// Empty/whitespace-only input returns "", nil. National-only numbers and extensions are rejected.
func NormalizePhone(raw string) (string, error) {
	trimmed := strings.TrimSpace(raw)
	if trimmed == "" {
		return "", nil
	}
	if !strings.HasPrefix(trimmed, "+") {
		return "", ErrInvalidPhone
	}
	digits, ok := extractE164Digits(trimmed[1:])
	if !ok || len(digits) < 7 || len(digits) > 15 || digits[0] == '0' {
		return "", ErrInvalidPhone
	}
	return "+" + string(digits), nil
}

func extractE164Digits(s string) ([]byte, bool) {
	digits := make([]byte, 0, 15)
	for _, r := range s {
		if r >= '0' && r <= '9' {
			digits = append(digits, byte(r))
			continue
		}
		if isPhoneFormattingSeparator(r) {
			continue
		}
		return nil, false
	}
	return digits, true
}

func isPhoneFormattingSeparator(r rune) bool {
	switch r {
	case ' ', '-', '.', '(', ')', '\t':
		return true
	default:
		return false
	}
}

// NormalizeEmployeeCode trims outer whitespace and uppercases while preserving
// leading zeroes, punctuation, and internal whitespace.
func NormalizeEmployeeCode(raw string) string {
	return strings.ToUpper(strings.TrimSpace(raw))
}

// Hasher computes deterministic domain-separated HMAC-SHA256 lookup hashes for profile identities.
type Hasher struct {
	secret []byte
}

// NewHasher builds a Hasher and fails closed when secret is empty or shorter than MinHMACSecretLen.
func NewHasher(secret string) (Hasher, error) {
	if len(strings.TrimSpace(secret)) == 0 || len(secret) < MinHMACSecretLen {
		return Hasher{}, ErrIdentityHashSecretMissing
	}
	key := make([]byte, len(secret))
	copy(key, secret)
	return Hasher{secret: key}, nil
}

func (h Hasher) String() string   { return "[REDACTED]" }
func (h Hasher) GoString() string { return "[REDACTED]" }

// Validate checks that the Hasher holds a configured secret of at least MinHMACSecretLen bytes.
func (h Hasher) Validate() error {
	if len(h.secret) < MinHMACSecretLen {
		return ErrIdentityHashSecretMissing
	}
	return nil
}

// HashNationalID normalizes raw national ID and returns its 64-char lowercase hex HMAC-SHA256.
// Empty normalized values return ("", false, nil) and never produce a hash.
func (h Hasher) HashNationalID(raw string) (string, bool, error) {
	if err := h.Validate(); err != nil {
		return "", false, err
	}
	normalized := NormalizeNationalID(raw)
	if normalized == "" {
		return "", false, nil
	}
	return h.hmacHex(nationalIDDomainPrefix, normalized), true, nil
}

// HashPhone normalizes raw phone to strict E.164 and returns its 64-char lowercase hex HMAC-SHA256.
// Empty input returns ("", false, nil); non-empty invalid phones fail with ErrInvalidPhone.
func (h Hasher) HashPhone(raw string) (string, bool, error) {
	if err := h.Validate(); err != nil {
		return "", false, err
	}
	normalized, err := NormalizePhone(raw)
	if err != nil {
		return "", false, err
	}
	if normalized == "" {
		return "", false, nil
	}
	return h.hmacHex(phoneDomainPrefix, normalized), true, nil
}

func (h Hasher) hmacHex(prefix, canonical string) string {
	mac := hmac.New(sha256.New, h.secret)
	_, _ = mac.Write([]byte(prefix))
	_, _ = mac.Write([]byte(canonical))
	return hex.EncodeToString(mac.Sum(nil))
}
