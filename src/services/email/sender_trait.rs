//! `EmailSender` trait — 이메일 전송을 추상화하여 테스트에서 mock 구현체를 주입할 수 있게 한다.
//!
//! # 예시
//!
//! ## 실제 OCI 전송
//! ```no_run
//! use oci_api::{Oci, email::EmailSender};
//! use oci_api::email::{EmailDelivery, Email, EmailAddress, Recipients};
//! use std::sync::Arc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let oci = Oci::from_env()?;
//! let sender: Arc<dyn EmailSender> = Arc::new(EmailDelivery::new(oci).await?);
//!
//! let email = Email::builder()
//!     .sender(EmailAddress::new("noreply@example.com"))
//!     .recipients(Recipients::to(vec![EmailAddress::new("user@example.com")]))
//!     .subject("Hello")
//!     .body_text("World")
//!     .build()?;
//!
//! let response = sender.send(email).await?;
//! println!("Sent: {}", response.message_id);
//! # Ok(())
//! # }
//! ```
//!
//! ## Mock 구현 (테스트용)
//! ```
//! use oci_api::email::{EmailSender, Email, SubmitEmailResponse};
//! use oci_api::error::Result;
//! use async_trait::async_trait;
//! use std::sync::{Arc, Mutex};
//!
//! struct MockEmailSender {
//!     sent: Arc<Mutex<Vec<Email>>>,
//! }
//!
//! #[async_trait]
//! impl EmailSender for MockEmailSender {
//!     async fn send(&self, email: Email) -> Result<SubmitEmailResponse> {
//!         self.sent.lock().unwrap().push(email);
//!         Ok(SubmitEmailResponse {
//!             message_id: "mock-id".to_owned(),
//!             envelope_id: "mock-env".to_owned(),
//!             suppressed_recipients: None,
//!         })
//!     }
//! }
//! ```

use async_trait::async_trait;

use crate::error::Result;
use super::models::{Email, SubmitEmailResponse};

/// 이메일 전송 추상화 trait.
///
/// `EmailDelivery`(실제 OCI API) 외에 mock/stub/dry-run 구현체를
/// 주입하여 테스트할 수 있도록 한다.
#[async_trait]
pub trait EmailSender: Send + Sync {
    /// 이메일을 전송한다.
    ///
    /// 성공 시 OCI의 `message_id`와 `envelope_id`를 포함한 응답을 반환한다.
    async fn send(&self, email: Email) -> Result<SubmitEmailResponse>;
}
