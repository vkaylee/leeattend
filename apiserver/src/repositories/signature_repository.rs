use async_trait::async_trait;
use uuid::Uuid;

use crate::diesel_pool::TenantAuthToken;
use crate::errors::AppResult;
use crate::models::signature::SignatureRequest;

#[async_trait]
pub trait SignatureRepository: Send + Sync {
    /// Lấy danh sách yêu cầu chữ ký theo Document ID
    async fn get_requests_by_document(
        &self,
        auth_token: &TenantAuthToken,
        document_id: Uuid,
    ) -> AppResult<Vec<SignatureRequest>>;

    /// Lấy danh sách yêu cầu chữ ký của một nhân viên (Signer)
    async fn get_requests_by_signer(
        &self,
        auth_token: &TenantAuthToken,
        signer_id: Uuid,
    ) -> AppResult<Vec<SignatureRequest>>;

    /// Tìm một yêu cầu chữ ký theo ID
    async fn get_signature_request(
        &self,
        auth_token: &TenantAuthToken,
        request_id: Uuid,
    ) -> AppResult<Option<SignatureRequest>>;

    /// Tạo yêu cầu chữ ký mới
    async fn create_signature_request(
        &self,
        auth_token: &TenantAuthToken,
        request: &SignatureRequest,
    ) -> AppResult<()>;

    /// Cập nhật trạng thái yêu cầu chữ ký (khi người dùng ký)
    async fn update_signature_status(
        &self,
        auth_token: &TenantAuthToken,
        request_id: Uuid,
        status: &str,
        signed_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> AppResult<SignatureRequest>;
}
