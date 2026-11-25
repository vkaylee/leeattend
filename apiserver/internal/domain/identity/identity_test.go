package identity

import (
	"errors"
	"fmt"
	"strings"
	"testing"
)

const testHMACSecret = "0123456789abcdef0123456789abcdef"

func TestNormalizeName(t *testing.T) {
	// NFD vs NFC Vietnamese characters + whitespace collapse + case-folding + punctuation kept.
	nfd := "  Nguyễn   Văn\tA-B.  "
	nfc := "nguyễn văn a-b."
	if got := NormalizeName(nfd); got != nfc {
		t.Fatalf("NormalizeName(%q) = %q, want %q", nfd, got, nfc)
	}
	if got := NormalizeName("   \t\n "); got != "" {
		t.Fatalf("expected empty string for whitespace-only name, got %q", got)
	}
	if NormalizeName("O'Connor") == NormalizeName("OConnor") {
		t.Fatal("punctuation must remain significant in NormalizeName")
	}
}

func TestNormalizeDOB(t *testing.T) {
	got, err := NormalizeDOB(" 1995-08-19 ")
	if err != nil || got != "1995-08-19" {
		t.Fatalf("NormalizeDOB valid = (%q, %v)", got, err)
	}
	if got, err := NormalizeDOB("  "); err != nil || got != "" {
		t.Fatalf("NormalizeDOB empty = (%q, %v)", got, err)
	}
	for _, bad := range []string{"1995-02-30", "1995/08/19", "1995-08-19T00:00:00Z", "19-08-1995"} {
		if _, err := NormalizeDOB(bad); !errors.Is(err, ErrInvalidDOB) {
			t.Fatalf("NormalizeDOB(%q) err = %v, want ErrInvalidDOB", bad, err)
		}
	}
}

func TestNormalizeNationalID(t *testing.T) {
	// NFKC fullwidth digits + visual separators removed + uppercase + leading zeroes kept.
	if got := NormalizeNationalID("  ００１-234.567/89_ab—c  "); got != "00123456789ABC" {
		t.Fatalf("NormalizeNationalID = %q, want 00123456789ABC", got)
	}
	if got := NormalizeNationalID(" - . / _ "); got != "" {
		t.Fatalf("expected empty for separator-only national ID, got %q", got)
	}
}

func TestNormalizePhone(t *testing.T) {
	cases := []struct {
		raw  string
		want string
	}{
		{"", ""},
		{"   ", ""},
		{"+84 912-345-678", "+84912345678"},
		{"+1 (415) 555.0199", "+14155550199"},
	}
	for _, tc := range cases {
		got, err := NormalizePhone(tc.raw)
		if err != nil || got != tc.want {
			t.Fatalf("NormalizePhone(%q) = (%q, %v), want (%q, nil)", tc.raw, got, err, tc.want)
		}
	}
	for _, bad := range []string{
		"0912345678",
		"0084912345678",
		"+84912345678 ext 101",
		"+84912345678x2",
		"+012345678",
		"+123456",
		"+1234567890123456",
		"++84912345678",
	} {
		if _, err := NormalizePhone(bad); !errors.Is(err, ErrInvalidPhone) {
			t.Fatalf("NormalizePhone(%q) err = %v, want ErrInvalidPhone", bad, err)
		}
	}
}

func TestNormalizeEmployeeCode(t *testing.T) {
	if got := NormalizeEmployeeCode("  001-ab c  "); got != "001-AB C" {
		t.Fatalf("NormalizeEmployeeCode = %q, want %q", got, "001-AB C")
	}
}

func TestHasherFailClosedAndRedacted(t *testing.T) {
	for _, bad := range []string{"", "   ", "too-short-secret"} {
		if _, err := NewHasher(bad); !errors.Is(err, ErrIdentityHashSecretMissing) {
			t.Fatalf("NewHasher(%q) err = %v, want ErrIdentityHashSecretMissing", bad, err)
		}
	}

	var zero Hasher
	if _, _, err := zero.HashNationalID("001234567"); !errors.Is(err, ErrIdentityHashSecretMissing) {
		t.Fatalf("zero Hasher.HashNationalID err = %v, want ErrIdentityHashSecretMissing", err)
	}
	if _, _, err := zero.HashPhone("+84912345678"); !errors.Is(err, ErrIdentityHashSecretMissing) {
		t.Fatalf("zero Hasher.HashPhone err = %v, want ErrIdentityHashSecretMissing", err)
	}

	h, err := NewHasher(testHMACSecret)
	if err != nil {
		t.Fatalf("NewHasher: %v", err)
	}
	if fmt.Sprint(h) != "[REDACTED]" || fmt.Sprintf("%#v", h) != "[REDACTED]" {
		t.Fatalf("Hasher leaked secret in formatting: %q / %#v", h.String(), h)
	}
}

