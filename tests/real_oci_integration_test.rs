//! Real OCI API integration tests
//!
//! These tests require actual OCI credentials and will make real API calls.
//! They are ignored by default and must be explicitly run with:
//! ```
//! cargo test --test real_oci_integration_test -- --ignored
//! ```
//!
//! Required authentication:
//! - `OCI_AUTH_MODE=api_key` with `OCI_USER_ID`, `OCI_TENANCY_ID`, `OCI_REGION`,
//!   `OCI_FINGERPRINT`, `OCI_PRIVATE_KEY`
//! - or `OCI_AUTH_MODE=instance_principal` on an OCI runtime where IMDS is reachable
//!
//! Additional optional environment variables:
//! - OCI_COMPARTMENT_ID (optional, defaults to tenancy ID)
//! - TEST_SENDER_EMAIL (optional, for email tests)
//! - TEST_RECIPIENT_EMAIL (optional, for email tests)

use oci_api::client::Oci;
use oci_api::services::email::{Email, EmailAddress, EmailDelivery, Recipients, Sender};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// Load .env file if it exists
fn load_env() {
    let _ = dotenvy::dotenv();
}

/// Helper to check if OCI credentials are configured
fn has_oci_credentials() -> bool {
    load_env();
    match std::env::var("OCI_AUTH_MODE")
        .unwrap_or_else(|_| "api_key".to_owned())
        .as_str()
    {
        "instance_principal" => is_imds_reachable(),
        "api_key" => {
            std::env::var("OCI_USER_ID").is_ok()
                && std::env::var("OCI_TENANCY_ID").is_ok()
                && std::env::var("OCI_REGION").is_ok()
                && std::env::var("OCI_FINGERPRINT").is_ok()
                && std::env::var("OCI_PRIVATE_KEY").is_ok()
        }
        _ => false,
    }
}

fn is_imds_reachable() -> bool {
    let address: SocketAddr = "169.254.169.254:80".parse().unwrap();
    TcpStream::connect_timeout(&address, Duration::from_millis(500)).is_ok()
}

#[tokio::test]
#[ignore] // Run with: cargo test --test real_oci_integration_test -- --ignored
async fn test_oci_client_from_env() {
    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials not configured");
        return;
    }

    let oci = Oci::from_env().expect("Failed to load OCI config from environment");

    assert!(!oci.signer().user_id().is_empty());
    assert!(!oci.tenancy_id().is_empty());
    assert!(!oci.region().is_empty());
    assert!(!oci.signer().fingerprint().is_empty());
}

#[tokio::test]
#[ignore]
async fn test_get_email_configuration() {
    let _ = env_logger::builder().is_test(true).try_init();

    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials not configured");
        return;
    }

    let oci_client = Oci::from_env().expect("Failed to create OCI client");
    let tenancy_id = oci_client.tenancy_id().to_string();

    println!("Config loaded:");
    println!("  Tenancy: {tenancy_id}");
    println!("  Region: {region}", region = oci_client.region());

    let email_client = EmailDelivery::new(oci_client)
        .await
        .expect("Failed to create EmailDelivery");

    let result = email_client.get_email_configuration(&tenancy_id).await;

    match result {
        Ok(email_config) => {
            assert!(!email_config.compartment_id.is_empty());
            assert!(!email_config.http_submit_endpoint.is_empty());
            println!("Email configuration retrieved successfully:");
            println!(
                "  Compartment ID: {compartment_id}",
                compartment_id = email_config.compartment_id
            );
            println!(
                "  HTTP Submit Endpoint: {http_submit_endpoint}",
                http_submit_endpoint = email_config.http_submit_endpoint
            );
            println!(
                "  SMTP Submit Endpoint: {smtp_submit_endpoint}",
                smtp_submit_endpoint = email_config.smtp_submit_endpoint
            );
        }
        Err(e) => {
            panic!("Failed to get email configuration: {e:?}");
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
}

#[tokio::test]
#[ignore]
async fn test_email_delivery_endpoint_caching() {
    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials not configured");
        return;
    }

    let oci_client = Oci::from_env().expect("Failed to load config");
    let tenancy_id = oci_client.tenancy_id().to_string();
    let email_client = EmailDelivery::new(oci_client)
        .await
        .expect("Failed to create EmailDelivery");

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

    let oci_client = Oci::from_env().expect("Failed to load config");
    let compartment_id =
        std::env::var("OCI_COMPARTMENT_ID").unwrap_or_else(|_| oci_client.tenancy_id().to_string());

    println!("Listing senders in compartment: {compartment_id}");

    let email_client = EmailDelivery::new(oci_client)
        .await
        .expect("Failed to create EmailDelivery");

    // Test: list all senders
    let result = email_client.list_senders(&compartment_id, None, None).await;

    match result {
        Ok(senders) => {
            println!(
                "Found {sender_count} approved senders:",
                sender_count = senders.len()
            );
            for sender in &senders {
                println!(
                    "  - {email} ({state:?})",
                    email = sender.email_address,
                    state = sender.lifecycle_state
                );
                println!("    ID: {id}", id = sender.id);
                println!(
                    "    Created: {time_created}",
                    time_created = sender.time_created
                );
            }

            // Test: filter by ACTIVE state
            if !senders.is_empty() {
                let active_senders = email_client
                    .list_senders(&compartment_id, Some("ACTIVE"), None)
                    .await
                    .expect("Failed to list active senders");

                println!(
                    "\nActive senders: {active_sender_count}",
                    active_sender_count = active_senders.len()
                );
                assert!(
                    active_senders.iter().all(|s| s.lifecycle_state
                        == oci_api::services::email::SenderLifecycleState::Active)
                );
            }
        }
        Err(e) => {
            panic!("Failed to list senders: {e:?}");
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

    let oci_client = Oci::from_env().expect("Failed to load config");
    let tenancy_id = oci_client.tenancy_id().to_string();
    let compartment_id = std::env::var("OCI_COMPARTMENT_ID").unwrap_or_else(|_| tenancy_id.clone());

    let email_client = EmailDelivery::new(oci_client)
        .await
        .expect("Failed to create EmailDelivery");

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
    println!(
        "Using approved sender: {sender_email}",
        sender_email = approved_sender.email_address
    );

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
            println!(
                "  Message ID: {message_id}",
                message_id = response.message_id
            );
            println!(
                "  Envelope ID: {envelope_id}",
                envelope_id = response.envelope_id
            );
            if let Some(ref suppressed) = response.suppressed_recipients {
                println!(
                    "  Suppressed recipients: {suppressed_count} recipients",
                    suppressed_count = suppressed.len()
                );
                for recipient in suppressed {
                    println!("    - {email}", email = recipient.email);
                }
            }

            assert!(!response.message_id.is_empty());
            assert!(!response.envelope_id.is_empty());
        }
        Err(e) => {
            panic!("Failed to submit email: {e:?}");
        }
    }
}
