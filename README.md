# oci-api

A Rust client library for Oracle Cloud Infrastructure (OCI) APIs.

Currently supports:
- **Email Delivery Service** - Send emails via OCI Email Delivery
- **Object Storage Service** - Manage buckets and objects
- **Vault Secrets Service** - Read current, staged, and versioned secret bundles
- **Keys Service** - Read keys and trigger rotation

## Features

- 🔐 OCI HTTP request signing (compliant with OCI specifications)
- 🔄 Dual auth modes: API key and Instance Principal
- 📧 Email Delivery API support
- 🗝️ Vault Secrets and Keys support
- 🔄 Async/await support (Tokio)
- 🛡️ Type-safe API with comprehensive error handling
- ⚙️ Flexible configuration (environment variables, config files, or programmatic)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
oci-api = "0.6.0"
tokio = { version = "1", features = ["full"] }
```

**Import commonly used types:**

```rust
use oci_api::Oci;
use oci_api::email::{EmailDelivery, Email, EmailAddress, Recipients};
use oci_api::keys::KeysClient;
use oci_api::object_storage::ObjectStorage;
use oci_api::vault::VaultSecretsClient;
```


## Configuration

`oci-api` supports two authentication modes via `OCI_AUTH_MODE`:

| Mode | Value | Typical runtime |
|------|-------|-----------------|
| API key | `api_key` (default) | local development, CI, explicit credential injection |
| Instance Principal | `instance_principal` | OCI Compute / Coolify-hosted runtime with instance identity |

There are two ways to configure OCI credentials which are used for generating(signing) `Authorization` headers and requests:

### Option 1: Environment Variables (Recommended)

**Using `OCI_CONFIG` (supports both file path and INI content directly)**

`OCI_CONFIG` can provide the following information:
- `user` → `user_id`
- `tenancy` → `tenancy_id`
- `region`
- `fingerprint`
- `key_file`: path to private key file


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
use oci_api::Oci;

let oci = Oci::from_env()?;
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

### Option 1-B: Instance Principal Runtime

When running on OCI infrastructure, switch to Instance Principal mode:

```bash
OCI_AUTH_MODE=instance_principal

# optional: override metadata endpoint for local mock tests
OCI_METADATA_BASE_URL=http://169.254.169.254/opc/v2
```

```rust
use oci_api::Oci;

let oci = Oci::from_env()?;
assert_eq!(oci.auth_mode(), oci_api::client::AuthMode::InstancePrincipal);
```

Notes:

- `OCI_REGION` and `OCI_TENANCY_ID` are optional in Instance Principal mode when OCI metadata is available.
- `OCI_REGION` is discovered from IMDS `regionInfo` and uses the canonical region identifier rather than the short-code-prone plain-text region endpoint.
- `OCI_TENANCY_ID` is discovered from the leaf identity certificate subject (`opc-tenant:` with `opc-identity:` fallback).
- The security token and session key are fetched lazily on the first signed request and refreshed automatically before expiry.
- Auth/service endpoint construction is realm-aware and uses the metadata-provided realm domain component.
- Local validation should use a mocked metadata/federation flow; end-to-end validation still requires an OCI-hosted runtime.

### Option 2: Programmatic Configuration

```rust
use oci_api::Oci;

// build from scratch using individual fields
let oci = Oci::builder()
    .user_id("ocid1.user.oc1..aaaaaa...")
    .tenancy_id("ocid1.tenancy.oc1..aaaaaa...")
    .region("ap-chuncheon-1")
    .fingerprint("aa:bb:cc:dd:ee:ff:11:22:33:44:55:66:77:88:99:00")
    .private_key("/path/to/private-key.pem")?
    .compartment_id("ocid1.compartment.oc1..aaaaaa...")
    .build()?;

// or load from config file and override specific fields
let oci = Oci::builder()
    .config("/path/to/.oci/config")?  // Load from file
    .private_key("/production/path/to/key.pem")?  // Override key_file from config
    .compartment_id("ocid1.compartment.oc1..aaaaaa...")  // Set compartment
    .build()?;

