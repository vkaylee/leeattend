package auth

import (
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"errors"
	"fmt"
	"strconv"
	"strings"

	"golang.org/x/crypto/argon2"
)

var (
	ErrInvalidHash         = errors.New("invalid password hash format")
	ErrIncompatibleVersion = errors.New("incompatible argon2 version")
)

// Default OWASP-compliant Argon2id parameters
const (
	Argon2Time    = 1
	Argon2Memory  = 64 * 1024 // 64 MB
	Argon2Threads = 4
	Argon2KeyLen  = 32
	SaltLen       = 16
)

// HashPassword generates a standard PHC-formatted Argon2id hash string ($argon2id$v=19$m=...,t=...,p=...$salt$hash).
func HashPassword(password string) (string, error) {
	salt := make([]byte, SaltLen)
	if _, err := rand.Read(salt); err != nil {
		return "", fmt.Errorf("generate salt: %w", err)
	}

	hash := argon2.IDKey([]byte(password), salt, Argon2Time, Argon2Memory, Argon2Threads, Argon2KeyLen)

	b64Salt := base64.RawStdEncoding.EncodeToString(salt)
	b64Hash := base64.RawStdEncoding.EncodeToString(hash)

	return fmt.Sprintf("$argon2id$v=%d$m=%d,t=%d,p=%d$%s$%s", argon2.Version, Argon2Memory, Argon2Time, Argon2Threads, b64Salt, b64Hash), nil
}

// VerifyPassword checks password against a PHC-formatted Argon2id hash.
// Returns (is_valid, needs_rehash).
func VerifyPassword(password, encodedHash string) (bool, bool) {
	if !strings.HasPrefix(encodedHash, "$argon2id$") {
		return false, false
	}

	parts := strings.Split(encodedHash, "$")
	if len(parts) != 6 {
		return false, false
	}

	var version int
	if _, err := fmt.Sscanf(parts[2], "v=%d", &version); err != nil || version != argon2.Version {
		return false, false
	}

	var memory uint32
	var time uint32
	var threads uint8
	var p uint32
	for _, param := range strings.Split(parts[3], ",") {
		kv := strings.Split(param, "=")
		if len(kv) != 2 {
			continue
		}
		val, err := strconv.ParseUint(kv[1], 10, 32)
		if err != nil {
			continue
		}
		switch kv[0] {
		case "m":
			memory = uint32(val)
		case "t":
			time = uint32(val)
		case "p":
			p = uint32(val)
			threads = uint8(p)
		}
	}
	if memory == 0 || time == 0 || threads == 0 {
		return false, false
	}

	salt, err := base64.RawStdEncoding.DecodeString(parts[4])
	if err != nil {
		return false, false
	}

	expectedHash, err := base64.RawStdEncoding.DecodeString(parts[5])
	if err != nil {
		return false, false
	}

	derivedHash := argon2.IDKey([]byte(password), salt, time, memory, threads, uint32(len(expectedHash)))
	if subtle.ConstantTimeCompare(derivedHash, expectedHash) == 1 {
		needsRehash := memory != Argon2Memory || time != Argon2Time || threads != Argon2Threads
		return true, needsRehash
	}

	return false, false
}
