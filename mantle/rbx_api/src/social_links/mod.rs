pub mod models;

use serde_json::json;

use crate::{
    errors::RobloxApiResult,
    helpers::{handle, handle_as_json},
    models::AssetId,
    RobloxApi,
};

use self::models::{
    CreateSocialLinkResponse, GetSocialLinkResponse, ListSocialLinksResponse, SocialLinkType,
};

impl RobloxApi {
    pub async fn create_social_link(
        &self,
        experience_id: AssetId,
        title: String,
        url: String,
        link_type: SocialLinkType,
    ) -> RobloxApiResult<CreateSocialLinkResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(format!(
                        "https://develop.roblox.com/v1/universes/{}/social-links",
                        experience_id
                    ))
                    .json(&json!({
                        "title": title,
                        "url": url,
                        "type": link_type,
                    })))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn update_social_link(
        &self,
        experience_id: AssetId,
        social_link_id: AssetId,
        title: String,
        url: String,
        link_type: SocialLinkType,
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .patch(format!(
                        "https://develop.roblox.com/v1/universes/{}/social-links/{}",
                        experience_id, social_link_id
                    ))
                    .json(&json!({
                        "title": title,
                        "url": url,
                        "type": link_type,
                    })))
            })
            .await;

        handle(res).await?;

        Ok(())
    }

    pub async fn delete_social_link(
        &self,
        experience_id: AssetId,
        social_link_id: AssetId,
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.delete(format!(
                    "https://develop.roblox.com/v1/universes/{}/social-links/{}",
                    experience_id, social_link_id
                )))
            })
            .await;

        handle(res).await?;

        Ok(())
    }

    pub async fn list_social_links(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<Vec<GetSocialLinkResponse>> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.get(format!(
                    "https://games.roblox.com/v1/games/{}/social-links/list",
                    experience_id
                )))
            })
            .await;

        Ok(handle_as_json::<ListSocialLinksResponse>(res).await?.data)
    }
}
