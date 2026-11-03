package config

import (
	"crypto/ed25519"
	"encoding/base64"
	"fmt"
	"strings"
	"testing"
)

func minimumDevEnv() map[string]string {
	return map[string]string{
		"DB_PRIMARY_URL":            "postgres://user:pass@localhost:5432/db",
		"DB_POOLER_TX_URL":          "postgres://user:pass@localhost:6543/db",
		"DB_POOLER_SESS_URL":        "postgres://user:pass@localhost:5452/db",
		"REDIS_URL":                 "redis://localhost:6379",
		"ORGANIZATION_NAME":         "Example Company",
		"STANDALONE_ADMIN_EMAIL":    "admin@example.invalid",
		"STANDALONE_ADMIN_PASSWORD": "strong-password",
		"LICENSE_PATH":              "/secure/license.json",
		"LICENSE_PUBLIC_KEY":        base64.RawURLEncoding.EncodeToString(make([]byte, ed25519.PublicKeySize)),
	}
}

func mockLookup(env map[string]string) LookupEnvFunc {
	return func(key string) (string, bool) {
		val, ok := env[key]
		return val, ok
	}
}

func TestLoad_MinimumDevSuccess(t *testing.T) {
	cfg, err := LoadWithLookup(mockLookup(minimumDevEnv()))
	if err != nil {
		t.Fatalf("unexpected load error: %v", err)
	}
	if cfg.AppEnv != "development" || cfg.AppPort != 3000 || cfg.AppHost != "0.0.0.0" {
		t.Fatalf("unexpected basic config: %+v", cfg)
	}
	if cfg.DBPrimaryURL.Expose() != "postgres://user:pass@localhost:5432/db" {
		t.Errorf("unexpected DBPrimaryURL: %s", cfg.DBPrimaryURL.Expose())
	}
	if cfg.JWTSecret.Expose() != "dev_secret_key_leeattend_2026" {
		t.Errorf("unexpected development JWT secret")
	}
	if cfg.S3BucketName != "leeattend" {
		t.Errorf("expected default bucket 'leeattend', got %q", cfg.S3BucketName)
	}
	if cfg.RateLimitMaxRead != 120 || cfg.RateLimitMaxWrite != 30 {
		t.Errorf("expected rate limits 120/30, got %d/%d", cfg.RateLimitMaxRead, cfg.RateLimitMaxWrite)
	}
}

func TestLoad_StandaloneRequiresSettings(t *testing.T) {
	env := minimumDevEnv()
	delete(env, "ORGANIZATION_NAME")
	delete(env, "LICENSE_PUBLIC_KEY")
	_, err := LoadWithLookup(mockLookup(env))
	if err == nil || !strings.Contains(err.Error(), "ORGANIZATION_NAME") || !strings.Contains(err.Error(), "LICENSE_PUBLIC_KEY") {
		t.Fatalf("expected mandatory standalone validation errors, got %v", err)
	}
}

func TestLoad_StandaloneAcceptsValidConfig(t *testing.T) {
	cfg, err := LoadWithLookup(mockLookup(minimumDevEnv()))
	if err != nil {
		t.Fatalf("unexpected standalone config error: %v", err)
	}
	if cfg.OrganizationName != "Example Company" {
		t.Fatalf("expected organization name, got %q", cfg.OrganizationName)
	}
}

func TestLoad_MissingRequiredAlways(t *testing.T) {
	emptyEnv := map[string]string{}
	cfg, err := LoadWithLookup(mockLookup(emptyEnv))
	if err == nil {
		t.Fatalf("expected error for empty env, got cfg: %+v", cfg)
	}

	cfgErr, ok := err.(ConfigErrors)
	if !ok {
		t.Fatalf("expected ConfigErrors type, got %T: %v", err, err)
	}

	expectedKeys := map[string]bool{
		"DB_PRIMARY_URL":     false,
		"DB_POOLER_TX_URL":   false,
		"DB_POOLER_SESS_URL": false,
		"REDIS_URL":          false,
	}

	for _, e := range cfgErr {
		if _, exists := expectedKeys[e.Key]; exists {
			expectedKeys[e.Key] = true
		}
	}

	for k, found := range expectedKeys {
		if !found {
			t.Errorf("expected missing error for %s, none found", k)
		}
	}
}

