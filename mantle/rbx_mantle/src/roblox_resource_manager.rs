use std::{
    env,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use chrono::{DateTime, Duration, Timelike, Utc};
use log::info;
use rbx_api::{
    asset_permissions::models::{
        GrantAssetPermissionRequestAction, GrantAssetPermissionRequestSubjectType,
        GrantAssetPermissionsRequestRequest,
    },
    assets::models::{
        CreateAssetQuota, CreateAudioAssetResponse, CreateImageAssetResponse, QuotaDuration,
    },
    badges::models::CreateBadgeResponse,
    developer_products::models::{
        CreateDeveloperProductIconResponse, CreateDeveloperProductResponse,
        GetDeveloperProductResponse,
    },
    experiences::models::{CreateExperienceResponse, ExperienceConfigurationModel},
    game_passes::models::{CreateGamePassResponse, GetGamePassResponse},
    models::{AssetId, AssetTypeId, CreatorType, UploadImageResponse},
    notifications::models::CreateNotificationResponse,
    places::models::{GetPlaceResponse, PlaceConfigurationModel},
    social_links::models::{CreateSocialLinkResponse, SocialLinkType},
    spatial_voice::models::UpdateSpatialVoiceSettingsRequest,
    RobloxApi,
};
use rbx_auth::{RobloxCookieStore, RobloxCsrfTokenStore};
use rbxcloud::rbx::{
    types::{PlaceId, UniverseId},
    v1::{PublishVersionType, RbxCloud},
};
use serde::{Deserialize, Serialize};
use yansi::Paint;

use super::resource_graph::{
    all_outputs, optional_output, single_output, Resource, ResourceId, ResourceManager,
};

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExperienceInputs {
    pub group_id: Option<AssetId>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExperienceActivationInputs {
    pub is_active: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileInputs {
    pub file_path: String,
    pub file_hash: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaceInputs {
    pub is_start: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SocialLinkInputs {
    pub title: String,
    pub url: String,
    pub link_type: SocialLinkType,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductInputs {
    pub name: String,
    pub description: String,
    pub price: u32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PassInputs {
    pub name: String,
    pub description: String,
    pub price: Option<u32>,
    pub icon_file_path: String,
    pub icon_file_hash: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BadgeInputs {
    pub name: String,
    pub description: String,
    pub enabled: bool,
    pub icon_file_path: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FileWithGroupIdInputs {
    pub file_path: String,
    pub file_hash: String,
    pub group_id: Option<AssetId>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetAliasInputs {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpatialVoiceInputs {
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotificationInputs {
    pub name: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::large_enum_variant)]
pub enum RobloxInputs {
    Experience(ExperienceInputs),
    ExperienceConfiguration(ExperienceConfigurationModel),
    ExperienceActivation(ExperienceActivationInputs),
    ExperienceIcon(FileInputs),
    ExperienceThumbnail(FileInputs),
    ExperienceThumbnailOrder,
    Place(PlaceInputs),
    PlaceFile(FileInputs),
    PlaceConfiguration(PlaceConfigurationModel),
    SocialLink(SocialLinkInputs),
    Product(ProductInputs),
    ProductIcon(FileInputs),
    Pass(PassInputs),
    Badge(BadgeInputs),
    BadgeIcon(FileInputs),
    ImageAsset(FileWithGroupIdInputs),
    AudioAsset(FileWithGroupIdInputs),
    AssetAlias(AssetAliasInputs),
    SpatialVoice(SpatialVoiceInputs),
    Notification(NotificationInputs),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExperienceOutputs {
    pub asset_id: AssetId,
    pub start_place_id: AssetId,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetOutputs {
    pub asset_id: AssetId,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NotificationOutputs {
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlaceFileOutputs {
    pub version: u64,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProductOutputs {
    pub asset_id: AssetId,
    pub product_id: AssetId,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PassOutputs {
    pub asset_id: AssetId,
    pub icon_asset_id: AssetId,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetWithInitialIconOutputs {
    pub asset_id: AssetId,
    pub initial_icon_asset_id: AssetId,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ImageAssetOutputs {
    pub asset_id: AssetId,
    pub decal_asset_id: Option<AssetId>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetAliasOutputs {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum RobloxOutputs {
    Experience(ExperienceOutputs),
    ExperienceConfiguration,
    ExperienceActivation,
    ExperienceIcon(AssetOutputs),
    ExperienceThumbnail(AssetOutputs),
    ExperienceThumbnailOrder,
    Place(AssetOutputs),
    PlaceFile(PlaceFileOutputs),
    PlaceConfiguration,
    SocialLink(AssetOutputs),
    Product(ProductOutputs),
    ProductIcon(AssetOutputs),
    Pass(PassOutputs),
    Badge(AssetWithInitialIconOutputs),
    BadgeIcon(AssetOutputs),
    ImageAsset(ImageAssetOutputs),
    AudioAsset(AssetOutputs),
    AssetAlias(AssetAliasOutputs),
    SpatialVoice,
    Notification(NotificationOutputs),
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RobloxResource {
    id: ResourceId,
    inputs: RobloxInputs,
    outputs: Option<RobloxOutputs>,
    dependencies: Vec<ResourceId>,
}

impl RobloxResource {
    pub fn new(id: &str, inputs: RobloxInputs, dependencies: &[&RobloxResource]) -> Self {
        Self {
            id: id.to_owned(),
            inputs,
            outputs: None,
            dependencies: dependencies.iter().map(|d| d.get_id()).collect(),
        }
    }

    pub fn existing(
        id: &str,
        inputs: RobloxInputs,
        outputs: RobloxOutputs,
        dependencies: &[&RobloxResource],
    ) -> Self {
        Self {
            id: id.to_owned(),
            inputs,
            outputs: Some(outputs),
            dependencies: dependencies.iter().map(|d| d.get_id()).collect(),
        }
    }

    pub fn add_dependency(&mut self, dependency: &RobloxResource) -> &mut Self {
        self.dependencies.push(dependency.get_id());
        self
    }
}

impl Resource<RobloxInputs, RobloxOutputs> for RobloxResource {
    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_inputs_hash(&self) -> String {
        // TODO: Should we separate hashes from displays?
        let hash = serde_yaml::to_string(&self.inputs)
            .map_err(|e| format!("Failed to compute inputs hash\n\t{}", e))
            .unwrap();
        if hash.is_empty() {
            ""
        } else {
            // We remove first 4 characters to remove "---\n", and we trim the end to remove "\n"
            hash[4..].trim_end()
        }
        .to_owned()
    }

    fn get_outputs_hash(&self) -> String {
        // TODO: Should we separate hashes from displays?
        let hash = serde_yaml::to_string(&self.outputs)
            .map_err(|e| format!("Failed to compute outputs hash\n\t{}", e))
            .unwrap();
        if hash.is_empty() {
            ""
        } else {
            // We remove first 4 characters to remove "---\n", and we trim the end to remove "\n"
            hash[4..].trim_end()
        }
        .to_owned()
    }

    fn get_inputs(&self) -> RobloxInputs {
        self.inputs.clone()
    }

    fn get_outputs(&self) -> Option<RobloxOutputs> {
        self.outputs.clone()
    }

    fn get_dependencies(&self) -> Vec<ResourceId> {
        self.dependencies.clone()
    }

    fn set_outputs(&mut self, outputs: RobloxOutputs) {
        self.outputs = Some(outputs);
    }
}

pub struct RobloxResourceManager {
    roblox_api: RobloxApi,
    roblox_cloud: Option<RbxCloud>,
    project_path: PathBuf,
    payment_source: CreatorType,
}

impl RobloxResourceManager {
    pub async fn new(project_path: &Path, payment_source: CreatorType) -> Result<Self, String> {
        let cookie_store = Arc::new(RobloxCookieStore::new()?);
        let csrf_token_store = RobloxCsrfTokenStore::new();
        let roblox_api = RobloxApi::new(cookie_store, csrf_token_store)?;
        roblox_api.validate_auth().await?;

        let open_cloud_api_key = match env::var("MANTLE_OPEN_CLOUD_API_KEY") {
            Ok(v) => {
                info!("Loaded cookie from ROBLOSECURITY environment variable.");
                Some(v)
            }
            Err(_) => None,
        };

        let roblox_cloud = open_cloud_api_key.map(|api_key| RbxCloud::new(&api_key));

        Ok(Self {
            roblox_api,
            roblox_cloud,
            project_path: project_path.to_path_buf(),
            payment_source,
        })
    }

    fn get_path(&self, file: String) -> PathBuf {
        self.project_path.join(file)
    }
}

#[async_trait]
impl ResourceManager<RobloxInputs, RobloxOutputs> for RobloxResourceManager {
    async fn get_create_price(
        &self,
        inputs: RobloxInputs,
        dependency_outputs: Vec<RobloxOutputs>,
    ) -> Result<Option<u32>, String> {
        match inputs {
            RobloxInputs::Badge(_) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);
                let free_quota = self
                    .roblox_api
                    .get_create_badge_free_quota(experience.asset_id)
                    .await?;

                let quota_reset = format_quota_reset(
                    (Utc::now() + Duration::days(1))
                        .with_hour(0)
                        .unwrap()
                        .with_minute(0)
                        .unwrap()
                        .with_second(0)
                        .unwrap()
                        .with_nanosecond(0)
                        .unwrap(),
                );

                if free_quota > 0 {
                    logger::log("");
                    logger::log(Paint::yellow(
                        format!("You will have {} free badge(s) remaining in the current period after creation. Your quota will reset in {}.", free_quota - 1, quota_reset),
                    ));
                    Ok(None)
                } else {
                    logger::log("");
                    logger::log(Paint::yellow(
                        format!("You have no free badges remaining in the current period. Your quota will reset in {}.", quota_reset),
                    ));

                    Ok(Some(100))
                }
            }
            _ => Ok(None),
        }
    }

    async fn create(
        &self,
        inputs: RobloxInputs,
        dependency_outputs: Vec<RobloxOutputs>,
        price: Option<u32>,
    ) -> Result<RobloxOutputs, String> {
        match inputs {
            RobloxInputs::Experience(inputs) => {
                let CreateExperienceResponse {
                    universe_id,
                    root_place_id,
                } = self.roblox_api.create_experience(inputs.group_id).await?;

                Ok(RobloxOutputs::Experience(ExperienceOutputs {
                    asset_id: universe_id,
                    start_place_id: root_place_id,
                }))
            }
            RobloxInputs::ExperienceConfiguration(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .configure_experience(experience.asset_id, &inputs)
                    .await?;

                Ok(RobloxOutputs::ExperienceConfiguration)
            }
            RobloxInputs::ExperienceActivation(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .set_experience_active(experience.asset_id, inputs.is_active)
                    .await?;

                Ok(RobloxOutputs::ExperienceActivation)
            }
            RobloxInputs::ExperienceIcon(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let UploadImageResponse { target_id } = self
                    .roblox_api
                    .upload_icon(experience.asset_id, self.get_path(inputs.file_path))
                    .await?;

                Ok(RobloxOutputs::ExperienceIcon(AssetOutputs {
                    asset_id: target_id,
                }))
            }
            RobloxInputs::ExperienceThumbnail(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let UploadImageResponse { target_id } = self
                    .roblox_api
                    .upload_thumbnail(experience.asset_id, self.get_path(inputs.file_path))
                    .await?;

                Ok(RobloxOutputs::ExperienceThumbnail(AssetOutputs {
                    asset_id: target_id,
                }))
            }
            RobloxInputs::ExperienceThumbnailOrder => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);
                let thumbnails =
                    all_outputs!(dependency_outputs, RobloxOutputs::ExperienceThumbnail);

                self.roblox_api
                    .set_experience_thumbnail_order(
                        experience.asset_id,
                        &thumbnails.iter().map(|t| t.asset_id).collect::<Vec<_>>(),
                    )
                    .await?;

                Ok(RobloxOutputs::ExperienceThumbnailOrder)
            }
            RobloxInputs::Place(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let asset_id = if inputs.is_start {
                    experience.start_place_id
                } else {
                    self.roblox_api
                        .create_place(experience.asset_id)
                        .await?
                        .place_id
                };

                Ok(RobloxOutputs::Place(AssetOutputs { asset_id }))
            }
            RobloxInputs::PlaceFile(inputs) => {
                let place = single_output!(dependency_outputs, RobloxOutputs::Place);
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                if let Some(roblox_cloud) = &self.roblox_cloud {
                    let response = roblox_cloud
                        .experience(UniverseId(experience.asset_id), PlaceId(place.asset_id))
                        .publish(
                            &self
                                .get_path(inputs.file_path)
                                .into_os_string()
                                .into_string()
                                .unwrap(),
                            PublishVersionType::Published,
                        )
                        .await
                        .map_err(|e| e.to_string())?;

                    Ok(RobloxOutputs::PlaceFile(PlaceFileOutputs {
                        version: response.version_number,
                    }))
                } else {
                    self.roblox_api
                        .upload_place(self.get_path(inputs.file_path), place.asset_id)
                        .await?;
                    let GetPlaceResponse {
                        current_saved_version,
                        ..
                    } = self.roblox_api.get_place(place.asset_id).await?;

                    Ok(RobloxOutputs::PlaceFile(PlaceFileOutputs {
                        version: current_saved_version,
                    }))
                }
            }
            RobloxInputs::PlaceConfiguration(inputs) => {
                let place = single_output!(dependency_outputs, RobloxOutputs::Place);

                self.roblox_api
                    .configure_place(place.asset_id, &inputs)
                    .await?;

                Ok(RobloxOutputs::PlaceConfiguration)
            }
            RobloxInputs::SocialLink(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let CreateSocialLinkResponse { id } = self
                    .roblox_api
                    .create_social_link(
                        experience.asset_id,
                        inputs.title,
                        inputs.url,
                        inputs.link_type,
                    )
                    .await?;

                Ok(RobloxOutputs::SocialLink(AssetOutputs { asset_id: id }))
            }
            RobloxInputs::ProductIcon(inputs) => {
                let product = single_output!(dependency_outputs, RobloxOutputs::Product);

                let CreateDeveloperProductIconResponse { image_asset_id } = self
                    .roblox_api
                    .create_developer_product_icon(
                        product.asset_id,
                        self.get_path(inputs.file_path),
                    )
                    .await?;

                Ok(RobloxOutputs::ProductIcon(AssetOutputs {
                    asset_id: image_asset_id,
                }))
            }
            RobloxInputs::Product(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let CreateDeveloperProductResponse { id } = self
                    .roblox_api
                    .create_developer_product(
                        experience.asset_id,
                        inputs.name,
                        inputs.price,
                        inputs.description,
                    )
                    .await?;

                let GetDeveloperProductResponse { id: product_id } =
                    self.roblox_api.get_developer_product(id).await?;

                Ok(RobloxOutputs::Product(ProductOutputs {
                    asset_id: product_id,
                    product_id: id,
                }))
            }
            RobloxInputs::Pass(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let CreateGamePassResponse { game_pass_id } = self
                    .roblox_api
                    .create_game_pass(
                        experience.asset_id,
                        inputs.name.clone(),
                        inputs.description.clone(),
                        self.get_path(inputs.icon_file_path),
                    )
                    .await?;
                let GetGamePassResponse {
                    icon_image_asset_id,
                    ..
                } = self
                    .roblox_api
                    .update_game_pass(
                        game_pass_id,
                        inputs.name,
                        inputs.description,
                        inputs.price,
                        None,
                    )
                    .await?;

                Ok(RobloxOutputs::Pass(PassOutputs {
                    asset_id: game_pass_id,
                    icon_asset_id: icon_image_asset_id,
                }))
            }
            RobloxInputs::Badge(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let CreateBadgeResponse { id, icon_image_id } = self
                    .roblox_api
                    .create_badge(
                        experience.asset_id,
                        inputs.name,
                        inputs.description,
                        self.get_path(inputs.icon_file_path),
                        self.payment_source.clone(),
                        price.unwrap_or(0),
                    )
                    .await?;

                Ok(RobloxOutputs::Badge(AssetWithInitialIconOutputs {
                    asset_id: id,
                    initial_icon_asset_id: icon_image_id,
                }))
            }
            RobloxInputs::BadgeIcon(_) => {
                let badge = single_output!(dependency_outputs, RobloxOutputs::Badge);

                Ok(RobloxOutputs::BadgeIcon(AssetOutputs {
                    asset_id: badge.initial_icon_asset_id,
                }))
            }
            RobloxInputs::ImageAsset(inputs) => {
                let CreateImageAssetResponse {
                    asset_id,
                    backing_asset_id,
                    ..
                } = self
                    .roblox_api
                    .create_image_asset(self.get_path(inputs.file_path), inputs.group_id)
                    .await?;

                Ok(RobloxOutputs::ImageAsset(ImageAssetOutputs {
                    asset_id: backing_asset_id,
                    decal_asset_id: Some(asset_id),
                }))
            }
            RobloxInputs::AudioAsset(inputs) => {
                let CreateAssetQuota {
                    usage,
                    capacity,
                    expiration_time,
                    duration,
                } = self
                    .roblox_api
                    .get_create_asset_quota(AssetTypeId::Audio)
                    .await?;

                let quota_reset = format_quota_reset(match expiration_time {
                    Some(ref x) => DateTime::parse_from_rfc3339(x)
                        .map_err(|e| format!("Unable to parse expiration_time: {}", e))?
                        .with_timezone(&Utc),
                    None => {
                        Utc::now()
                            + match duration {
                                // TODO: Learn how Roblox computes a "Month" to ensure this is an accurate estimate
                                QuotaDuration::Month => Duration::days(30),
                            }
                    }
                });

                if usage < capacity {
                    logger::log("");
                    logger::log(Paint::yellow(
                        format!(
                        "You will have {} audio upload(s) remaining in the current period after creation. Your quota will reset in {}.",
                        capacity - usage - 1,
                        quota_reset
                    )));

                    let CreateAudioAssetResponse { id } = self
                        .roblox_api
                        .create_audio_asset(
                            self.get_path(inputs.file_path),
                            inputs.group_id,
                            self.payment_source.clone(),
                        )
                        .await?;

                    Ok(RobloxOutputs::AudioAsset(AssetOutputs { asset_id: id }))
                } else {
                    Err(format!(
                        "You have reached your audio upload quota. Your quota will reset in {}.",
                        quota_reset
                    ))
                }
            }
            RobloxInputs::AssetAlias(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let image_asset = optional_output!(dependency_outputs, RobloxOutputs::ImageAsset);
                let audio_asset = optional_output!(dependency_outputs, RobloxOutputs::AudioAsset);
                let asset_id = match (image_asset, audio_asset) {
                    (Some(image_asset), None) => image_asset.asset_id,
                    (None, Some(audio_asset)) => audio_asset.asset_id,
                    _ => panic!("Missing expected output."),
                };

                self.roblox_api
                    .create_asset_alias(experience.asset_id, asset_id, inputs.name.clone())
                    .await?;

                if audio_asset.is_some() {
                    self.roblox_api
                        .grant_asset_permissions(
                            asset_id,
                            GrantAssetPermissionsRequestRequest {
                                subject_id: experience.asset_id,
                                subject_type: GrantAssetPermissionRequestSubjectType::Universe,
                                action: GrantAssetPermissionRequestAction::Use,
                            },
                        )
                        .await?;
                }

                Ok(RobloxOutputs::AssetAlias(AssetAliasOutputs {
                    name: inputs.name,
                }))
            }
            RobloxInputs::SpatialVoice(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .update_spatial_voice_settings(
                        experience.asset_id,
                        UpdateSpatialVoiceSettingsRequest {
                            opt_in: inputs.enabled,
                        },
                    )
                    .await?;

                Ok(RobloxOutputs::SpatialVoice)
            }
            RobloxInputs::Notification(inputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let CreateNotificationResponse { id } = self
                    .roblox_api
                    .create_notification(experience.asset_id, inputs.name, inputs.content)
                    .await?;

                Ok(RobloxOutputs::Notification(NotificationOutputs { id }))
            }
        }
    }

    async fn get_update_price(
        &self,
        _inputs: RobloxInputs,
        _outputs: RobloxOutputs,
        _dependency_outputs: Vec<RobloxOutputs>,
    ) -> Result<Option<u32>, String> {
        Ok(None)
    }

    // TODO: Consider moving `outputs` into `dependency_outputs`.
    async fn update(
        &self,
        inputs: RobloxInputs,
        outputs: RobloxOutputs,
        dependency_outputs: Vec<RobloxOutputs>,
        price: Option<u32>,
    ) -> Result<RobloxOutputs, String> {
        match (inputs.clone(), outputs.clone()) {
            (RobloxInputs::Experience(_), RobloxOutputs::Experience(_)) => {
                self.delete(outputs, dependency_outputs.clone()).await?;
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::ExperienceConfiguration(_), RobloxOutputs::ExperienceConfiguration) => {
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::ExperienceActivation(_), RobloxOutputs::ExperienceActivation) => {
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::ExperienceIcon(_), RobloxOutputs::ExperienceIcon(_)) => {
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::ExperienceThumbnail(_), RobloxOutputs::ExperienceThumbnail(_)) => {
                self.delete(outputs, dependency_outputs.clone()).await?;
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::ExperienceThumbnailOrder, RobloxOutputs::ExperienceThumbnailOrder) => {
                self.create(inputs, dependency_outputs, price).await
            }
            // TODO: is this correct?
            (RobloxInputs::Place(_), RobloxOutputs::Place(_)) => {
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::PlaceFile(_), RobloxOutputs::PlaceFile(_)) => {
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::PlaceConfiguration(_), RobloxOutputs::PlaceConfiguration) => {
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::SocialLink(inputs), RobloxOutputs::SocialLink(outputs)) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .update_social_link(
                        experience.asset_id,
                        outputs.asset_id,
                        inputs.title,
                        inputs.url,
                        inputs.link_type,
                    )
                    .await?;

                Ok(RobloxOutputs::SocialLink(outputs))
            }
            (RobloxInputs::ProductIcon(_), RobloxOutputs::ProductIcon(_)) => {
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::Product(inputs), RobloxOutputs::Product(outputs)) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .update_developer_product(
                        experience.asset_id,
                        outputs.asset_id,
                        inputs.name,
                        inputs.price,
                        inputs.description,
                    )
                    .await?;

                Ok(RobloxOutputs::Product(outputs))
            }
            (RobloxInputs::Pass(inputs), RobloxOutputs::Pass(outputs)) => {
                let GetGamePassResponse {
                    icon_image_asset_id,
                    ..
                } = self
                    .roblox_api
                    .update_game_pass(
                        outputs.asset_id,
                        inputs.name,
                        inputs.description,
                        inputs.price,
                        Some(self.get_path(inputs.icon_file_path)),
                    )
                    .await?;

                Ok(RobloxOutputs::Pass(PassOutputs {
                    asset_id: outputs.asset_id,
                    icon_asset_id: icon_image_asset_id,
                }))
            }
            (RobloxInputs::Badge(inputs), RobloxOutputs::Badge(outputs)) => {
                self.roblox_api
                    .update_badge(
                        outputs.asset_id,
                        inputs.name,
                        inputs.description,
                        inputs.enabled,
                    )
                    .await?;

                Ok(RobloxOutputs::Badge(outputs))
            }
            (RobloxInputs::BadgeIcon(inputs), RobloxOutputs::BadgeIcon(_)) => {
                let badge = single_output!(dependency_outputs, RobloxOutputs::Badge);

                let UploadImageResponse { target_id } = self
                    .roblox_api
                    .update_badge_icon(badge.asset_id, self.get_path(inputs.file_path))
                    .await?;

                Ok(RobloxOutputs::BadgeIcon(AssetOutputs {
                    asset_id: target_id,
                }))
            }
            (RobloxInputs::ImageAsset(_), RobloxOutputs::ImageAsset(_)) => {
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::AudioAsset(_), RobloxOutputs::AudioAsset(_)) => {
                self.create(inputs, dependency_outputs, price).await
            }
            (RobloxInputs::AssetAlias(inputs), RobloxOutputs::AssetAlias(outputs)) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let image_asset = optional_output!(dependency_outputs, RobloxOutputs::ImageAsset);
                let audio_asset = optional_output!(dependency_outputs, RobloxOutputs::AudioAsset);
                let asset_id = match (image_asset, audio_asset) {
                    (Some(image_asset), None) => image_asset.asset_id,
                    (None, Some(audio_asset)) => audio_asset.asset_id,
                    _ => panic!("Missing expected output."),
                };

                self.roblox_api
                    .update_asset_alias(
                        experience.asset_id,
                        asset_id,
                        outputs.name,
                        inputs.name.clone(),
                    )
                    .await?;

                if audio_asset.is_some() {
                    self.roblox_api
                        .grant_asset_permissions(
                            asset_id,
                            GrantAssetPermissionsRequestRequest {
                                subject_id: experience.asset_id,
                                subject_type: GrantAssetPermissionRequestSubjectType::Universe,
                                action: GrantAssetPermissionRequestAction::Use,
                            },
                        )
                        .await?;
                }

                Ok(RobloxOutputs::AssetAlias(AssetAliasOutputs {
                    name: inputs.name,
                }))
            }
            (RobloxInputs::SpatialVoice(inputs), RobloxOutputs::SpatialVoice) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .update_spatial_voice_settings(
                        experience.asset_id,
                        UpdateSpatialVoiceSettingsRequest {
                            opt_in: inputs.enabled,
                        },
                    )
                    .await?;

                Ok(RobloxOutputs::SpatialVoice)
            }
            (RobloxInputs::Notification(inputs), RobloxOutputs::Notification(outputs)) => {
                let asset_id = outputs.id.clone();
                self.roblox_api
                    .update_notification(asset_id, inputs.name, inputs.content)
                    .await?;

                Ok(RobloxOutputs::Notification(outputs))
            }
            _ => unreachable!(),
        }
    }

    // TODO: Do we need inputs?
    async fn delete(
        &self,
        outputs: RobloxOutputs,
        dependency_outputs: Vec<RobloxOutputs>,
    ) -> Result<(), String> {
        match outputs {
            RobloxOutputs::Experience(outputs) => {
                let model = ExperienceConfigurationModel {
                    is_archived: true,
                    ..Default::default()
                };
                self.roblox_api
                    .configure_experience(outputs.asset_id, &model)
                    .await?;
            }
            RobloxOutputs::ExperienceConfiguration => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let model = ExperienceConfigurationModel::default();
                self.roblox_api
                    .configure_experience(experience.asset_id, &model)
                    .await?;
            }
            RobloxOutputs::ExperienceActivation => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .set_experience_active(experience.asset_id, false)
                    .await?;
            }
            RobloxOutputs::ExperienceIcon(outputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .remove_experience_icon(experience.start_place_id, outputs.asset_id)
                    .await?;
            }
            RobloxOutputs::ExperienceThumbnail(outputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .delete_experience_thumbnail(experience.asset_id, outputs.asset_id)
                    .await?;
            }
            RobloxOutputs::ExperienceThumbnailOrder => {}
            RobloxOutputs::Place(outputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                if outputs.asset_id != experience.start_place_id {
                    self.roblox_api
                        .remove_place_from_experience(experience.asset_id, outputs.asset_id)
                        .await?;
                }
            }
            RobloxOutputs::PlaceFile(_) => {}
            RobloxOutputs::PlaceConfiguration => {
                let place = single_output!(dependency_outputs, RobloxOutputs::Place);

                let model = PlaceConfigurationModel::default();
                self.roblox_api
                    .configure_place(place.asset_id, &model)
                    .await?;
            }
            RobloxOutputs::SocialLink(outputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .delete_social_link(experience.asset_id, outputs.asset_id)
                    .await?;
            }
            RobloxOutputs::ProductIcon(_) => {}
            RobloxOutputs::Product(outputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                let utc = Utc::now();
                self.roblox_api
                    .update_developer_product(
                        experience.asset_id,
                        outputs.asset_id,
                        format!("zzz_DEPRECATED({})", utc.format("%F %T%.f")),
                        0,
                        "".to_owned(),
                    )
                    .await?;
            }
            RobloxOutputs::Pass(outputs) => {
                let utc = Utc::now();
                self.roblox_api
                    .update_game_pass(
                        outputs.asset_id,
                        format!("zzz_DEPRECATED({})", utc.format("%F %T%.f")),
                        "".to_owned(),
                        None,
                        None,
                    )
                    .await?;
            }
            RobloxOutputs::Badge(outputs) => {
                let utc = Utc::now();
                self.roblox_api
                    .update_badge(
                        outputs.asset_id,
                        format!("zzz_DEPRECATED({})", utc.format("%F %T%.f")),
                        "".to_owned(),
                        false,
                    )
                    .await?;
            }
            RobloxOutputs::BadgeIcon(_) => {}
            RobloxOutputs::ImageAsset(outputs) => {
                // TODO: Can we make this not optional and just not import the image asset? Maybe?
                if let Some(decal_asset_id) = outputs.decal_asset_id {
                    self.roblox_api.archive_asset(decal_asset_id).await?;
                }
            }
            RobloxOutputs::AudioAsset(outputs) => {
                self.roblox_api.archive_asset(outputs.asset_id).await?;
            }
            RobloxOutputs::AssetAlias(outputs) => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .delete_asset_alias(experience.asset_id, outputs.name)
                    .await?;
            }
            RobloxOutputs::SpatialVoice => {
                let experience = single_output!(dependency_outputs, RobloxOutputs::Experience);

                self.roblox_api
                    .update_spatial_voice_settings(
                        experience.asset_id,
                        UpdateSpatialVoiceSettingsRequest { opt_in: false },
                    )
                    .await?;
            }
            RobloxOutputs::Notification(outputs) => {
                self.roblox_api.archive_notification(outputs.id).await?;
            }
        }
        Ok(())
    }
}

fn format_quota_reset(reset: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = reset.signed_duration_since(now);

    let mut parts = Vec::<String>::new();
    if duration.num_days() > 0 {
        parts.push(format!("{}d", duration.num_days()));
    }
    if duration.num_hours() > 0 {
        parts.push(format!(
            "{}h",
            duration.num_hours() - duration.num_days() * 24
        ));
    }
    if duration.num_minutes() > 0 {
        parts.push(format!(
            "{}m",
            duration.num_minutes() - duration.num_hours() * 60
        ));
    }
    if duration.num_seconds() > 0 {
        parts.push(format!(
            "{}s",
            duration.num_seconds() - duration.num_minutes() * 60
        ));
    } else {
        parts.push(format!(
            "{}ms",
            duration.num_milliseconds() - duration.num_seconds() * 1000
        ));
    }

    parts.join(" ")
}
