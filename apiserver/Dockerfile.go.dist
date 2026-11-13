# Multi-stage production build for Go API Server (P8-01)
# Constraints: CGO_ENABLED=0, nonroot UID 65532, distroless runtime, static binary

ARG GO_VERSION=1.26.11
FROM golang:${GO_VERSION}-bookworm AS builder

ENV CGO_ENABLED=0 \
    GOOS=linux \
    GOARCH=amd64 \
    GOPROXY=https://proxy.golang.org,direct

WORKDIR /build

# Cache dependency layer
COPY go.mod go.sum ./
RUN go mod download

# Copy source tree and compile static binary
COPY . .
RUN go build -trimpath -ldflags="-s -w -X main.Version=0.1.0" -o /usr/local/bin/apiserver .

# Prepare runtime storage directory with non-root ownership
RUN mkdir -p /tmp/storage && chown -R 65532:65532 /tmp/storage

# Stage 2: Hermetic runtime image
FROM gcr.io/distroless/static-debian12 AS runtime

WORKDIR /app

# Copy binary and storage mount
COPY --from=builder /usr/local/bin/apiserver /usr/local/bin/apiserver
COPY --chown=65532:65532 --from=builder /tmp/storage /app/storage

# Environment defaults conforming to deployment-contract.md
ENV STORAGE_DIR=/app/storage \
    APP_PORT=3001 \
    APP_HOST=0.0.0.0 \
    APP_ENV=production

# Distroless non-root user (65532:65532)
USER 65532:65532

VOLUME ["/app/storage"]
EXPOSE 3001 3000

ENTRYPOINT ["/usr/local/bin/apiserver"]