func mustNewTestHasher(t *testing.T) Hasher {
	t.Helper()
	h, err := NewHasher(testHMACSecret)
	if err != nil {
		t.Fatalf("NewHasher: %v", err)
	}
	return h
}

func assertPresentHash(t *testing.T, label string, fn func() (string, bool, error)) string {
	t.Helper()
	hash, present, err := fn()
	if err != nil {
		t.Fatalf("%s unexpected err: %v", label, err)
	}
	if !present || len(hash) != 64 {
		t.Fatalf("%s = (%q, %v), want 64-char hash", label, hash, present)
	}
	return hash
}

func assertAbsentHash(t *testing.T, label string, fn func() (string, bool, error)) {
	t.Helper()
	hash, present, err := fn()
	if err != nil || present || hash != "" {
		t.Fatalf("%s must be absent, got (%q, %v, %v)", label, hash, present, err)
	}
}

func TestHasherNationalID(t *testing.T) {
	h := mustNewTestHasher(t)
	h1 := assertPresentHash(t, "nid1", func() (string, bool, error) { return h.HashNationalID("001-234-567") })
	h2 := assertPresentHash(t, "nid2", func() (string, bool, error) { return h.HashNationalID("００１ 234.567") })
	if h2 != h1 {
		t.Fatalf("equivalent normalized national IDs must hash identically: %q vs %q", h1, h2)
	}
	assertAbsentHash(t, "empty-nid", func() (string, bool, error) { return h.HashNationalID(" --- ") })
}

func TestHasherPhoneAndDomainSeparation(t *testing.T) {
	h := mustNewTestHasher(t)
	p1 := assertPresentHash(t, "phone1", func() (string, bool, error) { return h.HashPhone("+84 912-345-678") })
	p2 := assertPresentHash(t, "phone2", func() (string, bool, error) { return h.HashPhone("+84912345678") })
	if p2 != p1 {
		t.Fatalf("equivalent E.164 phones must hash identically")
	}
	assertAbsentHash(t, "empty-phone", func() (string, bool, error) { return h.HashPhone("  ") })
	if _, _, err := h.HashPhone("0912345678"); !errors.Is(err, ErrInvalidPhone) {
		t.Fatalf("national-only phone must fail with ErrInvalidPhone, got %v", err)
	}
	sameValNID := assertPresentHash(t, "same-val-nid", func() (string, bool, error) { return h.HashNationalID("+84912345678") })
	if strings.EqualFold(sameValNID, p1) {
		t.Fatal("national_id and phone hashes must be domain separated")
	}
}

func TestNormalizePartialPhoneAndSoftSuffix(t *testing.T) {
	valid := []struct {
		raw  string
		want string
	}{
		{"", ""},
		{"   ", ""},
		{"345-678", "345678"},
		{"(091) 234.5678", "0912345678"},
		{"5678", "5678"},
	}
	for _, tc := range valid {
		got, err := NormalizePartialPhone(tc.raw)
		if err != nil || got != tc.want {
			t.Fatalf("NormalizePartialPhone(%q) = (%q, %v), want (%q, nil)", tc.raw, got, err, tc.want)
		}
	}
	for _, bad := range []string{"123", "+84912345678", "345678 ext 1", "abcd", "1234567890123456"} {
		if _, err := NormalizePartialPhone(bad); !errors.Is(err, ErrInvalidPartialPhone) {
			t.Fatalf("NormalizePartialPhone(%q) err = %v, want ErrInvalidPartialPhone", bad, err)
		}
	}
	if got := PhoneSoftSuffix("+84912345678"); got != "345678" {
		t.Fatalf("PhoneSoftSuffix = %q, want 345678", got)
	}
	if got := PhoneSoftSuffix("+12345"); got != "12345" {
		t.Fatalf("PhoneSoftSuffix short = %q, want 12345", got)
	}
	if got := PhoneSoftSuffix("+12"); got != "" {
		t.Fatalf("PhoneSoftSuffix too short = %q, want empty", got)
	}
}
