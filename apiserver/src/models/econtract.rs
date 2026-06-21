use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignAnchor {
    document_id: Uuid,
    anchor_type: String, // "coordinates" | "text_pattern"
    anchor_text: Option<String>,
    page_number: Option<i32>,
    x: Option<f64>,
    y: Option<f64>,
    width: f64,
    height: f64,
}

impl SignAnchor {
    pub fn new(
        document_id: Uuid,
        anchor_type: String,
        anchor_text: Option<String>,
        page_number: Option<i32>,
        x: Option<f64>,
        y: Option<f64>,
        width: f64,
        height: f64,
    ) -> Self {
        Self {
            document_id,
            anchor_type,
            anchor_text,
            page_number,
            x,
            y,
            width,
            height,
        }
    }

    pub fn document_id(&self) -> Uuid {
        self.document_id
    }

    pub fn anchor_type(&self) -> &str {
        &self.anchor_type
    }

    pub fn anchor_text(&self) -> Option<&str> {
        self.anchor_text.as_deref()
    }

    pub fn page_number(&self) -> Option<i32> {
        self.page_number
    }

    pub fn x(&self) -> Option<f64> {
        self.x
    }

    pub fn y(&self) -> Option<f64> {
        self.y
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Signer {
    id: String,
    name: String,
    email: String,
    phone: Option<String>,
    role: String,
    routing_order: i32,
    auth_type: String,
    national_id: Option<String>,
    visual_anchors: Vec<SignAnchor>,
}

impl Signer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        name: String,
        email: String,
        phone: Option<String>,
        role: String,
        routing_order: i32,
        auth_type: String,
        national_id: Option<String>,
        visual_anchors: Vec<SignAnchor>,
    ) -> Self {
        Self {
            id,
            name,
            email,
            phone,
            role,
            routing_order,
            auth_type,
            national_id,
            visual_anchors,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn phone(&self) -> Option<&str> {
        self.phone.as_deref()
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn routing_order(&self) -> i32 {
        self.routing_order
    }

    pub fn auth_type(&self) -> &str {
        &self.auth_type
    }

    pub fn national_id(&self) -> Option<&str> {
        self.national_id.as_deref()
    }

    pub fn visual_anchors(&self) -> &[SignAnchor] {
        &self.visual_anchors
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSource {
    id: Uuid,
    file_name: String,
    content: Vec<u8>,
}

impl DocumentSource {
    pub fn new(id: Uuid, file_name: String, content: Vec<u8>) -> Self {
        Self {
            id,
            file_name,
            content,
        }
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn content(&self) -> &[u8] {
        &self.content
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestEnvelope {
    title: String,
    documents: Vec<DocumentSource>,
    signers: Vec<Signer>,
    redirect_url: String,
}

impl RequestEnvelope {
    pub fn new(
        title: String,
        documents: Vec<DocumentSource>,
        signers: Vec<Signer>,
        redirect_url: String,
    ) -> Self {
        Self {
            title,
            documents,
            signers,
            redirect_url,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn documents(&self) -> &[DocumentSource] {
        &self.documents
    }

    pub fn signers(&self) -> &[Signer] {
        &self.signers
    }

    pub fn redirect_url(&self) -> &str {
        &self.redirect_url
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct WebhookEvent {
    envelope_id: String,
    status: String,
}

impl WebhookEvent {
    pub fn new(envelope_id: String, status: String) -> Self {
        Self {
            envelope_id,
            status,
        }
    }

    pub fn envelope_id(&self) -> &str {
        &self.envelope_id
    }

    pub fn status(&self) -> &str {
        &self.status
    }
}
