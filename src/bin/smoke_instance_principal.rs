use std::env;
use std::error::Error;
use std::time::Duration;

use oci_api::Oci;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let oci = Oci::from_env()?;

    println!("starting OCI smoke runner");
    println!("auth mode: {:?}", oci.auth_mode());
    println!("region: {}", oci.region());
    println!("tenancy: {}", oci.tenancy_id());

    let secret_id = env::var("OCI_SMOKE_SECRET_ID").ok();
    let secret_stage = env::var("OCI_SMOKE_SECRET_STAGE").ok();
    let secret_version = env::var("OCI_SMOKE_SECRET_VERSION")
        .ok()
        .map(|value| value.parse::<i64>())
        .transpose()?;
    let kms_endpoint = env::var("OCI_SMOKE_KMS_MANAGEMENT_ENDPOINT").ok();
    let key_id = env::var("OCI_SMOKE_KEY_ID").ok();
    let rotate_key = env::var("OCI_SMOKE_ROTATE_KEY")
        .map(|value| value == "true")
        .unwrap_or(false);
    let keep_alive = env::var("OCI_SMOKE_KEEP_ALIVE")
        .map(|value| value != "false")
        .unwrap_or(true);

    if secret_id.is_none() && (kms_endpoint.is_none() || key_id.is_none()) {
        return Err(
            "set OCI_SMOKE_SECRET_ID and/or OCI_SMOKE_KMS_MANAGEMENT_ENDPOINT + OCI_SMOKE_KEY_ID"
                .into(),
        );
    }

    if let Some(secret_id) = secret_id.as_deref() {
        let vault = oci.vault();
        let bundle = match (secret_stage.as_deref(), secret_version) {
            (_, Some(version_number)) => {
                println!("reading versioned secret bundle");
                vault
                    .get_secret_bundle_by_version(secret_id, version_number)
                    .await?
            }
            (Some(stage), None) => {
                println!("reading staged secret bundle");
                vault.get_secret_bundle_by_stage(secret_id, stage).await?
            }
            (None, None) => {
                println!("reading current secret bundle");
                vault.get_secret_bundle(secret_id).await?
            }
        };

        let decoded = bundle.secret_bundle_content.decoded_bytes()?;
        println!("secret bundle read ok");
        println!("secret version: {:?}", bundle.version_number);
        println!("secret stages: {:?}", bundle.stages);
        println!(
            "secret content type: {}",
            bundle.secret_bundle_content.content_type
        );
        println!("secret content length: {}", decoded.len());
    }

    if let (Some(kms_endpoint), Some(key_id)) = (kms_endpoint.as_deref(), key_id.as_deref()) {
        let keys = oci.keys(kms_endpoint);
        let key = keys.get_key(key_id).await?;
        println!("key lookup ok");
        println!("key id: {}", key.id);
        println!("key lifecycle state: {:?}", key.lifecycle_state);
        println!("current key version: {:?}", key.current_key_version);

        if rotate_key {
            let rotated = keys.rotate_key(key_id).await?;
            println!("key rotation ok");
            println!("rotated key version: {:?}", rotated.current_key_version);
        }
    }

    println!("OCI smoke runner finished successfully");

    if keep_alive {
        println!("keeping container alive for log inspection");
        loop {
            sleep(Duration::from_secs(300)).await;
        }
    }

    Ok(())
}
