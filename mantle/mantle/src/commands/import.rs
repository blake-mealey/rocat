use std::sync::Arc;

use rbx_api::{models::AssetId, RobloxApi};
use rbx_auth::{RobloxCookieStore, RobloxCsrfTokenStore};
use yansi::Paint;

use rbx_mantle::{
    config::load_project_config,
    project::{load_project, Project},
    state::{import_graph, save_state},
};

pub async fn run(project: Option<&str>, environment: Option<&str>, target_id: &str) -> i32 {
    logger::start_action("Loading project:");
    let (project_path, config) = match load_project_config(project) {
        Ok(v) => v,
        Err(e) => {
            logger::end_action(Paint::red(e));
            return 1;
        }
    };
    let Project {
        current_graph,
        mut state,
        environment_config,
        state_config,
        ..
    } = match load_project(project_path.clone(), config, environment).await {
        Ok(Some(v)) => v,
        Ok(None) => {
            logger::end_action("No import necessary");
            return 0;
        }
        Err(e) => {
            logger::end_action(Paint::red(e));
            return 1;
        }
    };

    if !current_graph.get_resource_list().is_empty() {
        logger::end_action("Environment state already exists: no need to import.");
        return 0;
    }

    logger::end_action("Succeeded");

    let target_id = match target_id.parse::<AssetId>() {
        Ok(v) => v,
        Err(e) => {
            logger::log(Paint::red(format!(
                "Experience ID {} is invalid: {}",
                target_id, e
            )));
            return 1;
        }
    };

    logger::start_action("Import target:");
    let cookie_store = match RobloxCookieStore::new() {
        Ok(v) => Arc::new(v),
        Err(e) => {
            logger::end_action(Paint::red(e));
            return 1;
        }
    };
    let csrf_token_store = RobloxCsrfTokenStore::new();
    let roblox_api = match RobloxApi::new(cookie_store, csrf_token_store) {
        Ok(v) => v,
        Err(e) => {
            logger::end_action(Paint::red(e));
            return 1;
        }
    };
    match roblox_api.validate_auth().await {
        Ok(_) => {}
        Err(e) => {
            logger::end_action(Paint::red(e));
            return 1;
        }
    };

    let imported_graph = match import_graph(&roblox_api, target_id).await {
        Ok(v) => v,
        Err(e) => {
            logger::end_action(Paint::red(format!("Failed: {}", e)));
            return 1;
        }
    };
    logger::end_action("Succeeded");

    logger::start_action("Saving state:");
    state.environments.insert(
        environment_config.label.clone(),
        imported_graph.get_resource_list(),
    );
    match save_state(&project_path, &state_config, &state).await {
        Ok(_) => {}
        Err(e) => {
            logger::end_action(Paint::red(e));
            return 1;
        }
    };
    logger::end_action("Succeeded");

    0
}