```







## Email Delivery API

```rust
use oci_api::Oci;
use oci_api::email::{EmailDelivery, Email, EmailAddress, Recipients};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // create an email delivery instance
    let oci = Oci::from_env()?;
    let email_delivery = EmailDelivery::new(oci).await?;
    // or chaining from oci
    let email_delivery = Oci::from_env()?.email_delivery().await?;
    
    // make an email
    let email = Email::builder()
        .sender(EmailAddress::new("approved-sender@example.com"))  // Must be an approved sender
        .recipients(Recipients::to(vec![EmailAddress::new("recipient@example.com")]))
        .subject("Hello from OCI!")
        .body_html("<h1>This is a test email</h1><p>Sent via <strong>OCI Email Delivery API</strong>.</p>")
        .body_text("This is a test email sent via OCI Email Delivery API.")
        .build()?;
    
    // send email
    let response = email_delivery.send(email).await?;
    println!("Email sent! Message ID: {}", response.message_id);
    
    Ok(())
}
```

### Body Text & HTML

you can send body as text or HTML or both, but at least one is required. if both are provided(recommended), email clients will choose HTML if available, otherwise plain text.

```rust
use oci_api::Oci;
use oci_api::email::{EmailDelivery, Email, EmailAddress, Recipients};

let email = Email::builder()
    .sender(EmailAddress::new("approved-sender@example.com"))
    .recipients(Recipients::to(vec![EmailAddress::new("user@example.com")]))
    .subject("Simple Email")
    .body_html("<h1>Hello</h1><p>This is <strong>HTML</strong> content.</p>")
    .body_text("Plain text content")
    .build()?;

let response = email_delivery.send(email).await?;
```


### Email Address

EmailAddress is used for specifying sender, recipients, reply-to, etc. it can be created with just an email(`new`) or with a display name(`with_name`).

```rust
let just_email = EmailAddress::new("user@example.com");
let with_name = EmailAddress::with_name("user@example.com", "User Name");
```

#### Recipients

Recipients needs at least one `to` or `cc` or `bcc` recipient.
You can use builder pattern or multiple Recipients constructors(`to`(=`new`), `cc`, `bcc`) to create recipients,
and you can also add more recipients using `add_to`, `add_cc`, `add_bcc` methods.
each `to`, `cc`, `bcc` recipients will be unique by `EmailAddress.email` when constructed or added.

```rust
// Option 1: Using builder pattern (flexible for multiple fields)
let email = Email::builder()
    .sender(EmailAddress::new("approved-sender@example.com"))
    .subject("Group Email")
    .body_text("This email has CC and BCC recipients")
    .recipients(
        Recipients::builder() // it must be built with at least one of `to`, `cc`, `bcc`
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
        Recipients::to(vec![EmailAddress::new("to@example.com")]) // create with `to` recipients
            .add_to(vec![
                EmailAddress::with_name("to@example.com", "To User"), // duplicate, will be ignored
                EmailAddress::new("to2@example.com"), // will be added to `to` recipients
            ])
            .add_cc(vec![EmailAddress::new("cc@example.com")])
            .add_bcc(vec![EmailAddress::new("bcc@example.com")])
    )
    .build()?;

let response = email_client.send(email).await?;
```

You can also use `headers`(headerFields), `reply_to`(replyTo), and `message_id`(messageId) fields in `Email` struct. you can reference [here](https://docs.oracle.com/en-us/iaas/api/#/en/emaildeliverysubmission/20220926/datatypes/SubmitEmailDetails)

### Testing with `EmailSender` trait

`EmailDelivery` implements the `EmailSender` trait, which allows you to inject mock implementations for testing:

```rust
use oci_api::email::{EmailSender, EmailDelivery, Email, SubmitEmailResponse};
use oci_api::{async_trait, Result};
use std::sync::{Arc, Mutex};

