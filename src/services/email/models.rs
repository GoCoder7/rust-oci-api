//! Email Delivery API data models

use serde::{Deserialize, Serialize};

/// Email Configuration response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfiguration {
    /// Compartment OCID
    #[serde(rename = "compartmentId")]
    pub compartment_id: String,

    /// HTTP Submit endpoint
    #[serde(rename = "httpSubmitEndpoint")]
    pub http_submit_endpoint: String,

    /// SMTP Submit endpoint
    #[serde(rename = "smtpSubmitEndpoint")]
    pub smtp_submit_endpoint: String,

    /// Email Delivery Config ID (optional, can be null)
    #[serde(rename = "emailDeliveryConfigId")]
    pub email_delivery_config_id: Option<String>,
}

/// Email message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    /// Message ID (optional)
    #[serde(rename = "messageId", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,

    /// Sender
    pub sender: Sender,

    /// Recipients
    pub recipients: Recipients,

    /// Subject
    pub subject: String,

    /// Body (HTML)
    #[serde(rename = "bodyHtml", skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,

    /// Body (Plain Text)
    #[serde(rename = "bodyText", skip_serializing_if = "Option::is_none")]
    pub body_text: Option<String>,

    /// Reply-To address (optional)
    #[serde(rename = "replyTo", skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Vec<EmailAddress>>,

    /// Custom headers (optional)
    #[serde(rename = "headerFields", skip_serializing_if = "Option::is_none")]
    pub headers: Option<std::collections::HashMap<String, String>>,
}

/// Sender information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sender {
    /// Sender email address
    #[serde(rename = "senderAddress")]
    pub sender_address: EmailAddress,

    /// Compartment OCID
    #[serde(rename = "compartmentId")]
    pub compartment_id: String,
}

impl Sender {
    /// Create new sender (compartment_id will be set by EmailClient)
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            sender_address: EmailAddress::new(email),
            compartment_id: String::new(), // Will be set by EmailClient
        }
    }

    /// Create sender with name (compartment_id will be set by EmailClient)
    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            sender_address: EmailAddress::with_name(email, name),
            compartment_id: String::new(), // Will be set by EmailClient
        }
    }

    /// Internal method to set compartment_id (used by EmailClient)
    pub(crate) fn set_compartment_id(&mut self, compartment_id: impl Into<String>) {
        self.compartment_id = compartment_id.into();
    }
}

/// Email address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    /// Email address
    pub email: String,

    /// Name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// Implement PartialEq based on email only (ignore name for equality)
impl PartialEq for EmailAddress {
    fn eq(&self, other: &Self) -> bool {
        self.email == other.email
    }
}

impl Eq for EmailAddress {}

// Implement Hash based on email only (ignore name for hashing)
impl std::hash::Hash for EmailAddress {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.email.hash(state);
    }
}

/// Recipients list
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipients {
    /// To recipients
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<Vec<EmailAddress>>,

    /// CC recipients
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc: Option<Vec<EmailAddress>>,

    /// BCC recipients
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<EmailAddress>>,
}

/// Email submission response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitEmailResponse {
    /// Submitted email's message ID
    #[serde(rename = "messageId")]
    pub message_id: String,

    /// Envelope ID (not envelopeMessageId as in docs)
    #[serde(rename = "envelopeId")]
    pub envelope_id: String,

    /// Suppressed recipients (optional)
    #[serde(
        rename = "suppressedRecipients",
        skip_serializing_if = "Option::is_none"
    )]
    pub suppressed_recipients: Option<Vec<EmailAddress>>,
}

/// Sender summary from list_senders API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SenderSummary {
    /// Sender OCID
    pub id: String,

    /// Email address
    #[serde(rename = "emailAddress")]
    pub email_address: String,

    /// Lifecycle state
    #[serde(rename = "lifecycleState")]
    pub lifecycle_state: SenderLifecycleState,

    /// Time created
    #[serde(rename = "timeCreated")]
    pub time_created: String,

    /// Is SPF (Sender Policy Framework) configured (optional)
    #[serde(rename = "isSpf", skip_serializing_if = "Option::is_none")]
    pub is_spf: Option<bool>,

    /// Compartment ID (optional, not always included)
    #[serde(rename = "compartmentId", skip_serializing_if = "Option::is_none")]
    pub compartment_id: Option<String>,
}

