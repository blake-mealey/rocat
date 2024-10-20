pub mod models;

use std::path::PathBuf;

use reqwest::multipart::Form;
use serde_json::json;

use crate::{
    errors::RobloxApiResult,
    helpers::{get_file_part, handle, handle_as_json},
    models::{AssetId, CreatorType, UploadImageResponse},
    RobloxApi,
};

use self::models::{CreateBadgeResponse, ListBadgeResponse, ListBadgesResponse};

impl RobloxApi {
    pub async fn create_badge(
        &self,
        experience_id: AssetId,
        name: String,
        description: String,
        icon_file_path: PathBuf,
        payment_source: CreatorType,
        expected_cost: u32,
    ) -> RobloxApiResult<CreateBadgeResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(&format!(
                        "https://badges.roblox.com/v1/universes/{}/badges",
                        experience_id
                    ))
                    .multipart(
                        Form::new()
                            .part("request.files", get_file_part(&icon_file_path).await?)
                            .text("request.name", name.clone())
                            .text("request.description", description.clone())
                            .text("request.paymentSourceType", payment_source.to_string())
                            .text("request.expectedCost", expected_cost.to_string()),
                    ))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn update_badge(
        &self,
        badge_id: AssetId,
        name: String,
        description: String,
        enabled: bool,
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .patch(format!("https://badges.roblox.com/v1/badges/{}", badge_id))
                    .json(&json!({
                        "name": name,
                        "description": description,
                        "enabled": enabled,
                    })))
            })
            .await;

        handle(res).await?;

        Ok(())
    }

    pub async fn get_create_badge_free_quota(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<i32> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.get(format!(
                    "https://badges.roblox.com/v1/universes/{}/free-badges-quota",
                    experience_id
                )))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn list_badges(
        &self,
        experience_id: AssetId,
        page_cursor: Option<String>,
    ) -> RobloxApiResult<ListBadgesResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                let mut req = self.client.get(format!(
                    "https://badges.roblox.com/v1/universes/{}/badges",
                    experience_id
                ));
                if let Some(page_cursor) = &page_cursor {
                    req = req.query(&[("cursor", page_cursor)]);
                }
                Ok(req)
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn get_all_badges(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<Vec<ListBadgeResponse>> {
        let mut all_badges = Vec::new();

        let mut page_cursor: Option<String> = None;
        loop {
            let res = self.list_badges(experience_id, page_cursor).await?;
            all_badges.extend(res.data);

            if res.next_page_cursor.is_none() {
                break;
            }

            page_cursor = res.next_page_cursor;
        }

        Ok(all_badges)
    }

    pub async fn update_badge_icon(
        &self,
        badge_id: AssetId,
        icon_file: PathBuf,
    ) -> RobloxApiResult<UploadImageResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(&format!(
                        "https://publish.roblox.com/v1/badges/{}/icon",
                        badge_id
                    ))
                    .multipart(Form::new().part("request.files", get_file_part(&icon_file).await?)))
            })
            .await;

        handle_as_json(res).await
    }
}
