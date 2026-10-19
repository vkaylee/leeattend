package external

import (
	"context"
	"crypto/hmac"
	"crypto/sha256"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"sort"
	"strings"
	"time"
)

type S3Client struct {
	Endpoint   *url.URL
	Bucket     string
	AccessKey  string
	SecretKey  string
	Region     string
	PathStyle  bool
	HTTPClient *http.Client
}

func NewS3Client(endpoint, bucket, accessKey, secretKey, region string, pathStyle bool, client *http.Client) (*S3Client, error) {
	parsed, err := url.Parse(endpoint)
	if err != nil || parsed.Scheme != "http" && parsed.Scheme != "https" || parsed.Host == "" {
		return nil, errors.New("invalid S3 endpoint")
	}
	if bucket == "" || accessKey == "" || secretKey == "" {
		return nil, errors.New("S3 bucket and credentials are required")
	}
	if region == "" {
		region = "us-east-1"
	}
	if client == nil {
		client = &http.Client{Timeout: 30 * time.Second}
	}
	return &S3Client{Endpoint: parsed, Bucket: bucket, AccessKey: accessKey, SecretKey: secretKey, Region: region, PathStyle: pathStyle, HTTPClient: client}, nil
}

func (c *S3Client) objectURL(key string) (*url.URL, error) {
	if c == nil || c.Endpoint == nil {
		return nil, errors.New("S3 client is required")
	}
	if key == "" || strings.Contains(key, "..") {
		return nil, errors.New("invalid object key")
	}
	copyURL := *c.Endpoint
	path := strings.TrimSuffix(copyURL.Path, "/")
	if c.PathStyle {
		path += "/" + url.PathEscape(c.Bucket) + "/" + escapeKey(key)
	} else {
		copyURL.Host = c.Bucket + "." + copyURL.Host
		path += "/" + escapeKey(key)
	}
	copyURL.Path = path
	copyURL.RawPath = path
	return &copyURL, nil
}

func (c *S3Client) Put(ctx context.Context, key, contentType string, body io.Reader) error {
	return c.request(ctx, http.MethodPut, key, contentType, body)
}
func (c *S3Client) Delete(ctx context.Context, key string) error {
	return c.request(ctx, http.MethodDelete, key, "", nil)
}
func (c *S3Client) Get(ctx context.Context, key string) (*http.Response, error) {
	return c.do(ctx, http.MethodGet, key, "", nil, time.Time{})
}
func (c *S3Client) request(ctx context.Context, method, key, contentType string, body io.Reader) error {
	response, err := c.do(ctx, method, key, contentType, body, time.Time{})
	if response != nil {
		response.Body.Close()
	}
	return err
}

func (c *S3Client) Presign(ctx context.Context, method, key, contentType string, ttl time.Duration) (string, error) {
	if ttl <= 0 || ttl > 7*24*time.Hour {
		return "", errors.New("presign TTL must be between 1 second and 7 days")
	}
	if err := ctx.Err(); err != nil {
		return "", err
	}
	object, err := c.objectURL(key)
	if err != nil {
		return "", err
	}
	now := time.Now().UTC()
	credential := c.AccessKey + "/" + now.Format("20060102") + "/" + c.Region + "/s3/aws4_request"
	query := object.Query()
	query.Set("X-Amz-Algorithm", "AWS4-HMAC-SHA256")
	query.Set("X-Amz-Credential", credential)
	query.Set("X-Amz-Date", now.Format("20060102T150405Z"))
	query.Set("X-Amz-Expires", fmt.Sprintf("%d", int(ttl.Seconds())))
	signedHeaders := "host"
	if contentType != "" {
		signedHeaders = "content-type;host"
		query.Set("content-type", contentType)
	}
	query.Set("X-Amz-SignedHeaders", signedHeaders)
	object.RawQuery = query.Encode()
	canonicalQuery := canonicalQuery(object.Query())
	canonicalHeaders := "host:" + strings.ToLower(object.Host) + "\n"
	payloadHash := sha256Hex([]byte("UNSIGNED-PAYLOAD"))
	canonicalRequest := method + "\n" + canonicalPath(object) + "\n" + canonicalQuery + "\n" + canonicalHeaders + "\n" + signedHeaders + "\n" + payloadHash
	scope := now.Format("20060102") + "/" + c.Region + "/s3/aws4_request"
	stringToSign := "AWS4-HMAC-SHA256\n" + now.Format("20060102T150405Z") + "\n" + scope + "\n" + sha256Hex([]byte(canonicalRequest))
	signature := hexHMAC(signingKey(c.SecretKey, now.Format("20060102"), c.Region, "s3"), stringToSign)
	query.Set("X-Amz-Signature", signature)
	object.RawQuery = query.Encode()
	return object.String(), nil
}

func (c *S3Client) do(ctx context.Context, method, key, contentType string, body io.Reader, _ time.Time) (*http.Response, error) {
	rawURL, err := c.Presign(ctx, method, key, contentType, 15*time.Minute)
	if err != nil {
		return nil, err
	}
	request, err := http.NewRequestWithContext(ctx, method, rawURL, body)
	if err != nil {
		return nil, err
	}
	if contentType != "" {
		request.Header.Set("Content-Type", contentType)
	}
	response, err := c.HTTPClient.Do(request)
	if err != nil {
		return nil, fmt.Errorf("S3 request: %w", err)
	}
	if response.StatusCode >= 300 {
		response.Body.Close()
		return nil, fmt.Errorf("%w: S3 status %d", ErrUnavailable, response.StatusCode)
	}
	return response, nil
}

func escapeKey(key string) string {
	parts := strings.Split(key, "/")
	for index := range parts {
		parts[index] = url.PathEscape(parts[index])
	}
	return strings.Join(parts, "/")
}
func canonicalPath(value *url.URL) string {
	if value.EscapedPath() == "" {
		return "/"
	}
	return value.EscapedPath()
}
func canonicalQuery(values url.Values) string {
	keys := make([]string, 0, len(values))
	for key := range values {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	pairs := make([]string, 0, len(keys))
	for _, key := range keys {
		vals := append([]string(nil), values[key]...)
		sort.Strings(vals)
		for _, value := range vals {
			pairs = append(pairs, url.QueryEscape(key)+"="+url.QueryEscape(value))
		}
	}
	return strings.Join(pairs, "&")
}
func sha256Hex(data []byte) string { sum := sha256.Sum256(data); return fmt.Sprintf("%x", sum[:]) }
func hmacBytes(key []byte, value string) []byte {
	mac := hmac.New(sha256.New, key)
	_, _ = mac.Write([]byte(value))
	return mac.Sum(nil)
}
func hexHMAC(key []byte, value string) string { return fmt.Sprintf("%x", hmacBytes(key, value)) }
func signingKey(secret, date, region, service string) []byte {
	dateKey := hmacBytes([]byte("AWS4"+secret), date)
	regionKey := hmacBytes(dateKey, region)
	serviceKey := hmacBytes(regionKey, service)
	return hmacBytes(serviceKey, "aws4_request")
}