func TestLoad_ProductionMissingRequired(t *testing.T) {
	env := minimumDevEnv()
	env["APP_ENV"] = "production"

	_, err := LoadWithLookup(mockLookup(env))
	if err == nil {
		t.Fatal("expected failure in production mode with missing prod vars")
	}

	cfgErr, ok := err.(ConfigErrors)
	if !ok {
		t.Fatalf("expected ConfigErrors type, got %T: %v", err, err)
	}

	expectedProdKeys := map[string]bool{
		"JWT_SECRET":           false,
		"ENCRYPTION_KEYS_JSON": false,
		"ALLOWED_ORIGINS":      false,
		"S3_ENDPOINT_URL":      false,
		"S3_BUCKET_NAME":       false,
		"S3_ACCESS_KEY":        false,
		"S3_SECRET_KEY":        false,
	}

	for _, e := range cfgErr {
		if _, exists := expectedProdKeys[e.Key]; exists {
			expectedProdKeys[e.Key] = true
		}
	}

	for k, found := range expectedProdKeys {
		if !found {
			t.Errorf("expected production error for %s, none found", k)
		}
	}
}

func TestLoad_ProductionValid(t *testing.T) {
	env := minimumDevEnv()
	env["APP_ENV"] = "production"
	env["JWT_SECRET"] = "production_super_secret_signing_key_at_least_32_bytes_long"
	env["ENCRYPTION_KEYS_JSON"] = `{"keys":{"v1":"0123456789012345678901234567890123456789012345678901234567890123"},"active_key":"v1"}`
	env["ALLOWED_ORIGINS"] = "https://attend.example.com,https://api.example.com"
	env["S3_ENDPOINT_URL"] = "https://s3.amazonaws.com"
	env["S3_BUCKET_NAME"] = "corp-attendance"
	env["S3_ACCESS_KEY"] = "prod-access-key-1234"
	env["S3_SECRET_KEY"] = "prod-secret-key-5678"

	cfg, err := LoadWithLookup(mockLookup(env))
	if err != nil {
		t.Fatalf("unexpected error for valid production config: %v", err)
	}

	if !cfg.IsProduction() {
		t.Errorf("expected IsProduction() to be true")
	}
	if len(cfg.AllowedOrigins) != 2 {
		t.Errorf("expected 2 allowed origins, got %d", len(cfg.AllowedOrigins))
	}
	activeKey, ok := cfg.EncryptionKeys.GetActiveKey()
	if !ok || activeKey == "" {
		t.Errorf("expected active encryption key to resolve")
	}
}

func TestLoad_JWTSecretShortInProduction(t *testing.T) {
	env := minimumDevEnv()
	env["APP_ENV"] = "production"
	env["JWT_SECRET"] = "too_short"
	env["ENCRYPTION_KEYS_JSON"] = `{"keys":{"v1":"k"},"active_key":"v1"}`
	env["ALLOWED_ORIGINS"] = "https://app.com"
	env["S3_ENDPOINT_URL"] = "https://s3.amazonaws.com"
	env["S3_BUCKET_NAME"] = "b"
	env["S3_ACCESS_KEY"] = "a"
	env["S3_SECRET_KEY"] = "s"

	_, err := LoadWithLookup(mockLookup(env))
	if err == nil {
		t.Fatal("expected error for short JWT secret in production")
	}
	if !strings.Contains(err.Error(), "at least 32 bytes") {
		t.Errorf("expected error message about 32 bytes, got %v", err)
	}
	// Verify no secret value leaked in error message
	if strings.Contains(err.Error(), "too_short") {
		t.Errorf("secret leaked in error message: %s", err.Error())
	}
}

func TestLoad_InvalidURLPrefixes(t *testing.T) {
	env := minimumDevEnv()
	env["DB_PRIMARY_URL"] = "mysql://user:pass@localhost:3306/db"
	env["REDIS_URL"] = "http://localhost:6379"

	_, err := LoadWithLookup(mockLookup(env))
	if err == nil {
		t.Fatal("expected failure on invalid URL prefixes")
	}

	msg := err.Error()
	if !strings.Contains(msg, "must start with 'postgres://'") {
		t.Errorf("expected postgres prefix error, got %s", msg)
	}
	if !strings.Contains(msg, "must start with 'redis://'") {
		t.Errorf("expected redis prefix error, got %s", msg)
	}
	// Ensure passwords are not leaked in error
	if strings.Contains(msg, "pass") {
		t.Errorf("credential leaked in error message: %s", msg)
	}
}

