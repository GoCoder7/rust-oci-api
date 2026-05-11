use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use base64::{
    Engine as _, engine::general_purpose::STANDARD, engine::general_purpose::URL_SAFE_NO_PAD,
};
use reqwest::Client;
use rsa::RsaPrivateKey;
use rsa::pkcs8::{DecodePrivateKey, EncodePrivateKey, EncodePublicKey, LineEnding};
use rsa::rand_core::OsRng;
use serde::Deserialize;
use sha1::Sha1;
use sha2::Digest;
use tokio::sync::Mutex;

use crate::auth::providers::{OciAuthProvider, SignRequest, SignedHeaders};
use crate::client::signer::OciSigner;
use crate::error::{Error, Result};

const DEFAULT_METADATA_BASE_URL: &str = "http://169.254.169.254/opc/v2";
const METADATA_AUTHORIZATION: &str = "Bearer Oracle";
const REGION_PATH: &str = "/instance/region";
const LEAF_CERTIFICATE_PATH: &str = "/identity/cert.pem";
const LEAF_PRIVATE_KEY_PATH: &str = "/identity/key.pem";
const INTERMEDIATE_CERTIFICATE_PATH: &str = "/identity/intermediate.pem";
const DEFAULT_REFRESH_WINDOW_SECS: u64 = 300;

#[derive(Debug, Clone)]
pub struct InstancePrincipalConfig {
    pub region: String,
    pub tenancy_id: String,
    pub metadata_base_url: String,
    pub refresh_window: Duration,
    pub auth_scheme: String,
    pub auth_host_override: Option<String>,
}

impl InstancePrincipalConfig {
    pub fn new(region: impl Into<String>, tenancy_id: impl Into<String>) -> Self {
        Self {
            region: region.into(),
            tenancy_id: tenancy_id.into(),
            metadata_base_url: DEFAULT_METADATA_BASE_URL.to_owned(),
            refresh_window: Duration::from_secs(DEFAULT_REFRESH_WINDOW_SECS),
            auth_scheme: "https".to_owned(),
            auth_host_override: None,
        }
    }

    pub fn metadata_base_url(mut self, metadata_base_url: impl Into<String>) -> Self {
        self.metadata_base_url = metadata_base_url.into();
        self
    }

    pub fn refresh_window(mut self, refresh_window: Duration) -> Self {
        self.refresh_window = refresh_window;
        self
    }

    pub fn auth_scheme(mut self, auth_scheme: impl Into<String>) -> Self {
        self.auth_scheme = auth_scheme.into();
        self
    }

    pub fn auth_host_override(mut self, auth_host_override: impl Into<String>) -> Self {
        self.auth_host_override = Some(auth_host_override.into());
        self
    }
}

#[derive(Clone)]
pub struct InstancePrincipalAuthProvider {
    client: Client,
    config: InstancePrincipalConfig,
    state: Arc<Mutex<Option<InstancePrincipalState>>>,
}

struct InstancePrincipalState {
    signer: OciSigner,
    expires_at: SystemTime,
}

#[derive(Deserialize)]
struct FederationResponse {
    token: String,
}

impl InstancePrincipalAuthProvider {
    pub fn new(client: Client, config: InstancePrincipalConfig) -> Self {
        Self {
            client,
            config,
            state: Arc::new(Mutex::new(None)),
        }
    }

    async fn ensure_state(&self) -> Result<OciSigner> {
        let mut guard = self.state.lock().await;

        if let Some(state) = guard.as_ref() {
            let refresh_at = state
                .expires_at
                .checked_sub(self.config.refresh_window)
                .unwrap_or(UNIX_EPOCH);
            if SystemTime::now() < refresh_at {
                return Ok(state.signer.clone());
            }
        }

        let state = self.refresh_state().await?;
        let signer = state.signer.clone();
        *guard = Some(state);
        Ok(signer)
    }

