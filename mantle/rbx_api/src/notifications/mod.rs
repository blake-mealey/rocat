pub mod models;

use serde_json::json;

use crate::{
    errors::RobloxApiResult,
    helpers::{handle, handle_as_json},
    models::AssetId,
    RobloxApi,
};

use self::models::{
    CreateNotificationResponse, ListNotificationResponse, ListNotificationsResponse,
};

impl RobloxApi {
    pub async fn create_notification(
        &self,
        experience_id: AssetId,
        name: String,
        content: String,
    ) -> RobloxApiResult<CreateNotificationResponse> {
        let res = self.csrf_token_store.send_request(||async {
            Ok(self
            .client
            .post("https://apis.roblox.com/notifications/v1/developer-configuration/create-notification")
            .json(&json!({
                "universeId": experience_id,
                "name": name,
                "content": content,
            })))
        }).await;

        handle_as_json(res).await
    }

    pub async fn update_notification(
        &self,
        notification_id: String,
        name: String,
        content: String,
    ) -> RobloxApiResult<()> {
        let res = self.csrf_token_store.send_request(||async {
            Ok(self
            .client
            .post("https://apis.roblox.com/notifications/v1/developer-configuration/update-notification")
            .json(&json!({
                "id": notification_id,
                "name": name,
                "content": content,
            })))
        }).await;

        handle(res).await?;

        Ok(())
    }

    pub async fn archive_notification(&self, notification_id: String) -> RobloxApiResult<()> {
        let res = self.csrf_token_store.send_request(||async {
            Ok(self
            .client
            .post("https://apis.roblox.com/notifications/v1/developer-configuration/archive-notification")
            .json(&json!({
                "id": notification_id,
            })))
        }).await;

        handle(res).await?;

        Ok(())
    }

    pub async fn list_notifications(
        &self,
        experience_id: AssetId,
        count: u8,
        page_cursor: Option<String>,
    ) -> RobloxApiResult<ListNotificationsResponse> {
        let res = self.csrf_token_store.send_request(|| async {
            let mut req = self
                .client
                .get("https://apis.roblox.com/notifications/v1/developer-configuration/experience-notifications-list")
                .query(&[
                    ("universeId", &experience_id.to_string()),
                    ("count", &count.to_string()),
                ]);
            if let Some(page_cursor) = &page_cursor {
                req = req.query(&[("cursor", page_cursor)]);
            }
            Ok(req)
        }).await;

        handle_as_json(res).await
    }

    pub async fn get_all_notifications(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<Vec<ListNotificationResponse>> {
        let mut all_notifications = Vec::new();

        let mut page_cursor: Option<String> = None;
        loop {
            let res = self
                .list_notifications(experience_id, 100, page_cursor)
                .await?;
            all_notifications.extend(res.notification_string_configs);

            match res.next_page_cursor {
                None => break,
                Some(next_page_cursor) if next_page_cursor.is_empty() => break,
                _ => {}
            }

            page_cursor = res.next_page_cursor;
        }

        Ok(all_notifications)
    }
}
