package external

import (
	"context"
	"crypto/tls"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/mail"
	"net/smtp"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"time"
)

var ErrUnavailable = errors.New("external dependency unavailable")

type HTTPClient struct{ Client *http.Client }

func NewHTTPClient(timeout time.Duration) *HTTPClient {
	if timeout <= 0 {
		timeout = 10 * time.Second
	}
	return &HTTPClient{Client: &http.Client{Timeout: timeout}}
}

func (c *HTTPClient) Do(ctx context.Context, method, rawURL string, body io.Reader) (*http.Response, error) {
	if c == nil || c.Client == nil {
		return nil, errors.New("http client is required")
	}
	u, err := url.Parse(rawURL)
	if err != nil || u.Scheme != "https" && u.Scheme != "http" {
		return nil, errors.New("invalid external URL")
	}
	req, err := http.NewRequestWithContext(ctx, method, u.String(), body)
	if err != nil {
		return nil, fmt.Errorf("create external request: %w", err)
	}
	resp, err := c.Client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("external request: %w", err)
	}
	if resp.StatusCode >= 500 {
		resp.Body.Close()
		return nil, fmt.Errorf("%w: status %d", ErrUnavailable, resp.StatusCode)
	}
	return resp, nil
}

type FileStore struct{ Root string }

func (s FileStore) path(name string) (string, error) {
	if s.Root == "" || name == "" {
		return "", errors.New("file store root and name are required")
	}
	clean := filepath.Clean(name)
	if clean == "." || filepath.IsAbs(clean) || strings.HasPrefix(clean, ".."+string(filepath.Separator)) || clean == ".." {
		return "", errors.New("invalid file name")
	}
	return filepath.Join(s.Root, clean), nil
}
func (s FileStore) Put(ctx context.Context, name string, content io.Reader, mode os.FileMode) error {
	if err := ctx.Err(); err != nil {
		return err
	}
	path, err := s.path(name)
	if err != nil {
		return err
	}
	if err := os.MkdirAll(filepath.Dir(path), 0o750); err != nil {
		return fmt.Errorf("create file directory: %w", err)
	}
	file, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, mode&0o770)
	if err != nil {
		return fmt.Errorf("open file: %w", err)
	}
	defer file.Close()
	if _, err := io.Copy(file, content); err != nil {
		return fmt.Errorf("write file: %w", err)
	}
	return nil
}
func (s FileStore) Open(name string) (*os.File, error) {
	path, err := s.path(name)
	if err != nil {
		return nil, err
	}
	return os.Open(path)
}

// ClamAVClient speaks the INSTREAM protocol and returns malware verdicts.
type ClamAVClient struct {
	Address string
	Timeout time.Duration
}

func (c ClamAVClient) Scan(ctx context.Context, content io.Reader) error {
	if c.Address == "" {
		return errors.New("clamav address is required")
	}
	dialer := net.Dialer{Timeout: c.Timeout}
	conn, err := dialer.DialContext(ctx, "tcp", c.Address)
	if err != nil {
		return fmt.Errorf("%w: clamav dial: %v", ErrUnavailable, err)
	}
	defer conn.Close()
	_ = conn.SetDeadline(time.Now().Add(c.Timeout))
	if _, err := conn.Write([]byte("zINSTREAM\000")); err != nil {
		return fmt.Errorf("clamav start: %w", err)
	}
	buf := make([]byte, 32*1024)
	for {
		n, readErr := content.Read(buf)
		if n > 0 {
			header := []byte{byte(n >> 24), byte(n >> 16), byte(n >> 8), byte(n)}
			if _, err := conn.Write(append(header, buf[:n]...)); err != nil {
				return fmt.Errorf("clamav stream: %w", err)
			}
		}
		if errors.Is(readErr, io.EOF) {
			break
		}
		if readErr != nil {
			return fmt.Errorf("clamav read: %w", readErr)
		}
	}
	if _, err := conn.Write([]byte{0, 0, 0, 0}); err != nil {
		return fmt.Errorf("clamav finish: %w", err)
	}
	response, err := io.ReadAll(io.LimitReader(conn, 4096))
	if err != nil {
		return fmt.Errorf("clamav response: %w", err)
	}
	if strings.Contains(string(response), "FOUND") {
		return errors.New("malware detected")
	}
	if !strings.Contains(string(response), "OK") {
		return fmt.Errorf("%w: clamav response", ErrUnavailable)
	}
	return nil
}

// SMTPMailer sends a single message through an authenticated SMTP server.
type SMTPMailer struct {
	Host      string
	Port      string
	Username  string
	Password  string
	From      mail.Address
	TLSConfig *tls.Config
}

func (m SMTPMailer) Send(ctx context.Context, to mail.Address, subject, body string) error {
	if m.Host == "" || m.Port == "" || m.From.Address == "" || to.Address == "" {
		return errors.New("smtp configuration and recipients are required")
	}
	if err := ctx.Err(); err != nil {
		return err
	}
	header := "From: " + m.From.String() + "\r\nTo: " + to.String() + "\r\nSubject: " + subject + "\r\n\r\n"
	address := net.JoinHostPort(m.Host, m.Port)
	auth := smtp.PlainAuth("", m.Username, m.Password, m.Host)
	return smtp.SendMail(address, auth, m.From.Address, []string{to.Address}, []byte(header+body))
}
