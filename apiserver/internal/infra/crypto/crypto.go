package crypto

import (
	"crypto/aes"
	"crypto/cipher"
	"crypto/rand"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"strings"
)

var (
	ErrMissingActiveKey = errors.New("crypto: missing active key")
	ErrKeyNotFound      = errors.New("crypto: key version not found")
	ErrInvalidFormat    = errors.New("crypto: invalid format")
	ErrEncryptionFailed = errors.New("crypto: encryption failed")
	ErrDecryptionFailed = errors.New("crypto: decryption failed")
	ErrInvalidKeyLength = errors.New("crypto: invalid key length")
)

// Service provides AES-256-GCM encryption and decryption with key rotation matching Rust CryptoService.
type Service struct {
	keys        map[string][]byte
	activeKeyID string
}

// NewService creates a new Service with a designated active key.
func NewService(activeKeyID string, keys map[string][]byte) (*Service, error) {
	if _, ok := keys[activeKeyID]; !ok {
		return nil, ErrMissingActiveKey
	}
	return &Service{
		keys:        keys,
		activeKeyID: activeKeyID,
	}, nil
}

// FromJSON parses a JSON key configuration string formatted as {"active_key": "v1", "keys": {"v1": "..."}}.
func FromJSON(configJSON string) (*Service, error) {
	var parsed struct {
		ActiveKey string            `json:"active_key"`
		Keys      map[string]string `json:"keys"`
	}
	if err := json.Unmarshal([]byte(configJSON), &parsed); err != nil {
		return nil, ErrInvalidFormat
	}
	if parsed.ActiveKey == "" {
		return nil, ErrInvalidFormat
	}
	keys := make(map[string][]byte, len(parsed.Keys))
	for k, v := range parsed.Keys {
		var keyBytes []byte
		if b, err := base64.StdEncoding.DecodeString(v); err == nil && len(b) == 32 {
			keyBytes = b
		} else if h, err := hex.DecodeString(v); err == nil && len(h) == 32 {
			keyBytes = h
		} else if len(v) == 32 {
			keyBytes = []byte(v)
		} else {
			return nil, ErrInvalidKeyLength
		}
		keys[k] = keyBytes
	}
	return NewService(parsed.ActiveKey, keys)
}

// Encrypt encrypts plaintext using AES-256-GCM and returns version:nonce_b64:ciphertext_b64.
func (s *Service) Encrypt(plaintext string) (string, error) {
	keyBytes, ok := s.keys[s.activeKeyID]
	if !ok {
		return "", ErrMissingActiveKey
	}
	block, err := aes.NewCipher(keyBytes)
	if err != nil {
		return "", fmt.Errorf("%w: %v", ErrEncryptionFailed, err)
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return "", fmt.Errorf("%w: %v", ErrEncryptionFailed, err)
	}
	nonce := make([]byte, gcm.NonceSize())
	if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
		return "", fmt.Errorf("%w: %v", ErrEncryptionFailed, err)
	}
	ciphertext := gcm.Seal(nil, nonce, []byte(plaintext), []byte(s.activeKeyID))
	b64Nonce := base64.StdEncoding.EncodeToString(nonce)
	b64Ciphertext := base64.StdEncoding.EncodeToString(ciphertext)
	return fmt.Sprintf("%s:%s:%s", s.activeKeyID, b64Nonce, b64Ciphertext), nil
}

// Decrypt decrypts payload formatted as version:nonce_b64:ciphertext_b64.
func (s *Service) Decrypt(payload string) (string, error) {
	parts := strings.Split(payload, ":")
	if len(parts) != 3 {
		return "", ErrInvalidFormat
	}
	version := parts[0]
	keyBytes, ok := s.keys[version]
	if !ok {
		return "", fmt.Errorf("%w: %s", ErrKeyNotFound, version)
	}
	nonce, err := base64.StdEncoding.DecodeString(parts[1])
	if err != nil {
		return "", ErrInvalidFormat
	}
	ciphertext, err := base64.StdEncoding.DecodeString(parts[2])
	if err != nil {
		return "", ErrInvalidFormat
	}
	block, err := aes.NewCipher(keyBytes)
	if err != nil {
		return "", fmt.Errorf("%w: %v", ErrDecryptionFailed, err)
	}
	gcm, err := cipher.NewGCM(block)
	if err != nil {
		return "", fmt.Errorf("%w: %v", ErrDecryptionFailed, err)
	}
	if len(nonce) != gcm.NonceSize() {
		return "", ErrInvalidFormat
	}
	plaintext, err := gcm.Open(nil, nonce, ciphertext, []byte(version))
	if err != nil {
		return "", fmt.Errorf("%w: %v", ErrDecryptionFailed, err)
	}
	return string(plaintext), nil
}
