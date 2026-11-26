# LeeAttend Go test container (P0-06 build gates)
#
# Pinned toolchain + pinned gate binaries. Every gate version is an explicit
# build arg so a version change is a reviewable diff, never a floating "latest".
ARG GO_VERSION=1.26.11
FROM golang:${GO_VERSION}-bookworm

ARG STATICCHECK_VERSION=v0.6.1
ARG GOVULNCHECK_VERSION=v1.1.4
ARG GOLANGCI_LINT_VERSION=v2.5.0
ARG SQLC_VERSION=v1.29.0
ARG OAPI_CODEGEN_VERSION=v2.4.1

ENV GOTOOLCHAIN=local \
    GOFLAGS="-mod=mod -buildvcs=false" \
    CGO_ENABLED=1

RUN apt-get update \
    && apt-get install -y --no-install-recommends python3-yaml postgresql-client \
    && rm -rf /var/lib/apt/lists/* \
    && git config --global --add safe.directory '*' \
    && go version \
    && go install honnef.co/go/tools/cmd/staticcheck@${STATICCHECK_VERSION} \
    && go install golang.org/x/vuln/cmd/govulncheck@${GOVULNCHECK_VERSION} \
    && go install github.com/golangci/golangci-lint/v2/cmd/golangci-lint@${GOLANGCI_LINT_VERSION} \
    && go install github.com/sqlc-dev/sqlc/cmd/sqlc@${SQLC_VERSION} \
    && go install github.com/oapi-codegen/oapi-codegen/v2/cmd/oapi-codegen@${OAPI_CODEGEN_VERSION} \
    && rm -rf /go/pkg/mod/cache/download

# Fail the image build if any pinned gate binary is missing or unusable.
RUN staticcheck -version \
    && govulncheck -version \
    && golangci-lint version \
    && sqlc version \
    && oapi-codegen -version

WORKDIR /workspace