// Create a mock implementation for testing
struct MockEmailSender {
    sent: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl EmailSender for MockEmailSender {
    async fn send(&self, email: Email) -> Result<SubmitEmailResponse> {
        self.sent.lock().unwrap().push(email.subject.clone());
        Ok(SubmitEmailResponse {
            message_id: "mock-id".to_owned(),
            envelope_id: "mock-env".to_owned(),
            suppressed_recipients: None,
        })
    }
}

// Use trait object for dependency injection
async fn send_welcome(sender: &dyn EmailSender, email: Email) -> Result<SubmitEmailResponse> {
    sender.send(email).await
}
```

This pattern lets you:
- **Production**: Use `Arc<dyn EmailSender>` with `EmailDelivery` (real OCI API)
- **Test**: Use `Arc<dyn EmailSender>` with a mock (no network calls, verify sent emails)

For OCI Email Delivery documentation, see:
- [OCI Email Delivery Overview](https://docs.oracle.com/en-us/iaas/Content/Email/home.htm)
- [OCI Email Delivery API Reference](https://docs.oracle.com/en-us/iaas/api/#/en/emaildelivery/20170907/)
- [OCI Email Delivery Submission API Reference](https://docs.oracle.com/en-us/iaas/api/#/en/emaildeliverysubmission/20220926/)

<br>

## Object Storage API

```rust
use oci_api::Oci;
use oci_api::object_storage::ObjectStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an object storage instance
    let oci_client = Oci::from_env()?;
    let storage = ObjectStorage::new(&oci_client, "your_namespace");
    // or chaining from Oci directly
    let storage = Oci::from_env()?.object_storage("your_namespace");

    // Get Bucket
    let bucket = storage.get_bucket("your-bucket-name").await?;

    // Put Object
    let object_name = "test-object.txt";
    let value = "Hello, OCI Object Storage!";
    let object = bucket.put_object(object_name, value).await?;

    // Put Object with Checksum (Optional)
    use oci_api::services::object_storage::models::ChecksumAlgorithm;
    let object = bucket.put_object_with_checksum(
        object_name, 
        value, 
        ChecksumAlgorithm::SHA256
    ).await?;

    // Get Object
    let object = bucket.get_object(object_name).await?;

    // Get or Create Object(if not exists)
    let object = bucket.get_or_create_object(object_name, value).await?;
}
```

you can also work with retention rules for a bucket

```rust
use oci_api::services::object_storage::models::{RetentionRuleDetails, RetentionDuration, RetentionTimeUnit};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bucket = Oci::from_env()?
        .object_storage("your_namespace")
        .get_bucket("your-bucket-name")
        .await?;

    // Create a Retention Rule
    let details = RetentionRuleDetails {
        display_name: Some("My Rule".to_string()),
        duration: Some(RetentionDuration {
            time_amount: 30,
            time_unit: RetentionTimeUnit::Days,
        }),
        time_rule_locked: None,
    };
    let rule = bucket.create_retention_rule(details).await?;

    // Get Retention Rules Vector
    let rules = bucket.get_retention_rules().await?;

    // Get Retention Rule by ID
    let rule = bucket.get_retention_rule(&rule.id).await?;

    // Update Retention Rule
    let update_details = RetentionRuleDetails {
        display_name: Some("My Rule Updated".to_string()),
        ..Default::default()
    };
    let updated_rule = bucket.update_retention_rule(&rule, update_details).await?;

    // Delete Retention Rule
    bucket.delete_retention_rule(&rule).await?;
    
    Ok(())
}
```

### Object Integrity

It automatically maps available checksum headers into `md5`(`Content-MD5`)
and `checksum`(`opc-content-sha256`|`opc-content-sha384`|`opc-content-crc32c`) fields.

 You can verify the integrity of the downloaded object using the `verify_checksums()` method.

```rust
use oci_api::services::object_storage::models::ChecksumAlgorithm;

let object = bucket.get_object("my-object").await?;