/// Sender lifecycle state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SenderLifecycleState {
    /// Creating
    Creating,
    /// Active
    Active,
    /// Needs attention
    NeedsAttention,
    /// Inactive
    Inactive,
    /// Failed
    Failed,
    /// Deleting
    Deleting,
    /// Deleted
    Deleted,
}

impl EmailAddress {
    /// Create new email address
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: None,
        }
    }

    /// Create email address with name
    pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }
}

impl Recipients {
    /// Remove duplicates from email address list
    fn deduplicate(addresses: Vec<EmailAddress>) -> Vec<EmailAddress> {
        use std::collections::HashSet;
        let mut seen = HashSet::new();
        addresses
            .into_iter()
            .filter(|addr| seen.insert(addr.clone()))
            .collect()
    }

    /// Create recipients list with To recipients (alias for `to`)
    pub fn new(addresses: Vec<EmailAddress>) -> Self {
        Self::to(addresses)
    }

    /// Create recipients list with only To recipients
    pub fn to(addresses: Vec<EmailAddress>) -> Self {
        Self {
            to: Some(Self::deduplicate(addresses)),
            cc: None,
            bcc: None,
        }
    }

    /// Create recipients list with only CC recipients
    pub fn cc(addresses: Vec<EmailAddress>) -> Self {
        Self {
            to: None,
            cc: Some(Self::deduplicate(addresses)),
            bcc: None,
        }
    }

    /// Create recipients list with only BCC recipients
    pub fn bcc(addresses: Vec<EmailAddress>) -> Self {
        Self {
            to: None,
            cc: None,
            bcc: Some(Self::deduplicate(addresses)),
        }
    }

    /// Add To recipients to existing Recipients
    pub fn add_to(mut self, mut addresses: Vec<EmailAddress>) -> Self {
        if let Some(ref mut to) = self.to {
            to.append(&mut addresses);
            *to = Self::deduplicate(to.clone());
        } else {
            self.to = Some(Self::deduplicate(addresses));
        }
        self
    }

    /// Add CC recipients to existing Recipients
    pub fn add_cc(mut self, mut addresses: Vec<EmailAddress>) -> Self {
        if let Some(ref mut cc) = self.cc {
            cc.append(&mut addresses);
            *cc = Self::deduplicate(cc.clone());
        } else {
            self.cc = Some(Self::deduplicate(addresses));
        }
        self
    }

    /// Add BCC recipients to existing Recipients
    pub fn add_bcc(mut self, mut addresses: Vec<EmailAddress>) -> Self {
        if let Some(ref mut bcc) = self.bcc {
            bcc.append(&mut addresses);
            *bcc = Self::deduplicate(bcc.clone());
        } else {
            self.bcc = Some(Self::deduplicate(addresses));
        }
        self
    }

    /// Create a new builder for Recipients
    pub fn builder() -> RecipientsBuilder {
        RecipientsBuilder::default()
    }
}

/// Builder for Recipients
#[derive(Debug, Default)]
pub struct RecipientsBuilder {
    to: Option<Vec<EmailAddress>>,
    cc: Option<Vec<EmailAddress>>,
    bcc: Option<Vec<EmailAddress>>,
}

impl RecipientsBuilder {
    /// Set To recipients
    pub fn to(mut self, addresses: Vec<EmailAddress>) -> Self {
        self.to = Some(Recipients::deduplicate(addresses));
        self
    }

    /// Set CC recipients
    pub fn cc(mut self, addresses: Vec<EmailAddress>) -> Self {
        self.cc = Some(Recipients::deduplicate(addresses));
        self
    }

    /// Set BCC recipients
    pub fn bcc(mut self, addresses: Vec<EmailAddress>) -> Self {
        self.bcc = Some(Recipients::deduplicate(addresses));
        self
    }

