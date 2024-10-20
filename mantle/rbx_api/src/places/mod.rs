pub mod models;

use std::{fs, path::PathBuf};

use reqwest::{Body, StatusCode};
use serde_json::json;

use crate::{
    errors::{RobloxApiError, RobloxApiResult},
    helpers::{handle, handle_as_json, handle_as_json_with_status},
    models::AssetId,
    RobloxApi,
};

use self::models::{
    CreatePlaceResponse, GetPlaceResponse, ListPlaceResponse, ListPlacesResponse,
    PlaceConfigurationModel, PlaceFileFormat, RemovePlaceResponse,
};

impl RobloxApi {
    pub async fn upload_place(
        &self,
        place_file: PathBuf,
        place_id: AssetId,
    ) -> RobloxApiResult<()> {
        let file_format = match place_file.extension().and_then(|e| e.to_str()) {
            Some("rbxl") => PlaceFileFormat::Binary,
            Some("rbxlx") => PlaceFileFormat::Xml,
            _ => {
                return Err(RobloxApiError::InvalidFileExtension(
                    place_file.display().to_string(),
                ))
            }
        };

        let res = self
            .csrf_token_store
            .send_request(|| async {
                let data = fs::read(&place_file)?;

                let body: Body = match file_format {
                    PlaceFileFormat::Binary => data.into(),
                    PlaceFileFormat::Xml => String::from_utf8(data)?.into(),
                };

                let content_type = match file_format {
                    PlaceFileFormat::Binary => "application/octet-stream",
                    PlaceFileFormat::Xml => "application/xml",
                };

                let req = self
                    .client
                    .post("https://data.roblox.com/Data/Upload.ashx")
                    .query(&[("assetId", place_id.to_string())])
                    .header("Content-Type", content_type)
                    .body(body);

                Ok(req)
            })
            .await;

        let result = handle(res).await;

        match result {
            Err(RobloxApiError::Roblox {
                status_code,
                reason,
            }) => match (file_format, status_code) {
                (PlaceFileFormat::Xml, StatusCode::PAYLOAD_TOO_LARGE) => {
                    Err(RobloxApiError::RbxlxPlaceFileSizeTooLarge)
                }
                (PlaceFileFormat::Xml, StatusCode::NOT_FOUND) => {
                    Err(RobloxApiError::RbxlxPlaceFileSizeMayBeTooLarge)
                }
                (PlaceFileFormat::Binary, StatusCode::PAYLOAD_TOO_LARGE) => {
                    Err(RobloxApiError::RbxlPlaceFileSizeTooLarge)
                }
                (PlaceFileFormat::Binary, StatusCode::NOT_FOUND) => {
                    Err(RobloxApiError::RbxlPlaceFileSizeMayBeTooLarge)
                }
                _ => Err(RobloxApiError::Roblox {
                    status_code,
                    reason,
                }),
            },
            Err(e) => Err(e),
            Ok(_) => Ok(()),
        }
    }

    pub async fn get_place(&self, place_id: AssetId) -> RobloxApiResult<GetPlaceResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .get(format!("https://develop.roblox.com/v2/places/{}", place_id)))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn list_places(
        &self,
        experience_id: AssetId,
        page_cursor: Option<String>,
    ) -> RobloxApiResult<ListPlacesResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                let mut req = self.client.get(format!(
                    "https://develop.roblox.com/v1/universes/{}/places",
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

    pub async fn get_all_places(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<Vec<GetPlaceResponse>> {
        let mut all_places = Vec::new();

        let mut page_cursor: Option<String> = None;
        loop {
            let res = self.list_places(experience_id, page_cursor).await?;
            for ListPlaceResponse { id } in res.data {
                let place = self.get_place(id).await?;
                all_places.push(place);
            }

            if res.next_page_cursor.is_none() {
                break;
            }

            page_cursor = res.next_page_cursor;
        }

        Ok(all_places)
    }

    pub async fn remove_place_from_experience(
        &self,
        experience_id: AssetId,
        place_id: AssetId,
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post("https://www.roblox.com/universes/removeplace")
                    .form(&[
                        ("universeId", &experience_id.to_string()),
                        ("placeId", &place_id.to_string()),
                    ]))
            })
            .await;

        handle_as_json_with_status::<RemovePlaceResponse>(res).await?;

        Ok(())
    }

    pub async fn create_place(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<CreatePlaceResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(format!(
                        "https://apis.roblox.com/universes/v1/user/universes/{}/places",
                        experience_id
                    ))
                    .json(&json!({
                        "templatePlaceId": 95206881
                    })))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn configure_place(
        &self,
        place_id: AssetId,
        place_configuration: &PlaceConfigurationModel,
    ) -> RobloxApiResult<()> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .patch(format!("https://develop.roblox.com/v2/places/{}", place_id))
                    .json(place_configuration))
            })
            .await;

        handle(res).await?;

        Ok(())
    }
}
