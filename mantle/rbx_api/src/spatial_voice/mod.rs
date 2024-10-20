pub mod models;

use crate::{
    errors::RobloxApiResult, helpers::handle_as_json, models::AssetId,
    spatial_voice::models::UpdateSpatialVoiceSettingsResponse, RobloxApi,
};

use self::models::{GetSpatialVoiceSettingsResponse, UpdateSpatialVoiceSettingsRequest};

impl RobloxApi {
    pub async fn update_spatial_voice_settings(
        &self,
        experience_id: AssetId,
        settings: UpdateSpatialVoiceSettingsRequest,
    ) -> RobloxApiResult<UpdateSpatialVoiceSettingsResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(format!(
                        "https://voice.roblox.com/v1/settings/universe/{}",
                        experience_id
                    ))
                    .json(&settings))
            })
            .await;

        handle_as_json::<UpdateSpatialVoiceSettingsResponse>(res).await
    }

    pub async fn get_spatial_voice_settings(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<GetSpatialVoiceSettingsResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.get(format!(
                    "https://voice.roblox.com/v1/settings/universe/{}",
                    experience_id
                )))
            })
            .await;

        handle_as_json::<GetSpatialVoiceSettingsResponse>(res).await
    }
}
