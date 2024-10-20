pub mod models;

use serde_json::json;

use crate::{
    errors::RobloxApiResult,
    helpers::{handle, handle_as_json},
    models::AssetId,
    RobloxApi,
};

use self::models::ListGroupRolesResponse;

impl RobloxApi {
    /// * `role_id` - Not the same as rank, must be retrieved using [`RobloxApi::list_group_roles`]
    pub async fn update_user_group_role(
        &self,
        group_id: AssetId,
        user_id: AssetId,
        role_id: u64,
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .patch(format!(
                        "https://groups.roblox.com/v1/groups/{}/users/{}",
                        group_id, user_id
                    ))
                    .json(&json!({ "roleId": role_id })))
            })
            .await;

        handle(res).await?;

        Ok(())
    }

    pub async fn list_group_roles(
        &self,
        group_id: AssetId,
    ) -> RobloxApiResult<ListGroupRolesResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.get(format!(
                    "https://groups.roblox.com/v1/groups/{}/roles",
                    group_id
                )))
            })
            .await;

        handle_as_json(res).await
    }
}