    /// Build Recipients
    pub fn build(self) -> Recipients {
        Recipients {
            to: self.to,
            cc: self.cc,
            bcc: self.bcc,
        }
    }
}

impl Email {
    /// Create a new builder for Email
    pub fn builder() -> EmailBuilder {
        EmailBuilder::default()
    }
}

/// Builder for Email
#[derive(Debug, Default)]
pub struct EmailBuilder {
    message_id: Option<String>,
    sender: Option<EmailAddress>,
    recipients: Option<Recipients>,
    subject: Option<String>,
    body_html: Option<String>,
    body_text: Option<String>,
    reply_to: Option<Vec<EmailAddress>>,
    headers: Option<std::collections::HashMap<String, String>>,
}

impl EmailBuilder {
    /// Set message ID
    pub fn message_id(mut self, message_id: impl Into<String>) -> Self {
        self.message_id = Some(message_id.into());
        self
    }

    /// Set sender email address
    pub fn sender(mut self, sender: EmailAddress) -> Self {
        self.sender = Some(sender);
        self
    }

    /// Set recipients
    pub fn recipients(mut self, recipients: Recipients) -> Self {
        self.recipients = Some(recipients);
        self
    }

    /// Set subject
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Set HTML body
    pub fn body_html(mut self, body_html: impl Into<String>) -> Self {
        self.body_html = Some(body_html.into());
        self
    }

    /// Set plain text body
    pub fn body_text(mut self, body_text: impl Into<String>) -> Self {
        self.body_text = Some(body_text.into());
        self
    }

    /// Set reply-to addresses
    pub fn reply_to(mut self, reply_to: Vec<EmailAddress>) -> Self {
        self.reply_to = Some(reply_to);
        self
    }

    /// Set custom headers
    pub fn headers(mut self, headers: std::collections::HashMap<String, String>) -> Self {
        self.headers = Some(headers);
        self
    }

