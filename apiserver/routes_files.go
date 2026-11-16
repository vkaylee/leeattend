package main

import (
	"github.com/go-chi/chi/v5"
	"github.com/vkaylee/leeattend/apiserver/internal/api"
)

func registerFileRoutes(router chi.Router, files *api.FileHandler) {
	registerRoute(router, "POST /api/v1/files/upload", files.UploadFile)
	registerRoute(router, "POST /api/v1/files/presigned", files.CreateUploadURL)
	registerRoute(router, "GET /api/v1/files/{id}", files.GetMetadata)
	registerRoute(router, "PUT /api/v1/files/{id}", files.UpdateFile)
	registerRoute(router, "DELETE /api/v1/files/{id}", files.DeleteFile)
	registerRoute(router, "GET /api/v1/files/{id}/download", files.DownloadFile)
	registerRoute(router, "GET /api/v1/files/{id}/url", files.GetURL)
	registerRoute(router, "GET /api/v1/files/{id}/versions", files.ListVersions)
	registerRoute(router, "GET /api/v1/files/by-owner/{owner_type}/{owner_id}", files.ListByOwner)
	registerRoute(router, "POST /api/v1/documents/{document_id}/signature-requests", files.CreateSignatureRequest)
	registerRoute(router, "GET /api/v1/ess/signatures/pending", files.ListPendingSignatures)
	registerRoute(router, "POST /api/v1/ess/signatures/{request_id}/sign", files.SignDocument)
}

func registerDocumentRoutes(router chi.Router, handler *api.DocumentHandler) {
	registerRoute(router, "GET /api/v1/documents/folders", handler.ListFolders)
	registerRoute(router, "POST /api/v1/documents/folders", handler.CreateFolder)
	registerRoute(router, "GET /api/v1/documents/folders/{id}", handler.GetFolder)
	registerRoute(router, "PUT /api/v1/documents/folders/{id}", handler.RenameFolder)
	registerRoute(router, "DELETE /api/v1/documents/folders/{id}", handler.DeleteFolder)
	registerRoute(router, "POST /api/v1/documents/distributions", handler.Distribute)
	registerRoute(router, "POST /api/v1/documents/templates/preview", handler.PreviewTemplate)
	registerRoute(router, "POST /api/v1/documents/templates/{id}/generate", handler.GenerateTemplate)
}
