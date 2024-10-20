pub mod models;

use std::path::PathBuf;

use reqwest::multipart::Form;
use serde_json::json;

use crate::{
    errors::RobloxApiResult,
    helpers::{get_file_part, handle, handle_as_json},
    models::{AssetId, UploadImageResponse},
    RobloxApi,
};

use self::models::{GetExperienceThumbnailResponse, GetExperienceThumbnailsResponse};

impl RobloxApi {
    pub async fn upload_icon(
        &self,
        experience_id: AssetId,
        icon_file: PathBuf,
    ) -> RobloxApiResult<UploadImageResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(&format!(
                        "https://publish.roblox.com/v1/games/{}/icon",
                        experience_id
                    ))
                    .multipart(Form::new().part("request.files", get_file_part(&icon_file).await?)))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn upload_thumbnail(
        &self,
        experience_id: AssetId,
        thumbnail_file: PathBuf,
    ) -> RobloxApiResult<UploadImageResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(&format!(
                        "https://publish.roblox.com/v1/games/{}/thumbnail/image",
                        experience_id
                    ))
                    .multipart(
                        Form::new().part("request.files", get_file_part(&thumbnail_file).await?),
                    ))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn remove_experience_icon(
        &self,
        start_place_id: AssetId,
        icon_asset_id: AssetId,
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post("https://www.roblox.com/places/icons/remove-icon")
                    .form(&[
                        ("placeId", &start_place_id.to_string()),
                        ("placeIconId", &icon_asset_id.to_string()),
                    ]))
            })
            .await;

        handle(res).await?;

        Ok(())
    }

    pub async fn get_experience_thumbnails(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<Vec<GetExperienceThumbnailResponse>> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.get(format!(
                    "https://games.roblox.com/v1/games/{}/media",
                    experience_id
                )))
            })
            .await;

        Ok(handle_as_json::<GetExperienceThumbnailsResponse>(res)
            .await?
            .data)
    }

    pub async fn set_experience_thumbnail_order(
        &self,
        experience_id: AssetId,
        new_thumbnail_order: &[AssetId],
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(format!(
                        "https://develop.roblox.com/v1/universes/{}/thumbnails/order",
                        experience_id
                    ))
                    .json(&json!({ "thumbnailIds": new_thumbnail_order })))
            })
            .await;

        handle(res).await?;

        Ok(())
    }

    pub async fn delete_experience_thumbnail(
        &self,
        experience_id: AssetId,
        thumbnail_id: AssetId,
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.delete(format!(
                    "https://develop.roblox.com/v1/universes/{}/thumbnails/{}",
                    experience_id, thumbnail_id
                )))
            })
            .await;

        handle(res).await?;

        Ok(())
    }
}
