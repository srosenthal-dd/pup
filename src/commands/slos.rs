use anyhow::Result;
use datadog_api_client::datadogV1::api_service_level_objectives::{
    DeleteSLOOptionalParams, GetSLOOptionalParams, ListSLOsOptionalParams,
    ServiceLevelObjectivesAPI,
};
use datadog_api_client::datadogV1::model::{ServiceLevelObjective, ServiceLevelObjectiveRequest};

use crate::config::Config;
use crate::formatter;
use crate::util;

pub async fn list(
    cfg: &Config,
    query: Option<String>,
    tags_query: Option<String>,
    metrics_query: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<()> {
    let api = crate::make_api!(ServiceLevelObjectivesAPI, cfg);
    let mut params = ListSLOsOptionalParams::default();
    if let Some(query) = query {
        params = params.query(query);
    }
    if let Some(tags_query) = tags_query {
        params = params.tags_query(tags_query);
    }
    if let Some(metrics_query) = metrics_query {
        params = params.metrics_query(metrics_query);
    }
    if let Some(limit) = limit {
        params = params.limit(limit);
    }
    if let Some(offset) = offset {
        params = params.offset(offset);
    }
    let resp = api
        .list_slos(params)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list SLOs: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn get(cfg: &Config, id: &str) -> Result<()> {
    let api = crate::make_api!(ServiceLevelObjectivesAPI, cfg);
    let resp = api
        .get_slo(id.to_string(), GetSLOOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to get SLO: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn create(cfg: &Config, file: &str) -> Result<()> {
    let body: ServiceLevelObjectiveRequest = util::read_json_file(file)?;
    let api = crate::make_api!(ServiceLevelObjectivesAPI, cfg);
    let resp = api
        .create_slo(body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to create SLO: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn update(cfg: &Config, id: &str, file: &str) -> Result<()> {
    let body: ServiceLevelObjective = util::read_json_file(file)?;
    let api = crate::make_api!(ServiceLevelObjectivesAPI, cfg);
    let resp = api
        .update_slo(id.to_string(), body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to update SLO: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn delete(cfg: &Config, id: &str) -> Result<()> {
    let api = crate::make_api!(ServiceLevelObjectivesAPI, cfg);
    let resp = api
        .delete_slo(id.to_string(), DeleteSLOOptionalParams::default())
        .await
        .map_err(|e| anyhow::anyhow!("failed to delete SLO: {e:?}"))?;
    formatter::output(cfg, &resp)
}

pub async fn status(cfg: &Config, id: &str, from_ts: i64, to_ts: i64) -> Result<()> {
    use datadog_api_client::datadogV2::api_service_level_objectives::{
        GetSloStatusOptionalParams, ServiceLevelObjectivesAPI as SloV2API,
    };

    let api = crate::make_api!(SloV2API, cfg);
    let resp = api
        .get_slo_status(
            id.to_string(),
            from_ts,
            to_ts,
            GetSloStatusOptionalParams::default(),
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to get SLO status: {e:?}"))?;
    formatter::output(cfg, &resp)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_slos_list() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let mock = server
            .mock("GET", "/api/v1/slo")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": [], "errors": []}"#)
            .create_async()
            .await;

        let result = super::list(&cfg, None, None, None, None, None).await;
        assert!(result.is_ok(), "slos list failed: {:?}", result.err());
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_slos_list_with_query() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let mock = server
            .mock("GET", "/api/v1/slo")
            .match_query(mockito::Matcher::UrlEncoded(
                "query".into(),
                "monitor-history-reader".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": [], "errors": []}"#)
            .create_async()
            .await;

        let result = super::list(
            &cfg,
            Some("monitor-history-reader".into()),
            None,
            None,
            None,
            None,
        )
        .await;
        assert!(
            result.is_ok(),
            "slos list with query failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_slos_list_with_tags_query() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let mock = server
            .mock("GET", "/api/v1/slo")
            .match_query(mockito::Matcher::UrlEncoded(
                "tags_query".into(),
                "team:slo-app".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": [], "errors": []}"#)
            .create_async()
            .await;

        let result = super::list(&cfg, None, Some("team:slo-app".into()), None, None, None).await;
        assert!(
            result.is_ok(),
            "slos list with tags_query failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_slos_list_with_limit_and_offset() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let mock = server
            .mock("GET", "/api/v1/slo")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("limit".into(), "25".into()),
                mockito::Matcher::UrlEncoded("offset".into(), "50".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": [], "errors": []}"#)
            .create_async()
            .await;

        let result = super::list(&cfg, None, None, None, Some(25), Some(50)).await;
        assert!(
            result.is_ok(),
            "slos list with pagination failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_slos_list_with_metrics_query() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let mock = server
            .mock("GET", "/api/v1/slo")
            .match_query(mockito::Matcher::UrlEncoded(
                "metrics_query".into(),
                "sum:requests.error{service:api}".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": [], "errors": []}"#)
            .create_async()
            .await;

        let result = super::list(
            &cfg,
            None,
            None,
            Some("sum:requests.error{service:api}".into()),
            None,
            None,
        )
        .await;
        assert!(
            result.is_ok(),
            "slos list with metrics_query failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_slos_list_api_error() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let mock = server
            .mock("GET", "/api/v1/slo")
            .match_query(mockito::Matcher::UrlEncoded("query".into(), "team".into()))
            .with_status(500)
            .with_header("content-type", "application/json")
            .with_body(r#"{"errors":["boom"]}"#)
            .create_async()
            .await;

        let result = super::list(&cfg, Some("team".into()), None, None, None, None).await;
        assert!(
            result.is_err(),
            "slos list error path unexpectedly succeeded"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("failed to list SLOs"),
            "slos list error did not contain context"
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_slos_get() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(
            &mut server,
            "GET",
            r#"{"data": {"id": "abc123", "name": "Test SLO", "type": "metric", "thresholds": [{"timeframe": "7d", "target": 99.9}]}, "errors": []}"#,
        )
        .await;

        let result = super::get(&cfg, "abc123").await;
        assert!(result.is_ok(), "slos get failed: {:?}", result.err());
        cleanup_env();
    }

    #[tokio::test]
    async fn test_slos_delete() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());
        let _mock = mock_any(&mut server, "DELETE", r#"{"data": []}"#).await;

        let result = super::delete(&cfg, "abc123").await;
        assert!(result.is_ok(), "slos delete failed: {:?}", result.err());
        cleanup_env();
    }
}
