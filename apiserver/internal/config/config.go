package config

import (
	"crypto/ed25519"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"net/url"
	"os"
	"strconv"
	"strings"
	"time"
)

// Secret wraps sensitive string data to prevent accidental leakage in logs and formatters.
type Secret string

func (s Secret) String() string   { return "[REDACTED]" }
func (s Secret) GoString() string { return "[REDACTED]" }
func (s Secret) Expose() string   { return string(s) }

// JWTSecret holds the JWT signing secret.
type JWTSecret Secret

func (s JWTSecret) String() string   { return "[REDACTED]" }
func (s JWTSecret) GoString() string { return "[REDACTED]" }
func (s JWTSecret) Expose() string   { return string(s) }

// EncryptionKeysJSON holds the raw JSON-encoded encryption keys string.
type EncryptionKeysJSON Secret

func (s EncryptionKeysJSON) String() string   { return "[REDACTED]" }
func (s EncryptionKeysJSON) GoString() string { return "[REDACTED]" }
func (s EncryptionKeysJSON) Expose() string   { return string(s) }

// S3SecretKey holds S3 / MinIO secret access key or access key.
type S3SecretKey Secret

func (s S3SecretKey) String() string   { return "[REDACTED]" }
func (s S3SecretKey) GoString() string { return "[REDACTED]" }
func (s S3SecretKey) Expose() string   { return string(s) }

// RedisURL holds the Redis connection string.
type RedisURL Secret

func (s RedisURL) String() string   { return "[REDACTED]" }
func (s RedisURL) GoString() string { return "[REDACTED]" }
func (s RedisURL) Expose() string   { return string(s) }

// ConnectionURL holds a database connection string which may embed credentials.
type ConnectionURL Secret

func (s ConnectionURL) String() string   { return "[REDACTED]" }
func (s ConnectionURL) GoString() string { return "[REDACTED]" }
func (s ConnectionURL) Expose() string   { return string(s) }

// EncryptionKeys represents the decoded key-rotation structure for database URL encryption.
type EncryptionKeys struct {
	Keys      map[string]string `json:"keys"`
	ActiveKey string            `json:"active_key"`
}

func (e EncryptionKeys) String() string   { return "[REDACTED]" }
func (e EncryptionKeys) GoString() string { return "[REDACTED]" }

// GetActiveKey returns the active encryption key string.
func (e *EncryptionKeys) GetActiveKey() (string, bool) {
	if e == nil || e.Keys == nil {
		return "", false
	}
	k, ok := e.Keys[e.ActiveKey]
	return k, ok
}

// ValidationError records a configuration failure without echoing raw secret input.
type ValidationError struct {
	Key     string
	Message string
}

func (e ValidationError) Error() string {
	return fmt.Sprintf("%s: %s", e.Key, e.Message)
}

// ConfigErrors is a multi-error container for configuration validation failures.
type ConfigErrors []ValidationError

func (ce ConfigErrors) Error() string {
	if len(ce) == 0 {
		return ""
	}
	msgs := make([]string, len(ce))
	for i, e := range ce {
		msgs[i] = e.Error()
	}
	return strings.Join(msgs, "; ")
}

// Config holds all 47 canonical settings for the LeeAttend API server.
type Config struct {
	// 2.1 Core
	AppEnv       string
	AppPort      int
	AppHost      string
	PortFileName string

	// 2.2 Database
	DBPrimaryURL               ConnectionURL
	DBPoolerTxURL              ConnectionURL
	DBPoolerSessURL            ConnectionURL
	DBReaderURL                ConnectionURL
	DBAPIMaxConn               int
	DBAPIAcquireTimeoutSecs    int
	DBWorkerAcquireTimeoutSecs int
	PgBouncerHost              string
	PgBouncerPort              int

	// 2.3 Redis
	RedisURL RedisURL

	// 2.4 Security
	JWTSecret              JWTSecret
	EncryptionKeysJSON     EncryptionKeysJSON
	EncryptionKeys         EncryptionKeys
	AllowedOrigins         []string
	CookieDomain           string
	MaxRequestBodyBytes    int64
	MaxFileUploadBodyBytes int64
	MaxBulkUploadBodyBytes int64

	// 2.5 Storage (S3 / MinIO)
	S3EndpointURL           string
	S3BucketName            string
	S3AccessKey             S3SecretKey
	S3SecretKey             S3SecretKey
	S3Region                string
	S3UsePathStyle          bool
	FileProxyThresholdBytes int64
	PresignedURLTTLSecs     int
	StorageDir              string

	// 2.6 Worker
	WorkerMaxConcurrentJobs  int
	WorkerPollTimeoutSecs    int
	WorkerMaxRetries         int
	WorkerJobTimeoutSecs     int
	ImportChunkSize          int
	PairingSweepIntervalSecs int

	// 2.7 Rate Limiting
	RateLimitMaxRead    int
	RateLimitMaxWrite   int
	RateLimitWindowSecs int

	// 2.8 Virus Scanning
	ClamdHost string
	ClamdPort int

	// 2.11 Feature Flags
	FFNoShowTimeline bool

	// 2.12 Standalone deployment
	OrganizationName        string
	StandaloneAdminEmail    string
	StandaloneAdminPassword Secret
	LicensePath             string
	LicensePublicKey        string
}

