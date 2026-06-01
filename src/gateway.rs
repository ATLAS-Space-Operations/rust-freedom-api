use freedom_models::gateway;

use crate::{Api, error::Error};

/// Extension API for interacting with the Freedom Gateway licensing architecture
pub trait GatewayApi: Api {
    /// Fetch information about the latest version of Freedom Gateway
    fn get_gateway_latest_version(
        &self,
    ) -> impl Future<Output = Result<gateway::LatestVersion, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url("gateway/latestVersion")?;
            self.get_json_map(uri).await
        }
    }

    /// Fetch full list of gateway licenses
    fn get_gateway_licenses(
        &self,
    ) -> impl Future<Output = Result<gateway::View, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url("gateway-licenses")?;
            self.get_json_map(uri).await
        }
    }

    /// Fetch single gateway license by ID
    fn get_gateway_license_by_id(
        &self,
        id: u32,
    ) -> impl Future<Output = Result<gateway::ViewOne, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("gateway-licenses/{id}"))?;
            self.get_json_map(uri).await
        }
    }

    /// Regenerate a license by ID. This will create a new license key, and invalidate the previous
    /// license key associated with the gateway license ID
    fn regenerate_gateway_license(
        &self,
        id: u32,
    ) -> impl Future<Output = Result<gateway::RegenerateResponse, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("gateway-licenses/{id}/regenerate"))?;
            self.post_json_map(uri, serde_json::json!({})).await
        }
    }

    /// Checks whether the provided license is valid
    fn verify_gateway_license(
        &self,
        license_key: &str,
    ) -> impl Future<Output = Result<gateway::VerifyResponse, Error>> + Send + Sync {
        #[derive(Debug, serde::Serialize)]
        #[serde(rename_all = "camelCase")]
        struct Verify {
            license_key: String,
        }

        async move {
            let request = Verify {
                license_key: license_key.to_string(),
            };
            let uri = self.path_to_url("gateway-licenses/verify")?;
            self.post_json_map(uri, request).await
        }
    }
}

impl<T> GatewayApi for T where T: Api {}
