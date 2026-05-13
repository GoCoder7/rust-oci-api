use std::env;
use std::error::Error;

use oci_api::Oci;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

struct SmokeTargets<'a> {
    secret_id: Option<&'a str>,
    secret_stage: Option<&'a str>,
    secret_version: Option<i64>,
    kms_endpoint: Option<&'a str>,
    key_id: Option<&'a str>,
    email_check: bool,
    email_compartment_id: Option<&'a str>,
    object_storage_namespace: Option<&'a str>,
    object_storage_bucket: Option<&'a str>,
    rotate_key: bool,
}

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
    let email_check = env::var("OCI_SMOKE_EMAIL_CHECK")
        .map(|value| value == "true")
        .unwrap_or(false);
    let email_compartment_id = env::var("OCI_SMOKE_EMAIL_COMPARTMENT_ID").ok();
    let object_storage_namespace = env::var("OCI_SMOKE_OS_NAMESPACE").ok();
    let object_storage_bucket = env::var("OCI_SMOKE_OS_BUCKET").ok();
    let rotate_key = env::var("OCI_SMOKE_ROTATE_KEY")
        .map(|value| value == "true")
        .unwrap_or(false);
    let keep_alive = env::var("OCI_SMOKE_KEEP_ALIVE")
        .map(|value| value != "false")
        .unwrap_or(true);

    let has_key_target = kms_endpoint.is_some() && key_id.is_some();
    let has_email_target = email_check;
    let has_object_storage_target =
        object_storage_namespace.is_some() && object_storage_bucket.is_some();

    if secret_id.is_none() && !has_key_target && !has_email_target && !has_object_storage_target {
        println!("no smoke targets configured yet");
        if keep_alive {
            return serve_healthcheck("awaiting smoke target configuration", 200).await;
        }
        return Err(
            "set OCI_SMOKE_SECRET_ID, OCI_SMOKE_KMS_MANAGEMENT_ENDPOINT + OCI_SMOKE_KEY_ID, OCI_SMOKE_EMAIL_CHECK=true, and/or OCI_SMOKE_OS_NAMESPACE + OCI_SMOKE_OS_BUCKET"
                .into(),
        );
    }

    let smoke_targets = SmokeTargets {
        secret_id: secret_id.as_deref(),
        secret_stage: secret_stage.as_deref(),
        secret_version,
        kms_endpoint: kms_endpoint.as_deref(),
        key_id: key_id.as_deref(),
        email_check,
        email_compartment_id: email_compartment_id.as_deref(),
        object_storage_namespace: object_storage_namespace.as_deref(),
        object_storage_bucket: object_storage_bucket.as_deref(),
        rotate_key,
    };

    let smoke_result = run_smoke(&oci, smoke_targets).await;

    match smoke_result {
        Ok(()) => {
            println!("OCI smoke runner finished successfully");
            if keep_alive {
                return serve_healthcheck("smoke runner ready", 200).await;
            }
        }
        Err(error) => {
            eprintln!("OCI smoke runner failed: {error}");
            if keep_alive {
                return serve_healthcheck("smoke runner failed", 500).await;
            }
            return Err(error);
        }
    }

    Ok(())
}

async fn run_smoke(oci: &Oci, targets: SmokeTargets<'_>) -> Result<(), Box<dyn Error>> {
    if let Some(secret_id) = targets.secret_id {
        let vault = oci.vault();
        let bundle = match (targets.secret_stage, targets.secret_version) {
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

    if let (Some(kms_endpoint), Some(key_id)) = (targets.kms_endpoint, targets.key_id) {
        let keys = oci.keys(kms_endpoint);
        let key = keys.get_key(key_id).await?;
        println!("key lookup ok");
        println!("key id: {}", key.id);
        println!("key lifecycle state: {:?}", key.lifecycle_state);
        println!("current key version: {:?}", key.current_key_version);

        if targets.rotate_key {
            let rotated = keys.rotate_key(key_id).await?;
            println!("key rotation ok");
            println!("rotated key version: {:?}", rotated.current_key_version);
        }
    }

    if targets.email_check {
        let email_client = oci.email_delivery().await?;
        let compartment_id = targets.email_compartment_id.unwrap_or(oci.compartment_id());
        println!("reading email configuration");
        let configuration = email_client.get_email_configuration(compartment_id).await?;
        println!("email configuration read ok");
        println!(
            "email http submit endpoint: {}",
            configuration.http_submit_endpoint
        );
        println!(
            "email smtp submit endpoint: {}",
            configuration.smtp_submit_endpoint
        );

        let senders = email_client
            .list_senders(compartment_id, Some("ACTIVE"), None)
            .await?;
        println!("email approved sender count: {}", senders.len());
    }

    if let (Some(namespace), Some(bucket_name)) =
        (targets.object_storage_namespace, targets.object_storage_bucket)
    {
        let bucket = oci.object_storage(namespace).get_bucket(bucket_name).await?;
        println!("object storage bucket lookup ok");
        println!("object storage bucket name: {}", bucket.name);
        println!("object storage namespace: {}", bucket.namespace);
    }

    Ok(())
}

async fn serve_healthcheck(status: &str, status_code: u16) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;
    let reason_phrase = match status_code {
        200 => "OK",
        500 => "Internal Server Error",
        _ => "OK",
    };
    println!("health endpoint listening on 0.0.0.0:8080");
    println!("health status: {status}");

    loop {
        let (mut socket, _) = listener.accept().await?;
        let body = format!("{{\"status\":\"{status}\"}}");
        let response = format!(
            "HTTP/1.1 {status_code} {reason_phrase}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body
        );

        let mut buffer = [0_u8; 1024];
        let _ = socket.read(&mut buffer).await;
        socket.write_all(response.as_bytes()).await?;
        socket.shutdown().await?;
    }
}