// IsProduction returns true if APP_ENV is "production".
func (c *Config) IsProduction() bool {
	return c.AppEnv == "production"
}

// IsDevelopment returns true if APP_ENV is "development".
func (c *Config) IsDevelopment() bool {
	return c.AppEnv == "development"
}

// IsTesting returns true if APP_ENV is "test" or "testing".
func (c *Config) IsTesting() bool {
	return c.AppEnv == "test" || c.AppEnv == "testing"
}

// WorkerPollTimeout returns the worker poll timeout as a time.Duration.
func (c *Config) WorkerPollTimeout() time.Duration {
	return time.Duration(c.WorkerPollTimeoutSecs) * time.Second
}

// WorkerJobTimeout returns the worker job timeout as a time.Duration.
func (c *Config) WorkerJobTimeout() time.Duration {
	return time.Duration(c.WorkerJobTimeoutSecs) * time.Second
}

// PresignedURLTTL returns the S3 presigned URL TTL as a time.Duration.
func (c *Config) PresignedURLTTL() time.Duration {
	return time.Duration(c.PresignedURLTTLSecs) * time.Second
}

// RateLimitWindow returns the rate limit window as a time.Duration.
func (c *Config) RateLimitWindow() time.Duration {
	return time.Duration(c.RateLimitWindowSecs) * time.Second
}

// LookupEnvFunc abstracts reading environment variables for test isolation.
type LookupEnvFunc func(key string) (string, bool)

// Load reads configuration from process environment variables.
func Load() (*Config, error) {
	return LoadWithLookup(os.LookupEnv)
}

