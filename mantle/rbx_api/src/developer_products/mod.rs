pub mod models;

use std::path::PathBuf;

use reqwest::{header, multipart::Form};
use serde_json::json;

use crate::{
    errors::RobloxApiResult,
    helpers::{get_file_part, handle, handle_as_json},
    models::AssetId,
    RobloxApi,
};

use self::models::{
    CreateDeveloperProductIconResponse, CreateDeveloperProductResponse,
    GetDeveloperProductResponse, ListDeveloperProductResponseItem, ListDeveloperProductsResponse,
};

impl RobloxApi {
    pub async fn create_developer_product_icon(
        &self,
        developer_product_id: AssetId,
        icon_file: PathBuf,
    ) -> RobloxApiResult<CreateDeveloperProductIconResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(format!(
                        "https://apis.roblox.com/developer-products/v1/developer-products/{}/image",
                        developer_product_id
                    ))
                    .multipart(Form::new().part("imageFile", get_file_part(&icon_file).await?)))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn create_developer_product(
        &self,
        experience_id: AssetId,
        name: String,
        price: u32,
        description: String,
    ) -> RobloxApiResult<CreateDeveloperProductResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .post(format!(
                "https://apis.roblox.com/developer-products/v1/universes/{}/developerproducts",
                experience_id
            ))
                    .header(header::CONTENT_LENGTH, 0)
                    .query(&[
                        ("name", &name),
                        ("priceInRobux", &price.to_string()),
                        ("description", &description),
                    ]))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn list_developer_products(
        &self,
        experience_id: AssetId,
        page: u32,
    ) -> RobloxApiResult<ListDeveloperProductsResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self
                    .client
                    .get("https://apis.roblox.com/developer-products/v1/developer-products/list")
                    .query(&[
                        ("universeId", &experience_id.to_string()),
                        ("page", &page.to_string()),
                    ]))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn get_all_developer_products(
        &self,
        experience_id: AssetId,
    ) -> RobloxApiResult<Vec<ListDeveloperProductResponseItem>> {
        let mut all_products = Vec::new();

        let mut page: u32 = 1;
        loop {
            let res = self.list_developer_products(experience_id, page).await?;
            all_products.extend(res.developer_products);

            if res.final_page {
                break;
            }

            page += 1;
        }

        Ok(all_products)
    }

    pub async fn get_developer_product(
        &self,
        developer_product_id: AssetId,
    ) -> RobloxApiResult<GetDeveloperProductResponse> {
        let res = self
            .csrf_token_store
            .send_request(|| async {
                Ok(self.client.get(format!(
                    "https://apis.roblox.com/developer-products/v1/developer-products/{}",
                    developer_product_id
                )))
            })
            .await;

        handle_as_json(res).await
    }

    pub async fn update_developer_product(
        &self,
        experience_id: AssetId,
        product_id: AssetId,
        name: String,
        price: u32,
        description: String,
    ) -> RobloxApiResult<()> {
        let res = self.csrf_token_store.send_request(||async {
Ok(self
            .client
            .post(format!(
                "https://apis.roblox.com/developer-products/v1/universes/{}/developerproducts/{}/update",
                experience_id, product_id
            ))
            .json(&json!({
                "Name": name,
                "PriceInRobux": price,
                "Description": description,
            })))
        }).await;

        handle(res).await?;

        Ok(())
    }
}
