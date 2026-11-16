package service

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/mail"
	"strings"
	"time"

	"github.com/vkaylee/leeattend/apiserver/internal/infra/external"
	"github.com/vkaylee/leeattend/apiserver/internal/observability"
)

// ErrEmailNotConfigured indicates that no email transport is configured.
//
//lint:ignore ST1005 preserve user-facing diagnostic text
var ErrEmailNotConfigured = errors.New("Email is not configured. Insert a 'smtp' record into system_settings with either SMTP host/port or an 'http_api' block.")

// SettingsReader reads a raw JSON value from system_settings.
type SettingsReader interface {
	Get(context.Context, string) (json.RawMessage, bool, error)
}

type HTTPAPIConfig struct {
	URL          string          `json:"url"`
	AuthHeader   string          `json:"auth_header"`
	AuthValue    string          `json:"auth_value"`
	SuccessField string          `json:"success_field"`
	SuccessValue json.RawMessage `json:"success_value"`
	BodyTemplate string          `json:"body_template"`
}

type SMTPConfig struct {
	Host     string         `json:"host"`
	Port     int            `json:"port"`
	Username string         `json:"username"`
	Password string         `json:"password"`
	From     string         `json:"from"`
	BaseURL  string         `json:"base_url"`
	HTTPAPI  *HTTPAPIConfig `json:"http_api"`
}

// EmailService sends operational email using the current system_settings.smtp JSON document.
type EmailService struct {
	Settings SettingsReader
	Client   *http.Client
	Now      func() time.Time
}

func (s EmailService) loadConfig(ctx context.Context) (SMTPConfig, error) {
	if s.Settings == nil {
		return SMTPConfig{}, ErrEmailNotConfigured
	}
	raw, ok, err := s.Settings.Get(ctx, "smtp")
	if err != nil {
		return SMTPConfig{}, err
	}
	if !ok || len(raw) == 0 {
		return SMTPConfig{}, ErrEmailNotConfigured
	}
	var cfg SMTPConfig
	if err := json.Unmarshal(raw, &cfg); err != nil {
		return SMTPConfig{}, ErrEmailNotConfigured
	}
	return cfg, nil
}

// SendTestEmail sends a diagnostic email without rate limiting.
func (s EmailService) SendTestEmail(ctx context.Context, toEmail string) (string, error) {
	cfg, err := s.loadConfig(ctx)
	if err != nil {
		return "", err
	}
	if cfg.HTTPAPI == nil && (cfg.Host == "" || cfg.Port == 0) {
		return "", ErrEmailNotConfigured
	}

	now := time.Now
	if s.Now != nil {
		now = s.Now
	}
	baseURL := cfg.BaseURL
	if baseURL == "" {
		baseURL = "http://localhost:8080"
	}
	subject := "LeeAttend — Email Test"
	body := fmt.Sprintf(
		"Hello,\n\nThis is a test email from LeeAttend to confirm your email configuration is working correctly.\n\nSent at: %s\nApp URL: %s\n\nIf you received this, your email setup is ready.\n\nRegards,\nLeeAttend Team",
		now().UTC().Format("2006-01-02 15:04:05 UTC"), baseURL,
	)

	if cfg.HTTPAPI != nil {
		if err := s.sendViaHTTP(ctx, cfg, toEmail, subject, body); err != nil {
			return "", err
		}
		return fmt.Sprintf("Test email sent via HTTP API. Check %s for the message.", toEmail), nil
	}

	from := cfg.From
	if from == "" {
		from = "LeeAttend <noreply@leeattend.com>"
	}
	fromAddress, err := mail.ParseAddress(from)
	if err != nil {
		//lint:ignore ST1005 preserve user-facing SMTP diagnostic text
		return "", fmt.Errorf("Failed to parse SMTP_FROM: %v", err)
	}
	toAddress, err := mail.ParseAddress(toEmail)
	if err != nil {
		//lint:ignore ST1005 preserve user-facing SMTP diagnostic text
		return "", fmt.Errorf("Failed to parse recipient: %v", err)
	}
	mailer := external.SMTPMailer{
		Host:     cfg.Host,
		Port:     fmt.Sprintf("%d", cfg.Port),
		Username: cfg.Username,
		Password: cfg.Password,
		From:     *fromAddress,
	}
	if err := mailer.Send(ctx, *toAddress, subject, body); err != nil {
		//lint:ignore ST1005 preserve user-facing SMTP diagnostic text
		return "", fmt.Errorf("Failed to send test email via SMTP: %v\n\nPlease verify your SMTP settings in system_settings (key='smtp').", err)
	}
	return fmt.Sprintf("Test email sent successfully. Check %s for the message.", toEmail), nil
}

