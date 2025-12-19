use crate::gcp_authorizer_error::GcpAuthorizerError;
use async_trait::async_trait;
// use hyper::Uri;
use std::time::Duration;
use tonic::transport::{Channel, Uri};
pub struct GoogleEnvironment;

impl GoogleEnvironment {
    pub async fn init_google_services_channel<S: AsRef<str>>(api_url: S) -> Result<Channel, crate::error::Error> {
        let api_url_string = api_url.as_ref().to_string();
        let uri = Uri::from_maybe_shared(api_url_string)?;
        if uri.authority().is_none() {
            return Err(crate::error::ErrorKind::UrlErrorInvalidAuthority("domain is required".to_string()).into());
        }
        let domain_name = uri.authority().unwrap().host().to_string();
        let tls_config = Self::init_tls_config(domain_name);

        Ok(Channel::builder(uri)
            .tls_config(tls_config)?
            .connect_timeout(Duration::from_secs(30))
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .keep_alive_timeout(Duration::from_secs(60))
            .http2_keep_alive_interval(Duration::from_secs(60))
            .keep_alive_while_idle(true)
            .connect()
            .await?)
    }

    fn init_tls_config(domain_name: String) -> tonic::transport::ClientTlsConfig {
        tonic::transport::ClientTlsConfig::new()
            .with_native_roots()
            .domain_name(domain_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_normalize_label_key() {
        let _channel = GoogleEnvironment::init_google_services_channel("https://monitoring.googleapis.com")
            .await
            .unwrap();
    }
}

pub struct FakeAuthorizer;

impl FakeAuthorizer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Authorizer for FakeAuthorizer {
    fn project_id(&self) -> &str {
        "fake_project_id"
    }

    async fn token(&self) -> Result<String, GcpAuthorizerError> {
        // req.metadata_mut().insert(
        //     "authorization",
        //     MetadataValue::try_from(format!("Bearer {}", "fake_token")).unwrap(),
        // );
        Ok("fake_token".to_string())
    }
}

#[async_trait]
pub trait Authorizer {
    fn project_id(&self) -> &str;
    async fn token(&self) -> Result<String, GcpAuthorizerError>;
}