// Verify integrity against all available checksums
// Returns Ok(()) if all present checksums match, or an Error if any mismatch
object.verify_checksums()?;

// Access specific checksums
println!("MD5: {}", object.md5);

if let Some(checksum) = &object.checksum {
    match checksum.algorithm {
        ChecksumAlgorithm::SHA256 => println!("SHA256: {}", checksum.value),
        ChecksumAlgorithm::SHA384 => println!("SHA384: {}", checksum.value),
        ChecksumAlgorithm::CRC32C => println!("CRC32C: {}", checksum.value),
    }
}
```

For OCI Object Storage documentation, see:
- [OCI Object Storage Overview](https://docs.oracle.com/en-us/iaas/Content/Object/Concepts/objectstorageoverview.htm)
- [OCI Object Storage API Reference](https://docs.oracle.com/en-us/iaas/api/#/en/objectstorage/20160918/)

<br>

## Vault Secrets API

```rust
use oci_api::Oci;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oci = Oci::from_env()?;
    let vault = oci.vault();

    let current = vault.get_secret_bundle("ocid1.vaultsecret.oc1..example").await?;
    let current_value = current.secret_bundle_content.decoded_string()?;

    let pending = vault
        .get_secret_bundle_by_stage("ocid1.vaultsecret.oc1..example", "PENDING")
        .await?;

    let previous = vault
        .get_secret_bundle_by_version("ocid1.vaultsecret.oc1..example", 3)
        .await?;

    println!("current secret: {current_value}");
    println!("pending stages: {:?}", pending.stages);
    println!("previous version: {:?}", previous.version_number);
    Ok(())
}
```

Phase 1 scope intentionally focuses on:

- current secret bundle lookup
- staged secret bundle lookup
- versioned secret bundle lookup

## Keys API

```rust
use oci_api::Oci;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oci = Oci::from_env()?;

    // Use the KMS management endpoint for the target vault.
    let keys = oci.keys("management.kms.ap-seoul-1.oci.oraclecloud.com");

    let key = keys.get_key("ocid1.key.oc1.ap-seoul-1.example").await?;
    let rotated = keys.rotate_key("ocid1.key.oc1.ap-seoul-1.example").await?;

    println!("key: {}", key.id);
    println!("rotated version: {:?}", rotated.current_key_version);
    Ok(())
}
```

Phase 1 scope intentionally focuses on:

- key lookup
- rotate action

Coolify or other orchestration layers should manage test-runner/container lifecycle only. Secret and key operations should still go through OCI-authenticated API calls via `oci-api`.

<br>

## Smoke Runner Container

This repository also includes a temporary smoke-runner container entrypoint for OCI-hosted validation:

- binary: `smoke_instance_principal`
- container build: `Dockerfile`

Expected environment variables:

- `OCI_AUTH_MODE=instance_principal`
- `OCI_METADATA_BASE_URL` (optional override for local/mock tests)
- `OCI_SMOKE_SECRET_ID` (optional if key smoke is configured)
- `OCI_SMOKE_SECRET_STAGE` or `OCI_SMOKE_SECRET_VERSION` (optional)
- `OCI_SMOKE_KMS_MANAGEMENT_ENDPOINT` + `OCI_SMOKE_KEY_ID` (optional pair)
- `OCI_SMOKE_ROTATE_KEY=true|false` (optional, defaults to `false`)
- `OCI_SMOKE_KEEP_ALIVE=true|false` (optional, defaults to `true`)

The runner never prints secret values; it only reports metadata such as version, stages, content length, key lifecycle state, and current key version.

<br>

## Error Handling

The library provides comprehensive error types:

```rust
use oci_api::{Error, Result};

match email_client.send(email).await {
    Ok(response) => println!("Sent: {}", response.message_id),
    Err(Error::ApiError(status, body)) => {
        eprintln!("API error {}: {}", status, body);
    }
    Err(Error::AuthError(msg)) => {
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
You can request any OCI APIs, and I will try to implement them as soon as possible.
