use anyhow::Result;
use datadog_api_client::datadogV2::api_test_optimization::{
    SearchFlakyTestsOptionalParams, TestOptimizationAPI,
};

use crate::config::Config;
use crate::formatter;

// ---- Service Settings ----

pub async fn settings_get(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(TestOptimizationAPI, cfg);
    let body = crate::util::read_json_file(file)?;
    let resp = api
        .get_test_optimization_service_settings(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get test optimization service settings: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn settings_update(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(TestOptimizationAPI, cfg);
    let body = crate::util::read_json_file(file)?;
    let resp = api
        .update_test_optimization_service_settings(body)
        .await
        .map_err(|e| {
            anyhow::anyhow!("failed to update test optimization service settings: {e:?}")
        })?;
    formatter::output(cfg, &resp)
}

pub async fn settings_delete(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(TestOptimizationAPI, cfg);
    let body = crate::util::read_json_file(file)?;
    api.delete_test_optimization_service_settings(body)
        .await
        .map_err(|e| {
            anyhow::anyhow!("failed to delete test optimization service settings: {e:?}")
        })?;
    eprintln!("Service settings deleted successfully.");
    Ok(())
}

// ---- Flaky Tests ----

pub async fn flaky_tests_search(cfg: &Config, file: Option<String>) -> Result<()> {
    let api = crate::make_api!(TestOptimizationAPI, cfg);
    let params = if let Some(f) = file {
        let body = crate::util::read_json_file(&f)?;
        SearchFlakyTestsOptionalParams::default().body(body)
    } else {
        use datadog_api_client::datadogV2::model::{
            FlakyTestsSearchPageOptions, FlakyTestsSearchRequest,
            FlakyTestsSearchRequestAttributes, FlakyTestsSearchRequestData,
            FlakyTestsSearchRequestDataType,
        };
        let page_opts = FlakyTestsSearchPageOptions::new().limit(100);
        let attrs = FlakyTestsSearchRequestAttributes::new().page(page_opts);
        let data = FlakyTestsSearchRequestData::new()
            .attributes(attrs)
            .type_(FlakyTestsSearchRequestDataType::SEARCH_FLAKY_TESTS_REQUEST);
        let body = FlakyTestsSearchRequest::new().data(data);
        SearchFlakyTestsOptionalParams::default().body(body)
    };
    let resp = api
        .search_flaky_tests(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to search flaky tests: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn flaky_tests_update(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(TestOptimizationAPI, cfg);
    let body = crate::util::read_json_file(file)?;
    let resp = api
        .update_flaky_tests(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update flaky tests: {e:?}"))?;
    formatter::output(cfg, &resp)
}

// ---- Flaky Tests Management Policies ----

pub async fn flaky_tests_management_policies_get(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(TestOptimizationAPI, cfg);
    let body = crate::util::read_json_file(file)?;
    let resp = api
        .get_flaky_tests_management_policies(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get flaky tests management policies: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn flaky_tests_management_policies_update(cfg: &Config, file: &str) -> Result<()> {
    let api = crate::make_api!(TestOptimizationAPI, cfg);
    let body = crate::util::read_json_file(file)?;
    let resp = api
        .update_flaky_tests_management_policies(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update flaky tests management policies: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_flaky_tests_management_policies_get() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let tmp = write_temp_json(
            "pup_test_ftmp_get.json",
            r#"{"data":{"type":"test_optimization_get_flaky_tests_management_policies_request","attributes":{"repository_id":"test-repo"}}}"#,
        );
        let _mock = mock_any(&mut server, "POST", r#"{"data":{}}"#).await;
        let result = super::flaky_tests_management_policies_get(&cfg, tmp.to_str().unwrap()).await;
        assert!(
            result.is_ok(),
            "flaky_tests_management_policies_get failed: {:?}",
            result.err()
        );
        let _ = std::fs::remove_file(tmp);
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_flaky_tests_management_policies_update() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let tmp = write_temp_json(
            "pup_test_ftmp_update.json",
            r#"{"data":{"type":"test_optimization_update_flaky_tests_management_policies_request","attributes":{"repository_id":"test-repo"}}}"#,
        );
        let _mock = mock_any(&mut server, "PATCH", r#"{"data":{}}"#).await;
        let result =
            super::flaky_tests_management_policies_update(&cfg, tmp.to_str().unwrap()).await;
        assert!(
            result.is_ok(),
            "flaky_tests_management_policies_update failed: {:?}",
            result.err()
        );
        let _ = std::fs::remove_file(tmp);
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_flaky_tests_management_policies_get_missing_file() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let result =
            super::flaky_tests_management_policies_get(&cfg, "/nonexistent/file.json").await;
        assert!(
            result.is_err(),
            "flaky_tests_management_policies_get should fail for missing file"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }

    #[tokio::test]
    async fn test_flaky_tests_management_policies_update_missing_file() {
        let _lock = lock_env().await;
        std::env::set_var("DD_TOKEN_STORAGE", "file");
        let server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let result =
            super::flaky_tests_management_policies_update(&cfg, "/nonexistent/file.json").await;
        assert!(
            result.is_err(),
            "flaky_tests_management_policies_update should fail for missing file"
        );
        cleanup_env();
        std::env::remove_var("DD_TOKEN_STORAGE");
    }
}
