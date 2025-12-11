use freedom_models::gateway_licenses;

use crate::{Api, error::Error};

/// Extension API for interacting with the Freedom Gateway licensing architecture
pub trait GatewayApi: Api {
    fn get_all_gateway_licenses(
        &self,
    ) -> impl Future<Output = Result<gateway_licenses::View, Error>> + Send + Sync {
        let uri = self.path_to_url("gateway-licenses");
        self.get_json_map(uri)
    }

    fn get_all_gateway_license(
        &self,
        id: u32,
    ) -> impl Future<Output = Result<gateway_licenses::ViewOne, Error>> + Send + Sync {
        let uri = self.path_to_url(format!("gateway-licenses/{id}"));
        self.get_json_map(uri)
    }

    fn verify_gateway_license(
        &self,
        request: gateway_licenses::Verify,
    ) -> impl Future<Output = Result<gateway_licenses::VerifyResponse, Error>> + Send + Sync {
        let uri = self.path_to_url("gateway-licenses/verify");
        self.post_deserialize(uri, request)
    }

    fn regenerate_gateway_license(
        &self,
        id: u32,
    ) -> impl Future<Output = Result<gateway_licenses::RegenerateResponse, Error>> + Send + Sync
    {
        let uri = self.path_to_url(format!("gateway-licenses/{id}/regenerate"));
        self.post_deserialize(uri, serde_json::json!({}))
    }
}

impl<T> GatewayApi for T where T: Api {}