func TestLoad_AliasResolution(t *testing.T) {
	env := minimumDevEnv()
	env["DATABASE_URL"] = env["DB_PRIMARY_URL"]
	env["DATABASE_POOLER_TX_URL"] = env["DB_POOLER_TX_URL"]
	env["DATABASE_POOLER_SESS_URL"] = env["DB_POOLER_SESS_URL"]
	delete(env, "DB_PRIMARY_URL")
	delete(env, "DB_POOLER_TX_URL")
	delete(env, "DB_POOLER_SESS_URL")
	env["API_PORT"] = "8080"
	env["API_HOST"] = "127.0.0.1"
	env["PGPORT"] = "6432"

	cfg, err := LoadWithLookup(mockLookup(env))
	if err != nil {
		t.Fatalf("unexpected error resolving aliases: %v", err)
	}

	if cfg.AppPort != 8080 {
		t.Errorf("expected API_PORT 8080, got %d", cfg.AppPort)
	}
	if cfg.AppHost != "127.0.0.1" {
		t.Errorf("expected API_HOST '127.0.0.1', got %q", cfg.AppHost)
	}
	if cfg.PgBouncerPort != 6432 {
		t.Errorf("expected PGPORT 6432, got %d", cfg.PgBouncerPort)
	}
	if cfg.DBPrimaryURL.Expose() != "postgres://user:pass@localhost:5432/db" {
		t.Errorf("expected DB_PRIMARY_URL resolved from DATABASE_URL")
	}
}

func TestSecret_StringerMasking(t *testing.T) {
	jwt := JWTSecret("my-super-secret-jwt")
	enc := EncryptionKeysJSON(`{"active_key":"v1","keys":{"v1":"abc"}}`)
	s3Sec := S3SecretKey("s3-top-secret")
	red := RedisURL("redis://:secretpass@localhost:6379")
	conn := ConnectionURL("postgres://admin:topsecret@db:5432/main")
	keys := EncryptionKeys{ActiveKey: "v1", Keys: map[string]string{"v1": "abc"}}

	tests := []struct {
		name string
		val  any
	}{
		{"JWTSecret", jwt},
		{"EncryptionKeysJSON", enc},
		{"S3SecretKey", s3Sec},
		{"RedisURL", red},
		{"ConnectionURL", conn},
		{"EncryptionKeys", keys},
	}

	for _, tc := range tests {
		s := fmt.Sprintf("%s", tc.val)
		if s != "[REDACTED]" {
			t.Errorf("%s String() returned %q, expected '[REDACTED]'", tc.name, s)
		}
		v := fmt.Sprintf("%v", tc.val)
		if v != "[REDACTED]" {
			t.Errorf("%s %%v returned %q, expected '[REDACTED]'", tc.name, v)
		}
		goStr := fmt.Sprintf("%#v", tc.val)
		if goStr != "[REDACTED]" {
			t.Errorf("%s GoString() returned %q, expected '[REDACTED]'", tc.name, goStr)
		}
	}
}

func TestLoad_NumericalBounds(t *testing.T) {
	env := minimumDevEnv()
	env["APP_PORT"] = "99999" // out of u16 range
	env["WORKER_MAX_CONCURRENT_JOBS"] = "0"
	env["RATE_LIMIT_WINDOW_SECS"] = "-5"

	_, err := LoadWithLookup(mockLookup(env))
	if err == nil {
		t.Fatal("expected failure on out-of-bounds numbers")
	}
	msg := err.Error()
	if !strings.Contains(msg, "APP_PORT") {
		t.Errorf("expected APP_PORT bound error, got %s", msg)
	}
	if !strings.Contains(msg, "WORKER_MAX_CONCURRENT_JOBS") {
		t.Errorf("expected WORKER_MAX_CONCURRENT_JOBS bound error, got %s", msg)
	}
}

func TestRedactedSummary_MasksPasswords(t *testing.T) {
	env := minimumDevEnv()
	env["DB_PRIMARY_URL"] = "postgres://admin:supersecretpassword@db.internal:5432/leeattend"
	cfg, err := LoadWithLookup(mockLookup(env))
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}

	summary := cfg.RedactedSummary()
	if strings.Contains(summary, "supersecretpassword") {
		t.Errorf("password leaked in summary: %s", summary)
	}
	if !strings.Contains(summary, "admin:***@db.internal:5432/leeattend") {
		t.Errorf("expected masked URL format in summary, got: %s", summary)
	}
}
