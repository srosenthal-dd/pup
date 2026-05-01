use anyhow::Result;
use datadog_api_client::datadogV2::api_roles::{ListRolesOptionalParams, RolesAPI};
use datadog_api_client::datadogV2::api_service_accounts::{
    ListServiceAccountApplicationKeysOptionalParams, ServiceAccountsAPI,
};
use datadog_api_client::datadogV2::api_users::{ListUsersOptionalParams, UsersAPI};

use crate::config::Config;
use crate::formatter;
use crate::util;

pub async fn list(cfg: &Config, page_size: i64, page_number: i64) -> Result<()> {
    let api = crate::make_api!(UsersAPI, cfg);
    let resp = api
        .list_users(
            ListUsersOptionalParams::default()
                .page_size(page_size)
                .page_number(page_number),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to list users: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn get(cfg: &Config, id: &str) -> Result<()> {
    let api = crate::make_api!(UsersAPI, cfg);
    let resp = api
        .get_user(id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get user: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn roles_list(cfg: &Config) -> Result<()> {
    let api = crate::make_api!(RolesAPI, cfg);
    let resp = api
        .list_roles(ListRolesOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to list roles: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn service_accounts_create(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(ServiceAccountsAPI, cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .create_service_account(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create service account: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn service_account_app_keys_create(
    cfg: &Config,
    service_account_id: &str,
    file: &str,
) -> Result<()> {
    let api = crate::make_api!(ServiceAccountsAPI, cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .create_service_account_application_key(service_account_id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create service account application key: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn service_account_app_keys_list(cfg: &Config, service_account_id: &str) -> Result<()> {
    let api = crate::make_api!(ServiceAccountsAPI, cfg);
    let resp = api
        .list_service_account_application_keys(
            service_account_id.to_string(),
            ListServiceAccountApplicationKeysOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to list service account application keys: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn service_account_app_keys_get(
    cfg: &Config,
    service_account_id: &str,
    app_key_id: &str,
) -> Result<()> {
    let api = crate::make_api!(ServiceAccountsAPI, cfg);
    let resp = api
        .get_service_account_application_key(service_account_id.to_string(), app_key_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get service account application key: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn service_account_app_keys_update(
    cfg: &Config,
    service_account_id: &str,
    app_key_id: &str,
    file: &str,
) -> Result<()> {
    let api = crate::make_api!(ServiceAccountsAPI, cfg);
    let body = util::read_json_file(file)?;
    let resp = api
        .update_service_account_application_key(
            service_account_id.to_string(),
            app_key_id.to_string(),
            body,
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to update service account application key: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn service_account_app_keys_delete(
    cfg: &Config,
    service_account_id: &str,
    app_key_id: &str,
) -> Result<()> {
    let api = crate::make_api!(ServiceAccountsAPI, cfg);
    api.delete_service_account_application_key(
        service_account_id.to_string(),
        app_key_id.to_string(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("failed to delete service account application key: {e:?}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_users_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::list(&cfg, 10, 0).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_users_get() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": {}}"#).await;
        let _ = super::get(&cfg, "u1").await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_users_roles_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::roles_list(&cfg).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_service_account_app_keys_list() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "GET", r#"{"data":[],"meta":{}}"#).await;
        let result = super::service_account_app_keys_list(&cfg, "sa-test-id").await;
        assert!(
            result.is_ok(),
            "service account app keys list failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_service_account_app_keys_delete_missing() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("DELETE", mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Not Found"]}"#)
            .create_async()
            .await;
        let result =
            super::service_account_app_keys_delete(&cfg, "sa-test-id", "key-not-found").await;
        assert!(result.is_err(), "expected error for missing app key delete");
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_service_accounts_create() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "POST",
            r#"{"data":{"type":"users","id":"sa-new","attributes":{}}}"#,
        )
        .await;
        let result = super::service_accounts_create(&cfg, "/tmp/test.json").await;
        assert!(result.is_err(), "expected error for missing input file");
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_service_account_app_keys_get() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "GET",
            r#"{"data":{"type":"api_keys","id":"key-1","attributes":{}}}"#,
        )
        .await;
        let result = super::service_account_app_keys_get(&cfg, "sa-id", "key-1").await;
        assert!(
            result.is_ok(),
            "service account app key get failed: {:?}",
            result.err()
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_service_account_app_keys_get_error() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(404)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Not Found"]}"#)
            .create_async()
            .await;
        let result = super::service_account_app_keys_get(&cfg, "sa-id", "key-missing").await;
        assert!(result.is_err(), "expected error for missing app key get");
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_service_account_app_keys_create() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("POST", mockito::Matcher::Any)
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Forbidden"]}"#)
            .create_async()
            .await;
        let result = super::service_account_app_keys_create(&cfg, "sa-id", "/tmp/test.json").await;
        assert!(
            result.is_err(),
            "expected error for service account app key create"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_service_account_app_keys_update() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = server
            .mock("PATCH", mockito::Matcher::Any)
            .with_status(403)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["Forbidden"]}"#)
            .create_async()
            .await;
        let result =
            super::service_account_app_keys_update(&cfg, "sa-id", "key-1", "/tmp/test.json").await;
        assert!(
            result.is_err(),
            "expected error for service account app key update"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }
}
