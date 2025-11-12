# oci-api

A Rust client library for Oracle Cloud Infrastructure (OCI) APIs.

Currently supports:
- **Email Delivery Service** - Send emails via OCI Email Delivery

## Features

- 🔐 OCI HTTP request signing (compliant with OCI specifications)
- 📧 Email Delivery API support
- 🔄 Async/await support (Tokio)
- 🛡️ Type-safe API with comprehensive error handling
- ⚙️ Flexible configuration (environment variables, config files, or programmatic)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oci-api = "0.1"
tokio = { version = "1", features = ["full"] }
```

**Import commonly used types:**

```rust
use oci_api::{OciConfig, OciClient};
use oci_api::email::{EmailClient, Email, EmailAddress, Recipients};
```


## Configuration

There are two ways to configure OCI credentials which are used for generating(signing) `Authorization` headers and requests:

### Option 1: Environment Variables (Recommended)

**Using `OCI_CONFIG` (supports both file path and INI content directly)**

`OCI_CONFIG` can provide the following information:
- `user` → `user_id`
- `tenancy` → `tenancy_id`
- `region`
- `fingerprint`
- `key_file` → path to private key file


```bash
# use dotenvy or similar to load environment variables from `.env` in development

# point to a config file path
OCI_CONFIG=/path/to/.oci/config

# or provide content(INI) directly
OCI_CONFIG="[DEFAULT]
user=ocid1.user.oc1..aaaaaa...
tenancy=ocid1.tenancy.oc1..aaaaaa...
region=ap-chuncheon-1
fingerprint=aa:bb:cc:dd:ee:ff:11:22:33:44:55:66:77:88:99:00
key_file=~/.oci/private-key.pem"
```

**Using `OCI_PRIVATE_KEY` (supports both file path and PEM content directly):**
```bash
# it overrides the private key specified in OCI_CONFIG if both are set

# Provide private key file path
OCI_PRIVATE_KEY=/path/to/private-key.pem
# or provide PEM content directly:
OCI_PRIVATE_KEY="-----BEGIN PRIVATE KEY-----
MIIEvwIBADANBgk...
-----END PRIVATE KEY-----"
```

**Individual environment variables override `OCI_CONFIG` example:**

```bash

# if you use individual vars, you don't need to set OCI_CONFIG
# but you can still use it as a base
OCI_CONFIG=/path/to/.oci/config

# Override specific values (higher priority than OCI_CONFIG)
OCI_USER_ID=ocid1.user.oc1..different...      # Overrides 'user' from config
OCI_TENANCY_ID=ocid1.tenancy.oc1..different...  # Overrides 'tenancy' from config
OCI_REGION=ap-seoul-1                          # Overrides 'region' from config
OCI_FINGERPRINT=11:22:33:44:55:66:77:88:99:00:aa:bb:cc:dd:ee:ff  # Overrides 'fingerprint'
OCI_PRIVATE_KEY=/different/path/to/key.pem    # Overrides 'key_file' from config
OCI_COMPARTMENT_ID=ocid1.compartment.oc1..aaaaaa...  # Optional, defaults to tenancy_id, but needed for APIs if you use specific compartment
```

**Load configuration:**

```rust
use oci_api::OciConfig;

let config = OciConfig::from_env()?;
```

**Priority Summary:**

| Field | Priority 1  | Priority 2 | 
|-------|---------------------|------------|
| User ID | `OCI_USER_ID` | `user` from `OCI_CONFIG` | 
| Tenancy ID | `OCI_TENANCY_ID` | `tenancy` from `OCI_CONFIG` | 
| Region | `OCI_REGION` | `region` from `OCI_CONFIG` | 
| Fingerprint | `OCI_FINGERPRINT` | `fingerprint` from `OCI_CONFIG` | 
| Private Key | `OCI_PRIVATE_KEY` (file path or content) | `key_file` from `OCI_CONFIG` | 
| Compartment ID | `OCI_COMPARTMENT_ID` | Defaults to `tenancy_id` | 

\* `OCI_USER_ID`, `OCI_TENANCY_ID`, `OCI_REGION`, `OCI_FINGERPRINT`, and `OCI_PRIVATE_KEY` are required if `OCI_CONFIG` is not set.
\* `OCI_PRIVATE_KEY` is recommended even if `OCI_CONFIG` is used, if you do not want to change the config file content between environments.

---

### Option 2: Programmatic Configuration

```rust
use oci_api::OciConfig;

// build from scratch using individual fields
let config = OciConfig::builder()
    .user_id("ocid1.user.oc1..aaaaaa...")
    .tenancy_id("ocid1.tenancy.oc1..aaaaaa...")
    .region("ap-chuncheon-1")
    .fingerprint("aa:bb:cc:dd:ee:ff:11:22:33:44:55:66:77:88:99:00")
    .private_key("/path/to/private-key.pem")?
    .compartment_id("ocid1.compartment.oc1..aaaaaa...")
    .build()?;

// or load from config file and override specific fields
let config = OciConfig::builder()
    .config("/path/to/.oci/config")?  // Load from file
    .private_key("/production/path/to/key.pem")?  // Override key_file from config
    .compartment_id("ocid1.compartment.oc1..aaaaaa...")  // Set compartment
    .build()?;

