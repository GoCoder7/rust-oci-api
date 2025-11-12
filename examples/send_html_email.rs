//! HTML email sending example
//!
//! This example demonstrates how to send an HTML email with a plain text fallback.
//!
//! ## Prerequisites
//!
//! Set the following environment variables (or use .env file):
//! - `OCI_USER_ID`: Your user OCID
//! - `OCI_TENANCY_ID`: Your tenancy OCID
//! - `OCI_REGION`: Your OCI region (e.g., "us-ashburn-1")
//! - `OCI_FINGERPRINT`: Your API key fingerprint
//! - `OCI_PRIVATE_KEY`: Path to your private key file or PEM content
//! - `OCI_COMPARTMENT_ID`: (Optional) Your compartment OCID, defaults to tenancy
//!
//! ## Run
//!
//! ```bash
//! cargo run --example send_html_email
//! ```

use oci_api::auth::OciConfig;
use oci_api::client::OciClient;
use oci_api::services::email::{Email, EmailAddress, EmailClient, Recipients};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if exists
    let _ = dotenvy::dotenv();

    // Load configuration from environment variables
    println!("Loading OCI configuration from environment variables...");
    let config = OciConfig::from_env()?;
    let tenancy_id = config.tenancy_id.clone();
    let compartment_id = std::env::var("OCI_COMPARTMENT_ID").unwrap_or_else(|_| tenancy_id.clone());

    // Create OCI HTTP client
    println!("Creating OCI HTTP client...");
    let oci_client = OciClient::new(&config)?;

    // Create Email Delivery client
    println!("Creating Email Delivery client...");
    let email_client = EmailClient::new(oci_client).await?;

    // Get approved senders
    println!("Fetching approved senders...");
    let senders = email_client
        .list_senders(&compartment_id, Some("ACTIVE"), None)
        .await?;

    if senders.is_empty() {
        eprintln!("❌ No approved senders found!");
        eprintln!("Please configure an approved sender in your OCI compartment first.");
        return Ok(());
    }

    let approved_sender = &senders[0];
    println!("Using approved sender: {}", approved_sender.email_address);

    // Build HTML content
    let html_body = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <style>
                body { font-family: Arial, sans-serif; }
                .header { background-color: #4CAF50; color: white; padding: 20px; text-align: center; }
                .content { padding: 20px; }
                .footer { background-color: #f1f1f1; padding: 10px; text-align: center; font-size: 12px; }
            </style>
        </head>
        <body>
            <div class="header">
                <h1>Welcome to Our Newsletter!</h1>
            </div>
            <div class="content">
                <p>Dear Subscriber,</p>
                <p>Thank you for joining our newsletter. This is a sample HTML email sent using the <strong>OCI Email Delivery API</strong>.</p>
                <p>We're excited to keep you updated with our latest news and updates!</p>
            </div>
            <div class="footer">
                <p>© 2025 Example App. All rights reserved.</p>
                <p>You can unsubscribe at any time.</p>
            </div>
        </body>
        </html>
    "#;

    // Plain text fallback
    let text_body = r#"
Welcome to Our Newsletter!

Dear Subscriber,

Thank you for joining our newsletter. This is a sample HTML email sent using the OCI Email Delivery API.

We're excited to keep you updated with our latest news and updates!

---
© 2025 Example App. All rights reserved.
You can unsubscribe at any time.
    "#;

    // Build email using builder pattern
    let email = Email::builder()
        .sender(EmailAddress::with_name(
            &approved_sender.email_address,
            "Example Newsletter",
        ))
        .recipients(Recipients::to(vec![EmailAddress::new(
            &approved_sender.email_address,
        )]))
        .subject("📧 Welcome to Our Newsletter!")
        .body_html(html_body)
        .body_text(text_body)
        .build()?;

    // Send email
    println!("Sending HTML email...");
    match email_client.send(email).await {
        Ok(response) => {
            println!("✅ HTML email sent successfully!");
            println!("Message ID: {}", response.message_id);
            println!("Envelope ID: {}", response.envelope_id);
            if let Some(suppressed) = response.suppressed_recipients {
                if !suppressed.is_empty() {
                    println!("⚠️  Suppressed recipients:");
                    for recipient in suppressed {
                        println!("  - {}", recipient.email);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to send email: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