    async fn refresh_state(&self) -> Result<InstancePrincipalState> {
        let metadata = self.fetch_metadata_materials().await?;
        let session_private_key_pem = new_session_private_key_pem()?;
        let session_public_key = session_public_key_pem(&session_private_key_pem)?;

        let auth_key_id = format!(
            "{}/fed-x509/{}",
            self.config.tenancy_id,
            certificate_fingerprint(&metadata.leaf_certificate)?
        );
        let auth_signer = OciSigner::new_with_key_id(auth_key_id, &metadata.leaf_private_key)?;

        let request_body = serde_json::json!({
            "certificate": sanitize_pem_body(&metadata.leaf_certificate),
            "publicKey": sanitize_pem_body(&session_public_key),
            "intermediateCertificates": [sanitize_pem_body(&metadata.intermediate_certificate)],
        });
        let body_json = serde_json::to_string(&request_body)?;
        let path = "/v1/x509";
        let host = self.auth_host();
        let signed = auth_signer.sign_request_headers(
            "POST",
            path,
            None,
            Some(&body_json),
            Some("application/json"),
            None,
        )?;

        let response = self
            .client
            .post(format!("{}://{host}{path}", self.config.auth_scheme))
            .header("date", &signed.date)
            .header("authorization", &signed.authorization)
            .header(
                "content-type",
                signed
                    .content_type
                    .unwrap_or_else(|| "application/json".to_owned()),
            )
            .header(
                "content-length",
                signed
                    .content_length
                    .unwrap_or_else(|| body_json.len().to_string()),
            )
            .header(
                "x-content-sha256",
                signed
                    .x_content_sha256
                    .ok_or_else(|| Error::AuthError("Missing x-content-sha256".to_owned()))?,
            )
            .body(body_json)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(Error::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        let FederationResponse { token } = response.json().await?;
        let expires_at = jwt_expiration(&token)?;
        let session_signer =
            OciSigner::new_with_key_id(format!("ST${token}"), &session_private_key_pem)?;

        Ok(InstancePrincipalState {
            signer: session_signer,
            expires_at,
        })
    }

    async fn fetch_metadata_materials(&self) -> Result<MetadataMaterials> {
        let leaf_certificate = self.fetch_metadata_text(LEAF_CERTIFICATE_PATH).await?;
        let leaf_private_key = self.fetch_metadata_text(LEAF_PRIVATE_KEY_PATH).await?;
        let intermediate_certificate = self
            .fetch_metadata_text(INTERMEDIATE_CERTIFICATE_PATH)
            .await?;

        Ok(MetadataMaterials {
            leaf_certificate,
            leaf_private_key,
            intermediate_certificate,
        })
    }

    async fn fetch_metadata_text(&self, path: &str) -> Result<String> {
        let response = self
            .client
            .get(format!("{}{}", self.config.metadata_base_url, path))
            .header("authorization", METADATA_AUTHORIZATION)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(Error::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        response.text().await.map_err(Into::into)
    }

    pub async fn metadata_region(client: &Client, metadata_base_url: &str) -> Result<String> {
        let response = client
            .get(format!("{metadata_base_url}{REGION_PATH}"))
            .header("authorization", METADATA_AUTHORIZATION)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(Error::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        response.text().await.map_err(Into::into)
    }

    fn auth_host(&self) -> String {
        self.config
            .auth_host_override
            .clone()
            .unwrap_or_else(|| format!("auth.{}.oraclecloud.com", self.config.region))
    }
}

#[async_trait]
impl OciAuthProvider for InstancePrincipalAuthProvider {
    async fn sign(&self, request: &SignRequest<'_>) -> Result<SignedHeaders> {
        let signer = self.ensure_state().await?;
        let signed = signer.sign_request_headers(
            request.method,
            request.path,
            request.host,
            request.body,
            request.content_type,
            None,
        )?;

        Ok(SignedHeaders {
            date: signed.date,
            authorization: signed.authorization,
            content_type: signed.content_type,
            content_length: signed.content_length,
            x_content_sha256: signed.x_content_sha256,
            extra_headers: Vec::new(),
        })
    }
}

struct MetadataMaterials {
    leaf_certificate: String,
    leaf_private_key: String,
    intermediate_certificate: String,
}

fn sanitize_pem_body(value: &str) -> String {
    value
        .replace("-----BEGIN CERTIFICATE-----", "")
        .replace("-----END CERTIFICATE-----", "")
        .replace("-----BEGIN PUBLIC KEY-----", "")
        .replace("-----END PUBLIC KEY-----", "")
        .replace('\n', "")
}

fn certificate_fingerprint(certificate_pem: &str) -> Result<String> {
    let der_body = sanitize_pem_body(certificate_pem);
    let der = STANDARD
        .decode(der_body)
        .map_err(|e| Error::AuthError(format!("Failed to decode certificate: {e}")))?;
    let digest = Sha1::digest(der);
    Ok(digest
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(":"))
}

fn new_session_private_key_pem() -> Result<String> {
    let private_key = RsaPrivateKey::new(&mut OsRng, 2048)
        .map_err(|e| Error::AuthError(format!("Failed to generate session key: {e}")))?;
    private_key
        .to_pkcs8_pem(LineEnding::LF)
        .map(|pem| pem.to_string())
        .map_err(|e| Error::AuthError(format!("Failed to encode session key: {e}")))
}

fn session_public_key_pem(private_key_pem: &str) -> Result<String> {
    let private_key = RsaPrivateKey::from_pkcs8_pem(private_key_pem)
        .map_err(|e| Error::AuthError(format!("Failed to parse session key: {e}")))?;
    private_key
        .to_public_key()
        .to_public_key_pem(LineEnding::LF)
        .map_err(|e| Error::AuthError(format!("Failed to encode session public key: {e}")))
}

fn jwt_expiration(token: &str) -> Result<SystemTime> {
    let payload = token
        .split('.')
        .nth(1)
        .ok_or_else(|| Error::AuthError("Security token payload is missing".to_owned()))?;
    let decoded = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|e| Error::AuthError(format!("Failed to decode security token: {e}")))?;
    let value: serde_json::Value = serde_json::from_slice(&decoded)?;
    let exp = value
        .get("exp")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| Error::AuthError("Security token exp claim is missing".to_owned()))?;
    Ok(UNIX_EPOCH + Duration::from_secs(exp))
}

#[cfg(test)]
mod tests {
    use super::*;

