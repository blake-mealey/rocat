use serde::Serialize;

use crate::models::AssetId;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GrantAssetPermissionsRequest {
    pub requests: Vec<GrantAssetPermissionsRequestRequest>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GrantAssetPermissionsRequestRequest {
    pub subject_type: GrantAssetPermissionRequestSubjectType,
    pub subject_id: AssetId,
    pub action: GrantAssetPermissionRequestAction,
}

#[derive(Serialize, Clone)]
pub enum GrantAssetPermissionRequestSubjectType {
    Universe,
}

#[derive(Serialize, Clone)]
pub enum GrantAssetPermissionRequestAction {
    Use,
}

impl From<GrantAssetPermissionsRequestRequest> for GrantAssetPermissionsRequest {
    fn from(single_request: GrantAssetPermissionsRequestRequest) -> Self {
        GrantAssetPermissionsRequest {
            requests: vec![single_request],
        }
    }
}