    /// Build Email
    ///
    /// Returns an error if required fields are missing or invalid
    pub fn build(self) -> crate::error::Result<Email> {
        let sender_address = self
            .sender
            .ok_or_else(|| crate::error::OciError::ConfigError("Sender is required".to_string()))?;

        // Create Sender with empty compartment_id (will be set by send)
        let sender = Sender {
            sender_address,
            compartment_id: String::new(),
        };

        let recipients = self.recipients.ok_or_else(|| {
            crate::error::OciError::ConfigError("Recipients are required".to_string())
        })?;

        let subject = self.subject.ok_or_else(|| {
            crate::error::OciError::ConfigError("Subject is required".to_string())
        })?;

        // Validate that at least one body (HTML or text) is provided
        if self.body_html.is_none() && self.body_text.is_none() {
            return Err(crate::error::OciError::ConfigError(
                "At least one of body_html or body_text is required".to_string(),
            ));
        }

        Ok(Email {
            message_id: self.message_id,
            sender,
            recipients,
            subject,
            body_html: self.body_html,
            body_text: self.body_text,
            reply_to: self.reply_to,
            headers: self.headers,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_address_new() {
        let addr = EmailAddress::new("test@example.com");
        assert_eq!(addr.email, "test@example.com");
        assert_eq!(addr.name, None);
    }

    #[test]
    fn test_email_address_with_name() {
        let addr = EmailAddress::with_name("test@example.com", "Test User");
        assert_eq!(addr.email, "test@example.com");
        assert_eq!(addr.name, Some("Test User".to_string()));
    }

    #[test]
    fn test_recipients_to() {
        let recipients = Recipients::to(vec![
            EmailAddress::new("user1@example.com"),
            EmailAddress::new("user2@example.com"),
        ]);
        assert_eq!(recipients.to.as_ref().unwrap().len(), 2);
        assert_eq!(recipients.cc, None);
        assert_eq!(recipients.bcc, None);
    }

    #[test]
    fn test_recipients_builder() {
        let recipients = Recipients::builder()
            .to(vec![EmailAddress::new("to@example.com")])
            .cc(vec![EmailAddress::new("cc@example.com")])
            .bcc(vec![EmailAddress::new("bcc@example.com")])
            .build();

        assert_eq!(recipients.to.as_ref().unwrap().len(), 1);
        assert_eq!(recipients.cc.as_ref().unwrap().len(), 1);
        assert_eq!(recipients.bcc.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_submit_email_request_serialization() {
        let mut request = Email {
            message_id: Some("test-123".to_string()),
            sender: Sender::with_name("sender@example.com", "Sender"),
            recipients: Recipients::to(vec![EmailAddress::new("recipient@example.com")]),
            subject: "Test Subject".to_string(),
            body_html: Some("<html><body>Test</body></html>".to_string()),
            body_text: Some("Test".to_string()),
            reply_to: None,
            headers: None,
        };
        // Set compartment_id manually for test
        request.sender.set_compartment_id("ocid1.compartment.test");

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"sender\""));
        assert!(json.contains("\"recipients\""));
        assert!(json.contains("\"subject\""));
        assert!(json.contains("\"messageId\""));
    }

    #[test]
    fn test_submit_email_request_builder() {
        let mut request = Email::builder()
            .sender(EmailAddress::new("sender@example.com"))
            .recipients(
                Recipients::builder()
                    .to(vec![EmailAddress::new("recipient@example.com")])
                    .build(),
            )
            .subject("Test Subject")
            .body_text("Test body")
            .build()
            .unwrap();
        // Set compartment_id manually for test
        request.sender.set_compartment_id("ocid1.compartment.test");

        assert_eq!(request.subject, "Test Subject");
        assert_eq!(request.body_text.as_ref().unwrap(), "Test body");
        assert!(request.recipients.to.is_some());
    }

    #[test]
    fn test_submit_email_request_builder_missing_required_fields() {
        // Missing sender
        let result = Email::builder()
            .recipients(Recipients::to(vec![EmailAddress::new("to@example.com")]))
            .subject("Test")
            .build();
        assert!(result.is_err());

        // Missing recipients
        let result = Email::builder()
            .sender(EmailAddress::new("sender@example.com"))
            .subject("Test")
            .build();
        assert!(result.is_err());

        // Missing subject
        let result = Email::builder()
            .sender(EmailAddress::new("sender@example.com"))
            .recipients(Recipients::to(vec![EmailAddress::new("to@example.com")]))
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_email_configuration_deserialization() {
        let json = r#"{
            "compartmentId": "ocid1.compartment.test",
            "httpSubmitEndpoint": "https://email.ap-seoul-1.oci.oraclecloud.com",
            "smtpSubmitEndpoint": "smtp.email.ap-seoul-1.oci.oraclecloud.com"
        }"#;

        let config: EmailConfiguration = serde_json::from_str(json).unwrap();
        assert_eq!(config.compartment_id, "ocid1.compartment.test");
        assert_eq!(
            config.http_submit_endpoint,
            "https://email.ap-seoul-1.oci.oraclecloud.com"
        );
        assert_eq!(
            config.smtp_submit_endpoint,
            "smtp.email.ap-seoul-1.oci.oraclecloud.com"
        );
    }

    #[test]
    fn test_submit_email_response_deserialization() {
        let json = r#"{
            "messageId": "msg-123",
            "envelopeId": "env-456"
        }"#;

        let response: SubmitEmailResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.message_id, "msg-123");
        assert_eq!(response.envelope_id, "env-456");
    }

    #[test]
    fn test_complete_email_request_with_all_fields() {
        use std::collections::HashMap;

        let mut headers = HashMap::new();
        headers.insert("X-Test".to_string(), "test-value".to_string());

        let mut request = Email {
            message_id: Some("msg-001".to_string()),
            sender: Sender::with_name("sender@example.com", "Sender Name"),
            recipients: Recipients {
                to: Some(vec![EmailAddress::new("to@example.com")]),
                cc: Some(vec![EmailAddress::new("cc@example.com")]),
                bcc: Some(vec![EmailAddress::new("bcc@example.com")]),
            },
            subject: "Complete Test".to_string(),
            body_html: Some("<p>HTML body</p>".to_string()),
            body_text: Some("Text body".to_string()),
            reply_to: Some(vec![EmailAddress::new("replyto@example.com")]),
            headers: Some(headers),
        };
        request.sender.set_compartment_id("ocid1.compartment.test");

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: Email = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.message_id, Some("msg-001".to_string()));
        assert_eq!(deserialized.subject, "Complete Test");
        assert!(deserialized.recipients.to.is_some());
        assert!(deserialized.recipients.cc.is_some());
        assert!(deserialized.recipients.bcc.is_some());
        assert!(deserialized.reply_to.is_some());
        assert!(deserialized.headers.is_some());
    }

