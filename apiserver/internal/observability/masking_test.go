package observability

import (
	"testing"
)

func TestMaskEmail_ExactRustVectors(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"john.doe@example.com", "joh***@example.com"},
		{"lee@gmail.com", "l***@gmail.com"},
		{"me@test.com", "***@test.com"},
		{"a@b.com", "***@b.com"},
		{"longemailaddress@corp.com", "lon***@corp.com"},
	}

	for _, tc := range tests {
		actual := MaskEmail(tc.input)
		if actual != tc.expected {
			t.Errorf("MaskEmail(%q) = %q, expected %q", tc.input, actual, tc.expected)
		}
	}
}

func TestMaskEmail_InvalidAndEdgeCases(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"", "***"},
		{"a", "***"},
		{"ab", "***"},
		{"abc", "ab***"},
		{"invalid-email-address", "in***"},
		{"not@an@email.com", "no***"},
	}

	for _, tc := range tests {
		actual := MaskEmail(tc.input)
		if actual != tc.expected {
			t.Errorf("MaskEmail(%q) = %q, expected %q", tc.input, actual, tc.expected)
		}
	}
}

func TestMaskPhone_ExactRustVectors(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"0987654321", "098****321"},
		{"12345", "***"},
		{"1234567", "123*567"},
		{"", "***"},
		{"123456", "***"},
		{"01234567890", "012*****890"},
	}

	for _, tc := range tests {
		actual := MaskPhone(tc.input)
		if actual != tc.expected {
			t.Errorf("MaskPhone(%q) = %q, expected %q", tc.input, actual, tc.expected)
		}
	}
}

func TestMaskBankAccount_ExactRustVectors(t *testing.T) {
	tests := []struct {
		input    string
		expected string
	}{
		{"1234567890", "******7890"},
		{"123", "****"},
		{"12345", "*2345"},
		{"", "****"},
		{"1234", "****"},
		{"9876543210123456", "************3456"},
	}

	for _, tc := range tests {
		actual := MaskBankAccount(tc.input)
		if actual != tc.expected {
			t.Errorf("MaskBankAccount(%q) = %q, expected %q", tc.input, actual, tc.expected)
		}
	}
}

func TestMaskPIIValue_Keys(t *testing.T) {
	tests := []struct {
		key      string
		val      string
		expected string
	}{
		{"user_email", "alice@example.com", "ali***@example.com"},
		{"EMAIL_ADDRESS", "bob.smith@corp.com", "bob***@corp.com"},
		{"contact_phone", "0987654321", "098****321"},
		{"bank_account_num", "1234567890", "******7890"},
		{"account_number", "1234567890", "******7890"},
		{"password", "p@$$w0rd", "[REDACTED]"},
		{"user_password_hash", "$2b$12$e...", "[REDACTED]"},
		{"api_secret", "sec_12345", "[REDACTED]"},
		{"auth_token", "Bearer eyJhbGciOi...", "[REDACTED]"},
		{"jwt_token", "header.payload.sig", "[REDACTED]"},
		{"encryption_key", "0123456789abcdef", "[REDACTED]"},
		{"national_id", "123456789012", "[REDACTED]"},
		{"username", "alice_lee", "alice_lee"},
		{"workspace_id", "01946850-891a-7b3b-9a4f-9588fb7cfa02", "01946850-891a-7b3b-9a4f-9588fb7cfa02"},
	}

	for _, tc := range tests {
		actual := MaskPIIValue(tc.key, tc.val)
		if actual != tc.expected {
			t.Errorf("MaskPIIValue(%q, %q) = %q, expected %q", tc.key, tc.val, actual, tc.expected)
		}
	}
}
