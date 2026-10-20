package observability

import (
	"strings"
)

// MaskEmail masks email addresses according to the Rust reference implementation.
//
// If valid email with local and domain parts:
//   - local len <= 2: "***@domain"
//   - local len <= 4: first char + "***@domain"
//   - local len > 4:  first 3 chars + "***@domain"
//
// If invalid / no single '@':
//   - len <= 2: "***"
//   - len > 2:  first 2 chars + "***"
func MaskEmail(email string) string {
	parts := strings.Split(email, "@")
	if len(parts) != 2 {
		if len(email) <= 2 {
			return "***"
		}
		return email[:2] + "***"
	}

	localPart := parts[0]
	domainPart := parts[1]

	var maskedLocal string
	if len(localPart) <= 2 {
		maskedLocal = "***"
	} else if len(localPart) <= 4 {
		maskedLocal = localPart[:1] + "***"
	} else {
		maskedLocal = localPart[:3] + "***"
	}

	return maskedLocal + "@" + domainPart
}

// MaskPhone masks phone numbers according to the Rust reference implementation.
//
// If len <= 6: "***"
// Otherwise: first 3 chars + (len-6) '*' + last 3 chars.
func MaskPhone(phone string) string {
	n := len(phone)
	if n <= 6 {
		return "***"
	}
	first := phone[:3]
	last := phone[n-3:]
	stars := strings.Repeat("*", n-6)
	return first + stars + last
}

// MaskBankAccount masks bank accounts or account numbers according to the Rust reference implementation.
//
// If len <= 4: "****"
// Otherwise: (len-4) '*' + last 4 chars.
func MaskBankAccount(account string) string {
	n := len(account)
	if n <= 4 {
		return "****"
	}
	last := account[n-4:]
	stars := strings.Repeat("*", n-4)
	return stars + last
}

// MaskPIIValue applies centralized classification-safe field masking based on field key.
//
// Matches Rust apiserver/src/utils/logging.rs:15-35:
//   - Contains "email" -> MaskEmail
//   - Contains "phone" -> MaskPhone
//   - Contains "bank_account" or "account_number" -> MaskBankAccount
//   - Contains "password", "hash", "secret", "token", "jwt", "key", "national_id" -> "[REDACTED]"
//   - Otherwise -> original value
func MaskPIIValue(key, val string) string {
	k := strings.ToLower(key)
	switch {
	case strings.Contains(k, "email"):
		return MaskEmail(val)
	case strings.Contains(k, "phone"):
		return MaskPhone(val)
	case strings.Contains(k, "bank_account") || strings.Contains(k, "account_number"):
		return MaskBankAccount(val)
	case strings.Contains(k, "password") ||
		strings.Contains(k, "hash") ||
		strings.Contains(k, "secret") ||
		strings.Contains(k, "token") ||
		strings.Contains(k, "jwt") ||
		strings.Contains(k, "key") ||
		strings.Contains(k, "national_id"):
		return "[REDACTED]"
	default:
		return val
	}
}
