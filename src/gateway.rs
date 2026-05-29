use freedom_models::gateway_licenses;

use crate::{Api, error::Error};

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct Verify {
    license_key: String,
}

/// Extension API for interacting with the Freedom Gateway licensing architecture
pub trait GatewayApi: Api {
    /// Fetch information about the latest version of Freedom Gateway
    fn get_gateway_latest_version(
        &self,
    ) -> impl Future<Output = Result<gateway_licenses::LatestVersion, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url("gateway/latestVersion")?;
            self.get_json_map(uri).await
        }
    }

    fn get_all_gateway_licenses(
        &self,
    ) -> impl Future<Output = Result<gateway_licenses::View, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url("gateway-licenses")?;
            self.get_json_map(uri).await
        }
    }

    fn get_gateway_license(
        &self,
        id: u32,
    ) -> impl Future<Output = Result<gateway_licenses::ViewOne, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("gateway-licenses/{id}"))?;
            self.get_json_map(uri).await
        }
    }

    fn regenerate_gateway_license(
        &self,
        id: u32,
    ) -> impl Future<Output = Result<gateway_licenses::RegenerateResponse, Error>> + Send + Sync
    {
        async move {
            let uri = self.path_to_url(format!("gateway-licenses/{id}/regenerate"))?;
            self.post_json_map(uri, serde_json::json!({})).await
        }
    }

    fn verify_gateway_license(
        &self,
        license_key: &str,
    ) -> impl Future<Output = Result<gateway_licenses::VerifyResponse, Error>> + Send + Sync {
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
