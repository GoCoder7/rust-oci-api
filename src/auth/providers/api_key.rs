use async_trait::async_trait;

use crate::auth::providers::{OciAuthProvider, SignRequest, SignedHeaders};
use crate::client::signer::OciSigner;
use crate::error::Result;

#[derive(Clone)]
pub struct ApiKeyAuthProvider {
    signer: OciSigner,
}

impl ApiKeyAuthProvider {
    pub fn new(signer: OciSigner) -> Self {
        Self { signer }
    }

    pub fn signer(&self) -> &OciSigner {
        &self.signer
    }
}

#[async_trait]
impl OciAuthProvider for ApiKeyAuthProvider {
    async fn sign(&self, request: &SignRequest<'_>) -> Result<SignedHeaders> {
        let signed = self.signer.sign_request_headers(
            request.method,
            request.path,
            request.host,
            request.body,
            request.content_type,
            None,
        )?;

        Ok(SignedHeaders {
            date: signed.date,
            authorization: signed.authorization,
            content_type: signed.content_type,
            content_length: signed.content_length,
            x_content_sha256: signed.x_content_sha256,
            extra_headers: Vec::new(),
        })
    }
}