// LoadWithLookup reads and validates configuration using a supplied lookup function.
func LoadWithLookup(lookup LookupEnvFunc) (*Config, error) {
	var errs []ValidationError

	addErr := func(key, msg string) {
		errs = append(errs, ValidationError{Key: key, Message: msg})
	}

	get := func(key string, aliases ...string) (string, bool) {
		if val, ok := lookup(key); ok {
			trimmed := strings.TrimSpace(val)
			if trimmed != "" {
				return trimmed, true
			}
		}
		for _, alias := range aliases {
			if val, ok := lookup(alias); ok {
				trimmed := strings.TrimSpace(val)
				if trimmed != "" {
					return trimmed, true
				}
			}
		}
		return "", false
	}

	getInt := func(key string, defaultVal int, minVal, maxVal int, aliases ...string) int {
		raw, ok := get(key, aliases...)
		if !ok {
			return defaultVal
		}
		val, err := strconv.Atoi(raw)
		if err != nil {
			addErr(key, fmt.Sprintf("must be a valid integer, got %q", raw))
			return defaultVal
		}
		if val < minVal || val > maxVal {
			addErr(key, fmt.Sprintf("must be between %d and %d, got %d", minVal, maxVal, val))
			return defaultVal
		}
		return val
	}

	getInt64 := func(key string, defaultVal int64, minVal int64, aliases ...string) int64 {
		raw, ok := get(key, aliases...)
		if !ok {
			return defaultVal
		}
		val, err := strconv.ParseInt(raw, 10, 64)
		if err != nil {
			addErr(key, fmt.Sprintf("must be a valid integer, got %q", raw))
			return defaultVal
		}
		if val < minVal {
			addErr(key, fmt.Sprintf("must be at least %d, got %d", minVal, val))
			return defaultVal
		}
		return val
	}

	getBool := func(key string, defaultVal bool, aliases ...string) bool {
		raw, ok := get(key, aliases...)
		if !ok {
			return defaultVal
		}
		switch strings.ToLower(raw) {
		case "true", "1", "yes", "on":
			return true
		case "false", "0", "no", "off":
			return false
		default:
			addErr(key, fmt.Sprintf("must be a boolean (true/false), got %q", raw))
			return defaultVal
		}
	}

	// 2.1 Core Category
	appEnv, _ := get("APP_ENV", "API_ENV")
	if appEnv == "" {
		appEnv = "development"
	}
	switch appEnv {
	case "development", "production", "test", "testing":
	default:
		addErr("APP_ENV", fmt.Sprintf("must be one of: development, production, test, testing; got %q", appEnv))
	}
	isProd := appEnv == "production"

	appPort := getInt("APP_PORT", 3000, 1, 65535, "API_PORT")

	appHost, ok := get("APP_HOST", "API_HOST")
	if !ok {
		appHost = "0.0.0.0"
	}

	portFileName, ok := get("PORT_FILE_NAME")
	if !ok {
		portFileName = ".api_port"
	}

	// 2.2 Database Category
	validatePostgresURL := func(val, key string) {
		if !strings.HasPrefix(val, "postgres://") && !strings.HasPrefix(val, "postgresql://") {
			addErr(key, "must start with 'postgres://' or 'postgresql://'")
		}
	}

	dbPrimaryRaw, ok := get("DB_PRIMARY_URL", "DB_URL", "DATABASE_URL")
	if !ok {
		addErr("DB_PRIMARY_URL", "is required but not set (aliases: DB_URL, DATABASE_URL)")
	} else {
		validatePostgresURL(dbPrimaryRaw, "DB_PRIMARY_URL")
	}

	dbPoolerTxRaw, ok := get("DB_POOLER_TX_URL", "DATABASE_POOLER_TX_URL")
	if !ok {
		addErr("DB_POOLER_TX_URL", "is required but not set (aliases: DATABASE_POOLER_TX_URL)")
	} else {
		validatePostgresURL(dbPoolerTxRaw, "DB_POOLER_TX_URL")
	}

	dbPoolerSessRaw, ok := get("DB_POOLER_SESS_URL", "DATABASE_POOLER_SESS_URL")
	if !ok {
		addErr("DB_POOLER_SESS_URL", "is required but not set (aliases: DATABASE_POOLER_SESS_URL)")
	} else {
		validatePostgresURL(dbPoolerSessRaw, "DB_POOLER_SESS_URL")
	}

	dbReaderRaw, ok := get("DB_READER_URL", "DATABASE_READER_URL")
	if !ok {
		dbReaderRaw = dbPoolerTxRaw
	} else {
		validatePostgresURL(dbReaderRaw, "DB_READER_URL")
	}

	dbAPIMaxConn := getInt("DB_API_MAX_CONN", 20, 1, 1000)
	dbAPIAcquireTimeoutSecs := getInt("DB_API_ACQUIRE_TIMEOUT_SECS", 5, 1, 3600)
	dbWorkerAcquireTimeoutSecs := getInt("DB_WORKER_ACQUIRE_TIMEOUT_SECS", 30, 1, 3600)

	pgBouncerHost, ok := get("PGBOUNCER_HOST", "DATABASE_HOST")
	if !ok {
		pgBouncerHost = "localhost"
	}
	pgBouncerPort := getInt("PGBOUNCER_PORT", 5432, 1, 65535, "PGPORT")

	// 2.3 Redis Category
	redisRaw, ok := get("REDIS_URL")
	if !ok {
		addErr("REDIS_URL", "is required but not set")
	} else {
		if !strings.HasPrefix(redisRaw, "redis://") && !strings.HasPrefix(redisRaw, "rediss://") {
			addErr("REDIS_URL", "must start with 'redis://' or 'rediss://'")
		}
	}

	// 2.4 Security Category
	jwtSecretRaw, ok := get("JWT_SECRET")
	if !ok {
		if isProd {
			addErr("JWT_SECRET", "is required in production")
		} else {
			jwtSecretRaw = "dev_secret_key_leeattend_2026"
		}
	} else if isProd && len(jwtSecretRaw) < 32 {
		addErr("JWT_SECRET", "must be at least 32 bytes in production")
	}

	encryptionKeysRaw, ok := get("ENCRYPTION_KEYS_JSON")
	var parsedKeys EncryptionKeys
	if !ok {
		if isProd {
			addErr("ENCRYPTION_KEYS_JSON", "is required in production")
		} else {
			encryptionKeysRaw = `{"keys":{"v1":"0123456789012345678901234567890123456789012345678901234567890123"},"active_key":"v1"}`
			_ = json.Unmarshal([]byte(encryptionKeysRaw), &parsedKeys)
		}
	} else {
		if err := json.Unmarshal([]byte(encryptionKeysRaw), &parsedKeys); err != nil {
			addErr("ENCRYPTION_KEYS_JSON", "must be valid JSON matching encryption keys schema")
		} else if parsedKeys.ActiveKey == "" {
			addErr("ENCRYPTION_KEYS_JSON", "missing required 'active_key' field")
		} else if len(parsedKeys.Keys) == 0 {
			addErr("ENCRYPTION_KEYS_JSON", "missing required 'keys' map")
		} else if _, found := parsedKeys.Keys[parsedKeys.ActiveKey]; !found {
			addErr("ENCRYPTION_KEYS_JSON", fmt.Sprintf("active_key %q not found in keys map", parsedKeys.ActiveKey))
		}
	}

	allowedOriginsRaw, ok := get("ALLOWED_ORIGINS")
	var allowedOrigins []string
	if !ok {
		if isProd {
			addErr("ALLOWED_ORIGINS", "is required in production")
		} else {
			allowedOriginsRaw = "http://localhost:5173,http://localhost:3000,http://127.0.0.1:5173,http://127.0.0.1:3000"
		}
	}
	if allowedOriginsRaw != "" {
		for _, part := range strings.Split(allowedOriginsRaw, ",") {
			trimmed := strings.TrimSpace(part)
			if trimmed != "" {
				allowedOrigins = append(allowedOrigins, trimmed)
			}
		}
	}

	cookieDomain, _ := get("COOKIE_DOMAIN")
	maxRequestBodyBytes := getInt64("MAX_REQUEST_BODY_BYTES", 2097152, 1)
	maxFileUploadBodyBytes := getInt64("MAX_FILE_UPLOAD_BODY_BYTES", 20971520, 1)
	maxBulkUploadBodyBytes := getInt64("MAX_BULK_UPLOAD_BODY_BYTES", 10485760, 1)

	// 2.5 Storage Category
	s3EndpointURL, ok := get("S3_ENDPOINT_URL")
	if !ok {
		if isProd {
			addErr("S3_ENDPOINT_URL", "is required in production")
		} else {
			s3EndpointURL = "http://localhost:9000"
		}
	} else {
		if u, err := url.Parse(s3EndpointURL); err != nil || (u.Scheme != "http" && u.Scheme != "https") {
			addErr("S3_ENDPOINT_URL", "must be a valid HTTP/HTTPS URL")
		}
	}

	s3BucketName, ok := get("S3_BUCKET_NAME")
	if !ok {
		if isProd {
			addErr("S3_BUCKET_NAME", "is required in production")
		} else {
			s3BucketName = "leeattend"
		}
	}

	s3AccessKeyRaw, ok := get("S3_ACCESS_KEY")
	if !ok {
		if isProd {
			addErr("S3_ACCESS_KEY", "is required in production")
		} else {
			s3AccessKeyRaw = "minioadmin"
		}
	}

	s3SecretKeyRaw, ok := get("S3_SECRET_KEY")
	if !ok {
		if isProd {
			addErr("S3_SECRET_KEY", "is required in production")
		} else {
			s3SecretKeyRaw = "minioadmin"
		}
	}

	s3Region, ok := get("S3_REGION")
	if !ok {
		s3Region = "us-east-1"
	}
	s3UsePathStyle := getBool("S3_USE_PATH_STYLE", true)
	fileProxyThresholdBytes := getInt64("FILE_PROXY_THRESHOLD_BYTES", 1048576, 1)
	presignedURLTTLSecs := getInt("PRESIGNED_URL_TTL_SECS", 900, 1, 86400)

	storageDir, ok := get("STORAGE_DIR")
	if !ok {
		storageDir = "storage"
	}

	// 2.6 Worker Category
	workerMaxConcurrentJobs := getInt("WORKER_MAX_CONCURRENT_JOBS", 5, 1, 100)
	workerPollTimeoutSecs := getInt("WORKER_POLL_TIMEOUT_SECS", 30, 1, 300)
	workerMaxRetries := getInt("WORKER_MAX_RETRIES", 3, 0, 20)
	workerJobTimeoutSecs := getInt("WORKER_JOB_TIMEOUT_SECS", 300, 1, 3600)
	importChunkSize := getInt("IMPORT_CHUNK_SIZE", 500, 1, 10000)
	pairingSweepIntervalSecs := getInt("PAIRING_SWEEP_INTERVAL_SECS", 2, 1, 3600)

	// 2.7 Rate Limiting Category
	rateLimitMaxRead := getInt("RATE_LIMIT_MAX_READ", 120, 1, 1000000)
	rateLimitMaxWrite := getInt("RATE_LIMIT_MAX_WRITE", 30, 1, 1000000)
	rateLimitWindowSecs := getInt("RATE_LIMIT_WINDOW_SECS", 60, 1, 86400)

	// 2.8 Virus Scanning Category
	clamdHost, _ := get("CLAMD_HOST")
	clamdPort := getInt("CLAMD_PORT", 3310, 1, 65535)

	// Standalone deployment is mandatory.
	licensePath, _ := get("LICENSE_PATH")
	licensePublicKey, _ := get("LICENSE_PUBLIC_KEY")
	organizationName, _ := get("ORGANIZATION_NAME")
	standaloneAdminEmail, _ := get("STANDALONE_ADMIN_EMAIL")
	standaloneAdminPassword, _ := get("STANDALONE_ADMIN_PASSWORD")
	if organizationName == "" {
		addErr("ORGANIZATION_NAME", "is required")
	}
	if standaloneAdminEmail == "" {
		addErr("STANDALONE_ADMIN_EMAIL", "is required")
	}
	if standaloneAdminPassword == "" {
		addErr("STANDALONE_ADMIN_PASSWORD", "is required")
	}
	if licensePath == "" {
		addErr("LICENSE_PATH", "is required")
	}
	if licensePublicKey == "" {
		addErr("LICENSE_PUBLIC_KEY", "is required")
	} else if decoded, decodeErr := base64.RawURLEncoding.DecodeString(licensePublicKey); decodeErr != nil || len(decoded) != ed25519.PublicKeySize {
		addErr("LICENSE_PUBLIC_KEY", "must be a base64url Ed25519 public key")
	}

	// 2.11 Feature Flags Category
	ffNoShowTimeline := getBool("FF_NO_SHOW_TIMELINE", false)

	if len(errs) > 0 {
		return nil, ConfigErrors(errs)
	}

	return &Config{
		AppEnv:                     appEnv,
		AppPort:                    appPort,
		AppHost:                    appHost,
		PortFileName:               portFileName,
		DBPrimaryURL:               ConnectionURL(dbPrimaryRaw),
		DBPoolerTxURL:              ConnectionURL(dbPoolerTxRaw),
		DBPoolerSessURL:            ConnectionURL(dbPoolerSessRaw),
		DBReaderURL:                ConnectionURL(dbReaderRaw),
		DBAPIMaxConn:               dbAPIMaxConn,
		DBAPIAcquireTimeoutSecs:    dbAPIAcquireTimeoutSecs,
		DBWorkerAcquireTimeoutSecs: dbWorkerAcquireTimeoutSecs,
		PgBouncerHost:              pgBouncerHost,
		PgBouncerPort:              pgBouncerPort,
		RedisURL:                   RedisURL(redisRaw),
		JWTSecret:                  JWTSecret(jwtSecretRaw),
		EncryptionKeysJSON:         EncryptionKeysJSON(encryptionKeysRaw),
		EncryptionKeys:             parsedKeys,
		AllowedOrigins:             allowedOrigins,
		CookieDomain:               cookieDomain,
		MaxRequestBodyBytes:        maxRequestBodyBytes,
		MaxFileUploadBodyBytes:     maxFileUploadBodyBytes,
		MaxBulkUploadBodyBytes:     maxBulkUploadBodyBytes,
		S3EndpointURL:              s3EndpointURL,
		S3BucketName:               s3BucketName,
		S3AccessKey:                S3SecretKey(s3AccessKeyRaw),
		S3SecretKey:                S3SecretKey(s3SecretKeyRaw),
		S3Region:                   s3Region,
		S3UsePathStyle:             s3UsePathStyle,
		FileProxyThresholdBytes:    fileProxyThresholdBytes,
		PresignedURLTTLSecs:        presignedURLTTLSecs,
		StorageDir:                 storageDir,
		WorkerMaxConcurrentJobs:    workerMaxConcurrentJobs,
		WorkerPollTimeoutSecs:      workerPollTimeoutSecs,
		WorkerMaxRetries:           workerMaxRetries,
		WorkerJobTimeoutSecs:       workerJobTimeoutSecs,
		ImportChunkSize:            importChunkSize,
		PairingSweepIntervalSecs:   pairingSweepIntervalSecs,
		RateLimitMaxRead:           rateLimitMaxRead,
		RateLimitMaxWrite:          rateLimitMaxWrite,
		RateLimitWindowSecs:        rateLimitWindowSecs,
		ClamdHost:                  clamdHost,
		ClamdPort:                  clamdPort,
		FFNoShowTimeline:           ffNoShowTimeline,
		OrganizationName:           organizationName,
		StandaloneAdminEmail:       standaloneAdminEmail,
		StandaloneAdminPassword:    Secret(standaloneAdminPassword),
		LicensePath:                licensePath,
		LicensePublicKey:           licensePublicKey,
	}, nil
}

