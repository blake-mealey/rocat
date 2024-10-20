use crate::{errors::RobloxApiResult, helpers::handle, models::AssetId, RobloxApi};

use self::models::GrantAssetPermissionsRequest;

pub mod models;

impl RobloxApi {
    pub async fn grant_asset_permissions<R>(
        &self,
        asset_id: AssetId,
        request: R,
    ) -> RobloxApiResult<()>
    where
        R: Into<GrantAssetPermissionsRequest> + Clone,
    {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .patch(format!(
                        "https://apis.roblox.com/asset-permissions-api/v1/assets/{}/permissions",
                        asset_id
                    ))
                    .json(&request.clone().into()))
            })
            .await;

        handle(res).await?;

        Ok(())
    }
}
