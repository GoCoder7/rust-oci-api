//! Object Storage integration tests
//!
//! These tests require actual OCI credentials and will make real API calls.
//! They are ignored by default and must be explicitly run with:
//! ```
//! cargo test --test object_storage_integration_test -- --ignored
//! ```
//!
//! Required authentication:
//! - `OCI_AUTH_MODE=api_key` with `OCI_USER_ID`, `OCI_TENANCY_ID`, `OCI_REGION`,
//!   `OCI_FINGERPRINT`, `OCI_PRIVATE_KEY`
//! - or `OCI_AUTH_MODE=instance_principal` on an OCI runtime where IMDS is reachable
//! - or leave `OCI_AUTH_MODE` unset to autodetect instance principal on OCI and fall back to API key elsewhere
//!
//! Additional required environment variables:
//! - TEST_NAMESPACE (Object Storage Namespace)
//! - TEST_BUCKET (Bucket name to use for testing)

use oci_api::client::{AuthMode, Oci};
use oci_api::services::object_storage::ObjectStorage;
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

/// Load .env file if it exists
fn load_env() {
    let _ = dotenvy::dotenv();
}

/// Helper to check if OCI credentials are configured
fn has_oci_credentials() -> bool {
    load_env();
    let has_auth = match Oci::resolve_auth_mode_from_env() {
        Ok(AuthMode::InstancePrincipal) => is_imds_reachable(),
        Ok(AuthMode::ApiKey) => {
            std::env::var("OCI_USER_ID").is_ok()
                && std::env::var("OCI_TENANCY_ID").is_ok()
                && std::env::var("OCI_REGION").is_ok()
                && std::env::var("OCI_FINGERPRINT").is_ok()
                && std::env::var("OCI_PRIVATE_KEY").is_ok()
        }
        Err(_) => false,
    };

    has_auth && std::env::var("TEST_NAMESPACE").is_ok() && std::env::var("TEST_BUCKET").is_ok()
}

fn is_imds_reachable() -> bool {
    let address: SocketAddr = "169.254.169.254:80".parse().unwrap();
    TcpStream::connect_timeout(&address, Duration::from_millis(500)).is_ok()
}

#[tokio::test]
#[ignore]
async fn test_get_bucket() {
    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials or Object Storage config not configured");
        return;
    }

    let oci_client = Oci::from_env().expect("Failed to load OCI config");

    let namespace = std::env::var("TEST_NAMESPACE").unwrap();
    let bucket_name = std::env::var("TEST_BUCKET").unwrap();

    let os_client = ObjectStorage::new(&oci_client, namespace);
    let bucket = os_client.get_bucket(&bucket_name).await;

    assert!(bucket.is_ok(), "Failed to get bucket: {:?}", bucket.err());
    let bucket = bucket.unwrap();
    assert_eq!(bucket.name, bucket_name);
}

#[tokio::test]
#[ignore]
async fn test_put_and_get_object() {
    if !has_oci_credentials() {
        eprintln!("Skipping test: OCI credentials or Object Storage config not configured");
        return;
    }

    let oci_client = Oci::from_env().expect("Failed to load OCI config");

    let namespace = std::env::var("TEST_NAMESPACE").unwrap();
    let bucket_name = std::env::var("TEST_BUCKET").unwrap();

    let os_client = ObjectStorage::new(&oci_client, namespace);
    let bucket = os_client
        .get_bucket(&bucket_name)
        .await
        .expect("Failed to get bucket");

    let object_name = "test_object.bin";
    let content = vec![0x00_u8, 0x80, 0xFF, b'O', b'K'];

    // Put Object
    let put_result = bucket.put_object(object_name, content.clone()).await;
    assert!(
        put_result.is_ok(),
        "Failed to put object: {:?}",
        put_result.err()
    );
    let object = put_result.unwrap();
    assert_eq!(object.name, object_name);
    assert_eq!(object.value.as_ref(), content.as_slice());

    // Get Object
    let get_result = bucket.get_object(object_name).await;
    assert!(
        get_result.is_ok(),
        "Failed to get object: {:?}",
        get_result.err()
    );
    let object = get_result.unwrap();
    assert_eq!(object.name, object_name);
    assert_eq!(object.value.as_ref(), content.as_slice());

    let delete_result = bucket.delete_object(object_name).await;
    assert!(
        delete_result.is_ok(),
        "Failed to delete object: {:?}",
        delete_result.err()
    );
}