    #[test]
    fn test_complete_email_request_with_builder() {
        use std::collections::HashMap;

        let mut headers = HashMap::new();
        headers.insert("X-Test".to_string(), "test-value".to_string());

        let mut request = Email::builder()
            .message_id("msg-001")
            .sender(EmailAddress::with_name("sender@example.com", "Sender Name"))
            .recipients(
                Recipients::builder()
                    .to(vec![EmailAddress::new("to@example.com")])
                    .cc(vec![EmailAddress::new("cc@example.com")])
                    .bcc(vec![EmailAddress::new("bcc@example.com")])
                    .build(),
            )
            .subject("Complete Test")
            .body_html("<p>HTML body</p>")
            .body_text("Text body")
            .reply_to(vec![EmailAddress::new("replyto@example.com")])
            .headers(headers)
            .build()
            .unwrap();
        request.sender.set_compartment_id("ocid1.compartment.test");

        assert_eq!(request.message_id, Some("msg-001".to_string()));
        assert_eq!(request.subject, "Complete Test");
        assert!(request.recipients.to.is_some());
        assert!(request.recipients.cc.is_some());
        assert!(request.recipients.bcc.is_some());
        assert!(request.reply_to.is_some());
        assert!(request.headers.is_some());
    }

    #[test]
    fn test_recipients_constructors() {
        // Test new() - should be same as to()
        let recipients = Recipients::new(vec![EmailAddress::new("to@example.com")]);
        assert!(recipients.to.is_some());
        assert!(recipients.cc.is_none());
        assert!(recipients.bcc.is_none());

        // Test to()
        let recipients = Recipients::to(vec![EmailAddress::new("to@example.com")]);
        assert!(recipients.to.is_some());
        assert!(recipients.cc.is_none());
        assert!(recipients.bcc.is_none());

        // Test cc()
        let recipients = Recipients::cc(vec![EmailAddress::new("cc@example.com")]);
        assert!(recipients.to.is_none());
        assert!(recipients.cc.is_some());
        assert!(recipients.bcc.is_none());

        // Test bcc()
        let recipients = Recipients::bcc(vec![EmailAddress::new("bcc@example.com")]);
        assert!(recipients.to.is_none());
        assert!(recipients.cc.is_none());
        assert!(recipients.bcc.is_some());
    }

    #[test]
    fn test_recipients_add_methods() {
        // Start with TO recipients
        let recipients = Recipients::to(vec![EmailAddress::new("to1@example.com")])
            .add_to(vec![EmailAddress::new("to2@example.com")])
            .add_cc(vec![EmailAddress::new("cc@example.com")])
            .add_bcc(vec![EmailAddress::new("bcc@example.com")]);

        assert_eq!(recipients.to.as_ref().unwrap().len(), 2);
        assert_eq!(recipients.cc.as_ref().unwrap().len(), 1);
        assert_eq!(recipients.bcc.as_ref().unwrap().len(), 1);

        // Test adding to existing CC
        let recipients = Recipients::cc(vec![EmailAddress::new("cc1@example.com")]).add_cc(vec![
            EmailAddress::new("cc2@example.com"),
            EmailAddress::new("cc3@example.com"),
        ]);

        assert_eq!(recipients.cc.as_ref().unwrap().len(), 3);

        // Test adding when field is None
        let recipients = Recipients::to(vec![EmailAddress::new("to@example.com")])
            .add_bcc(vec![EmailAddress::new("bcc@example.com")]);

        assert!(recipients.to.is_some());
        assert!(recipients.bcc.is_some());
    }

