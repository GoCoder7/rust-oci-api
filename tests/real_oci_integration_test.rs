//! Real OCI API integration tests
//!
//! These tests require actual OCI credentials and will make real API calls.
//! They are ignored by default and must be explicitly run with:
//! ```
//! cargo test --test real_oci_integration_test -- --ignored
//! ```
//!
//! Required environment variables:
//! - OCI_USER_ID
//! - OCI_TENANCY_ID
//! - OCI_REGION
//! - OCI_FINGERPRINT
//! - OCI_PRIVATE_KEY (file path or PEM content)
//! - OCI_COMPARTMENT_ID (optional, defaults to tenancy ID)
//! - TEST_SENDER_EMAIL (optional, for email tests)
//! - TEST_RECIPIENT_EMAIL (optional, for email tests)

use oci_api::auth::OciConfig;
use oci_api::client::OciClient;
use oci_api::services::email::{Email, EmailAddress, EmailClient, Recipients, Sender};

/// Load .env file if it exists
fn load_env() {
    let _ = dotenvy::dotenv();
}

/// Helper to check if OCI credentials are configured
fn has_oci_credentials() -> bool {
    load_env();
    std::env::var("OCI_USER_ID").is_ok()
        && std::env::var("OCI_TENANCY_ID").is_ok()
        && std::env::var("OCI_REGION").is_ok()
        && std::env::var("OCI_FINGERPRINT").is_ok()
        && std::env::var("OCI_PRIVATE_KEY").is_ok()
}

#[tokio::test]
#[ignore] // Run with: cargo test --test real_oci_integration_test -- --ignored
async fn test_oci_config_from_env() {
    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials not configured");
        return;
    }

    let config = OciConfig::from_env().expect("Failed to load OCI config from environment");

    assert!(!config.user_id.is_empty());
    assert!(!config.tenancy_id.is_empty());
    assert!(!config.region.is_empty());
    assert!(!config.fingerprint.is_empty());
    assert!(!config.private_key.is_empty());
    assert!(config.private_key.contains("-----BEGIN"));
    assert!(config.private_key.contains("-----END"));
}

#[tokio::test]
#[ignore]
async fn test_oci_client_creation_from_env() {
    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials not configured");
        return;
    }

    let config = OciConfig::from_env().expect("Failed to load config");
    let client = OciClient::new(&config).expect("Failed to create OCI client");

    assert_eq!(client.region(), config.region);
}