```







## Email Delivery API

```rust
use oci_api::{OciConfig, OciClient};
use oci_api::email::{EmailClient, Email, EmailAddress, Recipients};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load configuration and create clients
    let config = OciConfig::from_env()?;
    let oci_client = OciClient::new(&config)?;
    let email_client = EmailClient::new(oci_client).await?;
    
    // Prepare email
    let email = Email::builder()
        .sender(EmailAddress::new("approved-sender@example.com"))  // Must be an approved sender
        .recipients(Recipients::to(vec![EmailAddress::new("recipient@example.com")]))
        .subject("Hello from OCI!")
        .body_html("<h1>This is a test email</h1><p>Sent via <strong>OCI Email Delivery API</strong>.</p>")
        .body_text("This is a test email sent via OCI Email Delivery API.")
        .build()?;
    
    // Send email (compartment_id is automatically injected from OciClient)
    let response = email_client.send(email).await?;
    println!("Email sent! Message ID: {}", response.message_id);
    
    Ok(())
}
```

### Body Text & HTML

you can send body as text or HTML or both, but at least one is required. if both are provided(recommended), email clients will choose HTML if available, otherwise plain text.

```rust
use oci_api::{OciConfig, OciClient};
use oci_api::email::{EmailClient, Email, EmailAddress, Recipients};

let email = Email::builder()
    .sender(EmailAddress::new("approved-sender@example.com"))
    .recipients(Recipients::to(vec![EmailAddress::new("user@example.com")]))
    .subject("Simple Email")
    .body_html("<h1>Hello</h1><p>This is <strong>HTML</strong> content.</p>")
    .body_text("Plain text content")
    .build()?;

let response = email_client.send(email).await?;
```


### Email Address

EmailAddress is used for specifying sender, recipients, reply-to, etc. it can be created with just an email(`new`) or with a display name(`with_name`).

```rust
let just_email = EmailAddress::new("user@example.com");
let with_name = EmailAddress::with_name("user@example.com", "User Name");
```

#### Recipients

You can use multiple Recipients constructors(`to`(=`new`), `cc`, `bcc`) or builder pattern.
and you can also add more recipients using `add_to`, `add_cc`, `add_bcc` methods.
each `to`, `cc`, `bcc` vectors will be unique by `EmailAddress.email` when constructed or added.

```rust
// Option 1: Using builder pattern (flexible for multiple fields)
let email = Email::builder()
    .sender(EmailAddress::new("approved-sender@example.com"))
    .subject("Group Email")
    .body_text("This email has CC and BCC recipients")
    .recipients(
        Recipients::builder()
            .to(vec![
                EmailAddress::new("to1@example.com"),
                EmailAddress::with_name("to1@example.com", "to1"), // duplicate, will be ignored
                EmailAddress::with_name("to2@example.com", "User Two"),
            ])
            .cc(vec![EmailAddress::new("cc@example.com")])
            .bcc(vec![EmailAddress::new("bcc@example.com")])
            .build()
    )
    .build()?;

// Option 2: Using specific constructor and add with `add_*` methods (chainable)
let email = Email::builder()
    .sender(EmailAddress::new("approved-sender@example.com"))
    .subject("Group Email")
    .body_text("This email has CC and BCC recipients")
    .recipients(
        Recipients::to(vec![EmailAddress::new("to@example.com")])
            .add_to(vec![EmailAddress::with_name("to@example.com", "To User")]) // duplicate, will be ignored
            .add_cc(vec![EmailAddress::new("cc@example.com")])
            .add_bcc(vec![EmailAddress::new("bcc@example.com")])
    )
    .build()?;

let response = email_client.send(email).await?;
```

You can also use `headers`(headerFields), `reply_to`(replyTo), and `message_id`(messageId) fields in `Email` struct. you can reference [here](https://docs.oracle.com/en-us/iaas/api/#/en/emaildeliverysubmission/20220926/datatypes/SubmitEmailDetails)

For OCI Email Delivery documentation, see:
- [OCI Email Delivery Overview](https://docs.oracle.com/en-us/iaas/Content/Email/home.htm)
- [OCI Email Delivery API Reference](https://docs.oracle.com/en-us/iaas/api/#/en/emaildelivery/20170907/)
- [OCI Email Delivery Submission API Reference](https://docs.oracle.com/en-us/iaas/api/#/en/emaildeliverysubmission/20220926/)

<br>

## Error Handling

The library provides comprehensive error types:

```rust
use oci_api::{OciError, Result};

match email_client.send(email).await {
    Ok(response) => println!("Sent: {}", response.message_id),
    Err(OciError::ApiError(status, body)) => {
        eprintln!("API error {}: {}", status, body);
    }
    Err(OciError::AuthError(msg)) => {
        eprintln!("Authentication error: {}", msg);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

Error types:
- `ConfigError` - Configuration loading/validation errors
- `EnvError` - Environment variable errors
- `KeyError` - Private key loading errors
- `AuthError` - Authentication/signing errors
- `ApiError` - OCI API errors (with HTTP status and response body)
- `NetworkError` - Network/HTTP client errors
- `IniError` - Config file parsing errors
- `Other` - Other errors


## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.


## Support

For issues and feature requests, please use [GitHub Issues](https://github.com/GoCoder7/rust-oci-api/issues).