    #[test]
    fn test_build_missing_body() {
        // Missing both body_html and body_text should fail
        let result = Email::builder()
            .sender(EmailAddress::new("sender@example.com"))
            .recipients(Recipients::to(vec![EmailAddress::new("to@example.com")]))
            .subject("Test")
            .build();

        assert!(result.is_err());
        if let Err(crate::error::OciError::ConfigError(msg)) = result {
            assert!(msg.contains("body"));
        } else {
            panic!("Expected ConfigError about body");
        }
    }

    #[test]
    fn test_build_with_only_html_body() {
        // Only body_html should be OK
        let result = Email::builder()
            .sender(EmailAddress::new("sender@example.com"))
            .recipients(Recipients::to(vec![EmailAddress::new("to@example.com")]))
            .subject("Test")
            .body_html("<p>HTML content</p>")
            .build();

        assert!(result.is_ok());
        let request = result.unwrap();
        assert!(request.body_html.is_some());
        assert!(request.body_text.is_none());
    }

    #[test]
    fn test_build_with_only_text_body() {
        // Only body_text should be OK
        let result = Email::builder()
            .sender(EmailAddress::new("sender@example.com"))
            .recipients(Recipients::to(vec![EmailAddress::new("to@example.com")]))
            .subject("Test")
            .body_text("Text content")
            .build();

        assert!(result.is_ok());
        let request = result.unwrap();
        assert!(request.body_html.is_none());
        assert!(request.body_text.is_some());
    }

    #[test]
    fn test_recipients_deduplication() {
        // Test TO deduplication
        let recipients = Recipients::to(vec![
            EmailAddress::new("user@example.com"),
            EmailAddress::new("user@example.com"), // duplicate
            EmailAddress::new("other@example.com"),
        ]);
        assert_eq!(recipients.to.as_ref().unwrap().len(), 2);

        // Test CC deduplication
        let recipients = Recipients::cc(vec![
            EmailAddress::new("cc1@example.com"),
            EmailAddress::new("cc1@example.com"), // duplicate
        ]);
        assert_eq!(recipients.cc.as_ref().unwrap().len(), 1);

        // Test BCC deduplication
        let recipients = Recipients::bcc(vec![
            EmailAddress::new("bcc@example.com"),
            EmailAddress::new("bcc@example.com"), // duplicate
            EmailAddress::new("bcc@example.com"), // duplicate
        ]);
        assert_eq!(recipients.bcc.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_recipients_add_methods_deduplication() {
        // Test adding duplicates
        let recipients = Recipients::to(vec![EmailAddress::new("to@example.com")]).add_to(vec![
            EmailAddress::new("to@example.com"), // duplicate of existing
            EmailAddress::new("to2@example.com"),
        ]);
        assert_eq!(recipients.to.as_ref().unwrap().len(), 2);

        // Test multiple add operations with duplicates
        let recipients = Recipients::to(vec![EmailAddress::new("user1@example.com")])
            .add_to(vec![EmailAddress::new("user2@example.com")])
            .add_to(vec![
                EmailAddress::new("user1@example.com"), // duplicate
                EmailAddress::new("user3@example.com"),
            ]);
        assert_eq!(recipients.to.as_ref().unwrap().len(), 3);
    }

    #[test]
    fn test_recipients_builder_deduplication() {
        // Test builder with duplicates
        let recipients = Recipients::builder()
            .to(vec![
                EmailAddress::new("to@example.com"),
                EmailAddress::new("to@example.com"), // duplicate
            ])
            .cc(vec![
                EmailAddress::new("cc@example.com"),
                EmailAddress::new("cc@example.com"), // duplicate
            ])
            .build();

        assert_eq!(recipients.to.as_ref().unwrap().len(), 1);
        assert_eq!(recipients.cc.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_email_address_with_name_deduplication() {
        // Same email with different names should be treated as duplicates
        let recipients = Recipients::to(vec![
            EmailAddress::new("user@example.com"),
            EmailAddress::with_name("user@example.com", "User Name"),
        ]);

        // Should keep only one (the first one encountered)
        assert_eq!(recipients.to.as_ref().unwrap().len(), 1);
    }
}