#[tokio::test]
#[ignore]
async fn test_get_email_configuration() {
    let _ = env_logger::builder().is_test(true).try_init();

    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials not configured");
        return;
    }

    let config = OciConfig::from_env().expect("Failed to load config");
    let tenancy_id = config.tenancy_id.clone();

    println!("Config loaded:");
    println!("  Tenancy: {}", tenancy_id);
    println!("  Region: {}", config.region);
    println!("  Private key length: {}", config.private_key.len());

    let oci_client = OciClient::new(&config).expect("Failed to create OCI client");
    let email_client = EmailClient::new(oci_client)
        .await
        .expect("Failed to create EmailClient");

    let result = email_client.get_email_configuration(&tenancy_id).await;

    match result {
        Ok(email_config) => {
            assert!(!email_config.compartment_id.is_empty());
            assert!(!email_config.http_submit_endpoint.is_empty());
            println!("Email configuration retrieved successfully:");
            println!("  Compartment ID: {}", email_config.compartment_id);
            println!(
                "  HTTP Submit Endpoint: {}",
                email_config.http_submit_endpoint
            );
            println!(
                "  SMTP Submit Endpoint: {}",
                email_config.smtp_submit_endpoint
            );
        }
        Err(e) => {
            panic!("Failed to get email configuration: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_send_full_flow() {
    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials not configured");
        return;
    }

    // Skip this test - it requires approved sender email
    // Use test_send_with_real_sender instead
    eprintln!(
        "Skipping test: This test requires TEST_SENDER_EMAIL environment variable with an approved sender."
    );
    eprintln!("Use test_send_with_real_sender instead, which auto-detects approved senders.");
    return;

    // Get sender and recipient from env or use defaults for testing
    let test_sender =
        std::env::var("TEST_SENDER_EMAIL").unwrap_or_else(|_| "sender@example.com".to_string());
    let test_recipient = std::env::var("TEST_RECIPIENT_EMAIL")
        .unwrap_or_else(|_| "recipient@example.com".to_string());

    let config = OciConfig::from_env().expect("Failed to load config");
    let tenancy_id = config.tenancy_id.clone();
    let oci_client = OciClient::new(&config).expect("Failed to create OCI client");
    let email_client = EmailClient::new(oci_client)
        .await
        .expect("Failed to create EmailClient");

    // Create email request
    let email_request = Email {
        message_id: None,
        sender: Sender {
            sender_address: EmailAddress::with_name(&test_sender, "OCI API Test"),
            compartment_id: String::new(),
        },
        recipients: Recipients::to(vec![EmailAddress::new(&test_recipient)]),
        subject: "Test Email from OCI API Rust Client".to_string(),
        body_html: Some(
            "<html><body>\
             <h1>Test Email</h1>\
             <p>This is a test email sent from the OCI API Rust client integration test.</p>\
             </body></html>"
                .to_string(),
        ),
        body_text: Some(
            "Test Email\n\nThis is a test email sent from the OCI API Rust client integration test."
                .to_string(),
        ),
        reply_to: None,
        headers: None,
    };

    // Submit email
    let result = email_client.send(email_request).await;

    match result {
        Ok(response) => {
            assert!(!response.message_id.is_empty());
            assert!(!response.envelope_id.is_empty());
            println!("Email submitted successfully:");
            println!("  Message ID: {}", response.message_id);
            println!("  Envelope ID: {}", response.envelope_id);
        }
        Err(e) => {
            panic!("Failed to submit email: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_email_delivery_endpoint_caching() {
    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials not configured");
        return;
    }

    let config = OciConfig::from_env().expect("Failed to load config");
    let tenancy_id = config.tenancy_id.clone();
    let oci_client = OciClient::new(&config).expect("Failed to create OCI client");
    let email_client = EmailClient::new(oci_client)
        .await
        .expect("Failed to create EmailClient");

    // First call - email_client is now immutable
    let config1 = email_client
        .get_email_configuration(&tenancy_id)
        .await
        .expect("Failed to get email configuration");

    // Second call - uses the same immutable client
    let config2 = email_client
        .get_email_configuration(&tenancy_id)
        .await
        .expect("Failed to get email configuration on second call");

    assert_eq!(config1.compartment_id, config2.compartment_id);
    assert_eq!(config1.http_submit_endpoint, config2.http_submit_endpoint);
}

#[tokio::test]
#[ignore]
async fn test_list_senders() {
    let _ = env_logger::builder().is_test(true).try_init();

    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials not configured");
        return;
    }

    let config = OciConfig::from_env().expect("Failed to load config");
    let compartment_id =
        std::env::var("OCI_COMPARTMENT_ID").unwrap_or_else(|_| config.tenancy_id.clone());

    println!("Listing senders in compartment: {}", compartment_id);

    let oci_client = OciClient::new(&config).expect("Failed to create OCI client");
    let email_client = EmailClient::new(oci_client)
        .await
        .expect("Failed to create EmailClient");

    // Test: list all senders
    let result = email_client.list_senders(&compartment_id, None, None).await;

    match result {
        Ok(senders) => {
            println!("Found {} approved senders:", senders.len());
            for sender in &senders {
                println!(
                    "  - {} ({:?})",
                    sender.email_address, sender.lifecycle_state
                );
                println!("    ID: {}", sender.id);
                println!("    Created: {}", sender.time_created);
            }

            // Test: filter by ACTIVE state
            if !senders.is_empty() {
                let active_senders = email_client
                    .list_senders(&compartment_id, Some("ACTIVE"), None)
                    .await
                    .expect("Failed to list active senders");

                println!("\nActive senders: {}", active_senders.len());
                assert!(
                    active_senders.iter().all(|s| s.lifecycle_state
                        == oci_api::services::email::SenderLifecycleState::Active)
                );
            }
        }
        Err(e) => {
            panic!("Failed to list senders: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_send_with_real_sender() {
    let _ = env_logger::builder().is_test(true).try_init();

    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials not configured");
        return;
    }

    let config = OciConfig::from_env().expect("Failed to load config");
    let tenancy_id = config.tenancy_id.clone();
    let compartment_id = std::env::var("OCI_COMPARTMENT_ID").unwrap_or_else(|_| tenancy_id.clone());

    let oci_client = OciClient::new(&config).expect("Failed to create OCI client");
    let email_client = EmailClient::new(oci_client)
        .await
        .expect("Failed to create EmailClient");

    // Get approved senders first
    let senders = email_client
        .list_senders(&compartment_id, Some("ACTIVE"), None)
        .await
        .expect("Failed to list senders");

    if senders.is_empty() {
        eprintln!("No active approved senders found. Skipping email submission test.");
        eprintln!("Please configure an approved sender in your OCI compartment first.");
        return;
    }

    let approved_sender = &senders[0];
    println!("Using approved sender: {}", approved_sender.email_address);

    // Get recipient from env or use the same sender for testing
    let test_recipient = std::env::var("TEST_RECIPIENT_EMAIL")
        .unwrap_or_else(|_| approved_sender.email_address.clone());

    // Create email request with your example format
    let email_request = Email {
        message_id: None,
        sender: Sender {
            sender_address: EmailAddress::with_name(&approved_sender.email_address, "GoCoder"),
            compartment_id: String::new(),
        },
        recipients: Recipients::to(vec![EmailAddress::new(&test_recipient)]),
        subject: "test subject".to_string(),
        body_text: Some("test body without tags".to_string()),
        body_html: Some(
            "<h1 style='background-color: red'>test body</h1><div>hello</div>".to_string(),
        ),
        reply_to: None,
        headers: None,
    };

    // Submit email
    let result = email_client.send(email_request).await;

    match result {
        Ok(response) => {
            println!("✅ Email submitted successfully!");
            println!("  Message ID: {}", response.message_id);
            println!("  Envelope ID: {}", response.envelope_id);
            if let Some(ref suppressed) = response.suppressed_recipients {
                println!("  Suppressed recipients: {} recipients", suppressed.len());
                for recipient in suppressed {
                    println!("    - {}", recipient.email);
                }
            }

            assert!(!response.message_id.is_empty());
            assert!(!response.envelope_id.is_empty());
        }
        Err(e) => {
            panic!("Failed to submit email: {:?}", e);
        }
    }
}
