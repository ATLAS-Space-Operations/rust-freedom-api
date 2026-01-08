use freedom_models::gateway_licenses;

use crate::{Api, error::Error};

/// Extension API for interacting with the Freedom Gateway licensing architecture
pub trait GatewayApi: Api {
    fn get_all_gateway_licenses(
        &self,
    ) -> impl Future<Output = Result<gateway_licenses::View, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url("gateway-licenses")?;
            self.get_json_map(uri).await
        }
    }

    fn get_all_gateway_license(
        &self,
        id: u32,
    ) -> impl Future<Output = Result<gateway_licenses::ViewOne, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url(format!("gateway-licenses/{id}"))?;
            self.get_json_map(uri).await
        }
    }

    fn verify_gateway_license(
        &self,
        request: gateway_licenses::Verify,
    ) -> impl Future<Output = Result<gateway_licenses::VerifyResponse, Error>> + Send + Sync {
        async move {
            let uri = self.path_to_url("gateway-licenses/verify")?;
            self.post_deserialize(uri, request).await
        }
    }

    fn regenerate_gateway_license(
        &self,
        id: u32,
    ) -> impl Future<Output = Result<gateway_licenses::RegenerateResponse, Error>> + Send + Sync
    {
        async move {
            let uri = self.path_to_url(format!("gateway-licenses/{id}/regenerate"))?;
            self.post_deserialize(uri, serde_json::json!({})).await
        }
    }
}

impl<T> GatewayApi for T where T: Api {}
