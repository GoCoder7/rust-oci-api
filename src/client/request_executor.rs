use reqwest::Method;

use crate::auth::{OciAuthProvider, SignRequest};
use crate::error::{Error, Result};

pub(crate) struct RequestTarget<'a> {
    pub scheme: &'a str,
    pub host: &'a str,
    pub path: &'a str,
}

pub(crate) struct RequestPayload<'a> {
    pub body: Option<String>,
    pub content_type: Option<&'a str>,
    pub extra_headers: Vec<(String, String)>,
}

#[derive(Clone)]
pub(crate) struct RequestExecutor {
    client: reqwest::Client,
    auth_provider: std::sync::Arc<dyn OciAuthProvider>,
}

impl RequestExecutor {
    pub fn new(
        client: reqwest::Client,
        auth_provider: std::sync::Arc<dyn OciAuthProvider>,
    ) -> Self {
        Self {
            client,
            auth_provider,
        }
    }

    pub async fn execute(
        &self,
        method: Method,
        target: RequestTarget<'_>,
        payload: RequestPayload<'_>,
    ) -> Result<reqwest::Response> {
        let sign_request = SignRequest {
            method: method.as_str(),
            path: target.path,
            host: Some(target.host),
            body: payload.body.as_deref(),
            content_type: payload.content_type,
        };
        let signed = self.auth_provider.sign(&sign_request).await?;

        let url = format!("{}://{}{}", target.scheme, target.host, target.path);
        let mut request = self.client.request(method, &url);
        request = request
            .header("host", target.host)
            .header("date", signed.date)
            .header("authorization", signed.authorization);

        if let Some(content_type) = signed.content_type {
            request = request.header("content-type", content_type);
        }
        if let Some(content_length) = signed.content_length {
            request = request.header("content-length", content_length);
        }
        if let Some(body_sha256) = signed.x_content_sha256 {
            request = request.header("x-content-sha256", body_sha256);
        }

        for (name, value) in signed
            .extra_headers
            .into_iter()
            .chain(payload.extra_headers)
        {
            request = request.header(name, value);
        }

        if let Some(body) = payload.body {
            request = request.body(body);
        }

        let response = request.send().await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            return Err(Error::ApiError {
                code: status.to_string(),
                message: body,
            });
        }

        Ok(response)
    }
}
