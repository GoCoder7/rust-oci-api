//! Send a test email to go@gocoder.xyz

use oci_api::client::Oci;
use oci_api::services::email::{Email, EmailAddress, EmailDelivery, Recipients};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file if exists
    let _ = dotenvy::dotenv();

    println!("🚀 Sending test email to go@gocoder.xyz...\n");

    // Load configuration
    let oci_client = Oci::from_env()?;
    let tenancy_id = oci_client.tenancy_id().to_string();
    let compartment_id = std::env::var("OCI_COMPARTMENT_ID").unwrap_or_else(|_| tenancy_id.clone());

    // Create clients
    let email_client = EmailDelivery::new(oci_client).await?;

    // Get approved senders
    println!("📋 Fetching approved senders...");
    let senders = email_client
        .list_senders(&compartment_id, Some("ACTIVE"), None)
        .await?;

    if senders.is_empty() {
        eprintln!("❌ No approved senders found!");
        return Ok(());
    }

    let approved_sender = &senders[0];
    println!(
        "✉️  Using sender: {sender_email}\n",
        sender_email = approved_sender.email_address
    );

    // Build email content using builder pattern
    let email = Email::builder()
        .sender(EmailAddress::with_name(
            &approved_sender.email_address,
            "GoCoder Test System",
        ))
        .recipients(Recipients::to(vec![EmailAddress::new("go@gocoder.xyz")]))
        .subject("🧪 OCI API Rust Client Test Email")
        .body_text(
            "Hello!\n\n\
             This is a test email sent from the OCI API Rust client.\n\n\
             If you're seeing this, it means the email delivery system is working correctly! 🎉\n\n\
             Best regards,\n\
             GoCoder Test System"
        )
        .body_html(
            r#"
            <!DOCTYPE html>
            <html>
            <head>
                <style>
                    body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; }
                    .container { max-width: 600px; margin: 0 auto; padding: 20px; }
                    .header { background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
                              color: white; padding: 30px; border-radius: 10px 10px 0 0; text-align: center; }
                    .content { background: #f9f9f9; padding: 30px; border-radius: 0 0 10px 10px; }
                    .badge { background: #4CAF50; color: white; padding: 5px 10px; 
                             border-radius: 5px; display: inline-block; margin: 10px 0; }
                    .footer { text-align: center; margin-top: 20px; color: #666; font-size: 12px; }
                </style>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>🧪 Test Email</h1>
                        <p>OCI API Rust Client</p>
                    </div>
                    <div class="content">
                        <p>Hello!</p>
                        <div class="badge">✅ System Test</div>
                        <p>This is a test email sent from the <strong>OCI API Rust client</strong>.</p>
                        <p>If you're seeing this, it means the email delivery system is working correctly! 🎉</p>
                        <p><strong>Technical Details:</strong></p>
                        <ul>
                            <li>API: OCI Email Delivery Service</li>
                            <li>Client: Rust Implementation</li>
                            <li>Region: ap-chuncheon-1</li>
                            <li>Status: <span style="color: #4CAF50;">✓ Operational</span></li>
                        </ul>
                        <p>Best regards,<br>GoCoder Test System</p>
                    </div>
                    <div class="footer">
                        <p>This is an automated test email. Please do not reply.</p>
                    </div>
                </div>
            </body>
            </html>
            "#
        )
        .build()?;

    // Send email
    println!("📤 Sending email...");
    match email_client.send(email).await {
        Ok(response) => {
            println!("\n✅ Email sent successfully!");
            println!(
                "📨 Message ID: {message_id}",
                message_id = response.message_id
            );
            println!(
                "📮 Envelope ID: {envelope_id}",
                envelope_id = response.envelope_id
            );

            if let Some(suppressed) = response.suppressed_recipients
                && !suppressed.is_empty()
            {
                println!("\n⚠️  Note: Some recipients were suppressed:");
                for recipient in suppressed {
                    println!("   - {email}", email = recipient.email);
                }
            }

            println!("\n💌 Check go@gocoder.xyz inbox!");
        }
        Err(e) => {
            eprintln!("\n❌ Failed to send email: {e}");
            return Err(e.into());
        }
    }

    Ok(())
}