// RedactedSummary returns a safe, redacted multi-line summary of settings for startup logging.
func (c *Config) RedactedSummary() string {
	var b strings.Builder
	b.WriteString("=== LeeAttend API Server Configuration ===\n")
	b.WriteString(fmt.Sprintf("Environment: %s\n", c.AppEnv))
	b.WriteString(fmt.Sprintf("Bind: %s:%d\n", c.AppHost, c.AppPort))
	b.WriteString(fmt.Sprintf("DB Primary: %s\n", maskConnectionURL(string(c.DBPrimaryURL))))
	b.WriteString(fmt.Sprintf("DB Pooler TX: %s\n", maskConnectionURL(string(c.DBPoolerTxURL))))
	b.WriteString(fmt.Sprintf("DB Pooler SESS: %s\n", maskConnectionURL(string(c.DBPoolerSessURL))))
	b.WriteString(fmt.Sprintf("Redis: %s\n", maskConnectionURL(string(c.RedisURL))))
	b.WriteString(fmt.Sprintf("Allowed Origins: %s\n", strings.Join(c.AllowedOrigins, ", ")))
	b.WriteString(fmt.Sprintf("S3 Endpoint: %s (Bucket: %s)\n", c.S3EndpointURL, c.S3BucketName))
	b.WriteString(fmt.Sprintf("Worker Concurrency: %d, Retries: %d\n", c.WorkerMaxConcurrentJobs, c.WorkerMaxRetries))
	b.WriteString(fmt.Sprintf("Rate Limit: Read %d / Write %d per %ds\n", c.RateLimitMaxRead, c.RateLimitMaxWrite, c.RateLimitWindowSecs))
	b.WriteString("==========================================\n")
	return b.String()
}

// maskConnectionURL replaces the password in a connection URL with '***'.
func maskConnectionURL(raw string) string {
	if raw == "" {
		return "<empty>"
	}
	atPos := strings.Index(raw, "@")
	if atPos == -1 {
		if len(raw) > 60 {
			return raw[:57] + "..."
		}
		return raw
	}
	userPass := raw[:atPos]
	schemeEnd := strings.Index(raw, "://")
	if schemeEnd != -1 {
		afterScheme := userPass[schemeEnd+3:]
		colonPos := strings.Index(afterScheme, ":")
		if colonPos != -1 {
			before := userPass[:schemeEnd+3+colonPos+1]
			after := raw[atPos:]
			return before + "***" + after
		}
	}
	return raw[:atPos] + "***" + raw[atPos:]
}