    use mockito::{Matcher, Server};
    use rsa::pkcs8::EncodePrivateKey;

    fn test_private_key_pem() -> String {
        RsaPrivateKey::new(&mut OsRng, 2048)
            .unwrap()
            .to_pkcs8_pem(LineEnding::LF)
            .unwrap()
            .to_string()
    }

    fn test_certificate_pem(label: &str) -> String {
        format!(
            "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
            STANDARD.encode(label.as_bytes())
        )
    }

    fn test_jwt(exp: u64) -> String {
        let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"RS256","typ":"JWT"}"#);
        let payload =
            URL_SAFE_NO_PAD.encode(format!(r#"{{"exp":{exp},"sub":"instance-principal"}}"#));
        format!("{header}.{payload}.signature")
    }

    fn strip_http_scheme(url: &str) -> String {
        url.trim_start_matches("http://").to_owned()
    }

    #[tokio::test]
    async fn test_metadata_region_fetches_imds_value() {
        let mut server = Server::new_async().await;
        let _mock = server
            .mock("GET", "/opc/v2/instance/region")
            .match_header("authorization", METADATA_AUTHORIZATION)
            .with_status(200)
            .with_body("ap-seoul-1")
            .create_async()
            .await;

        let region = InstancePrincipalAuthProvider::metadata_region(
            &Client::new(),
            &format!("{}/opc/v2", server.url()),
        )
        .await
        .unwrap();

        assert_eq!(region, "ap-seoul-1");
    }

    #[tokio::test]
    async fn test_sign_fetches_and_reuses_security_token() {
        let mut metadata_server = Server::new_async().await;
        let leaf_private_key = test_private_key_pem();
        let leaf_certificate = test_certificate_pem("leaf");
        let intermediate_certificate = test_certificate_pem("intermediate");

        let _leaf_cert = metadata_server
            .mock("GET", "/opc/v2/identity/cert.pem")
            .match_header("authorization", METADATA_AUTHORIZATION)
            .expect(1)
            .with_status(200)
            .with_body(leaf_certificate.clone())
            .create_async()
            .await;
        let _leaf_key = metadata_server
            .mock("GET", "/opc/v2/identity/key.pem")
            .match_header("authorization", METADATA_AUTHORIZATION)
            .expect(1)
            .with_status(200)
            .with_body(leaf_private_key.clone())
            .create_async()
            .await;
        let _intermediate = metadata_server
            .mock("GET", "/opc/v2/identity/intermediate.pem")
            .match_header("authorization", METADATA_AUTHORIZATION)
            .expect(1)
            .with_status(200)
            .with_body(intermediate_certificate.clone())
            .create_async()
            .await;

        let exp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;
        let token = test_jwt(exp);

        let mut auth_server = Server::new_async().await;
        let auth_host = strip_http_scheme(&auth_server.url());
        let _auth = auth_server
            .mock("POST", "/v1/x509")
            .match_header(
                "content-type",
                Matcher::Regex("application/json".to_owned()),
            )
            .match_header(
                "authorization",
                Matcher::Regex(r#"keyId="ocid1\.tenancy\.oc1\.\.example/fed-x509/"#.to_owned()),
            )
            .expect(1)
            .with_status(200)
            .with_body(format!(r#"{{"token":"{token}"}}"#))
            .create_async()
            .await;

        let provider = InstancePrincipalAuthProvider::new(
            Client::new(),
            InstancePrincipalConfig::new("ap-seoul-1", "ocid1.tenancy.oc1..example")
                .metadata_base_url(format!("{}/opc/v2", metadata_server.url()))
                .auth_scheme("http")
                .auth_host_override(auth_host)
                .refresh_window(Duration::from_secs(60)),
        );

        let request = SignRequest {
            method: "GET",
            path: "/n/test_namespace/b/test_bucket/o/test_object",
            host: Some("objectstorage.ap-seoul-1.oraclecloud.com"),
            body: None,
            content_type: None,
        };

        let first = provider.sign(&request).await.unwrap();
        let second = provider.sign(&request).await.unwrap();

        assert!(first.authorization.contains("keyId=\"ST$"));
        assert_eq!(first.authorization, second.authorization);
    }
}