func (s EmailService) sendViaHTTP(ctx context.Context, cfg SMTPConfig, to, subject, body string) error {
	api := cfg.HTTPAPI
	if api == nil || strings.TrimSpace(api.URL) == "" {
		return ErrEmailNotConfigured
	}
	fromName, fromEmail := parseFromAddress(cfg.From)
	payload := api.BodyTemplate
	replacements := map[string]string{
		"{{from_name}}":  fromName,
		"{{from_email}}": fromEmail,
		"{{to}}":         to,
		"{{subject}}":    subject,
		"{{body}}":       body,
	}
	for placeholder, value := range replacements {
		encoded, _ := json.Marshal(value)
		escaped := string(encoded)
		if len(escaped) >= 2 {
			escaped = escaped[1 : len(escaped)-1]
		}
		payload = strings.ReplaceAll(payload, placeholder, escaped)
	}
	authValue := api.AuthValue
	authValue = strings.ReplaceAll(authValue, "{{username}}", cfg.Username)
	authValue = strings.ReplaceAll(authValue, "{{password}}", cfg.Password)

	client := s.Client
	if client == nil {
		client = &http.Client{Timeout: 10 * time.Second}
	}
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, api.URL, bytes.NewBufferString(payload))
	if err != nil {
		//lint:ignore ST1005 preserve user-facing HTTP API diagnostic text
		return fmt.Errorf("Failed to send email via HTTP API: %v", err)
	}
	req.Header.Set("Content-Type", "application/json")
	if api.AuthHeader != "" && authValue != "" {
		req.Header.Set(api.AuthHeader, authValue)
	}
	resp, err := client.Do(req)
	if err != nil {
		//lint:ignore ST1005 preserve user-facing HTTP API diagnostic text
		return fmt.Errorf("Failed to send email via HTTP API: %v", err)
	}
	defer func() {
		_, _ = io.Copy(io.Discard, resp.Body)
		_ = resp.Body.Close()
	}()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		//lint:ignore ST1005 preserve user-facing HTTP API diagnostic text
		return fmt.Errorf("Email API returned HTTP %d", resp.StatusCode)
	}
	if api.SuccessField != "" && len(api.SuccessValue) > 0 {
		respBody, err := io.ReadAll(io.LimitReader(resp.Body, 1<<20))
		if err != nil {
			//lint:ignore ST1005 preserve user-facing HTTP API diagnostic text
			return fmt.Errorf("Failed to parse email API response: %v", err)
		}
		var decoded map[string]json.RawMessage
		if err := json.Unmarshal(respBody, &decoded); err != nil {
			//lint:ignore ST1005 preserve user-facing HTTP API diagnostic text
			return fmt.Errorf("Failed to parse email API response: %v", err)
		}
		actual, ok := decoded[api.SuccessField]
		if !ok || string(bytes.TrimSpace(actual)) != string(bytes.TrimSpace(api.SuccessValue)) {
			//lint:ignore ST1005 preserve user-facing HTTP API diagnostic text
			return fmt.Errorf("Email API indicated failure: field '%s' was %s, expected %s", api.SuccessField, string(actual), string(api.SuccessValue))
		}
	}
	return nil
}

func parseFromAddress(from string) (string, string) {
	address, err := mail.ParseAddress(from)
	if err != nil {
		return "", from
	}
	return address.Name, address.Address
}

var _ = observability.MaskEmail
