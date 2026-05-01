use anyhow::Result;

use crate::client;
use crate::config::Config;
use crate::formatter;
use crate::util;

pub async fn services_list(cfg: &Config, env: String, from: String, to: String) -> Result<()> {
    let from_ts = util::parse_time_to_unix(&from)?;
    let to_ts = util::parse_time_to_unix(&to)?;
    let path = format!("/api/v2/apm/services?start={from_ts}&end={to_ts}&filter[env]={env}");
    let data = client::raw_get(cfg, &path, &[]).await?;
    formatter::output(cfg, &data)
}

pub async fn services_stats(cfg: &Config, env: String, from: String, to: String) -> Result<()> {
    let from_ts = util::parse_time_to_unix(&from)?;
    let to_ts = util::parse_time_to_unix(&to)?;
    let path = format!("/api/v2/apm/services/stats?start={from_ts}&end={to_ts}&filter[env]={env}");
    let data = client::raw_get(cfg, &path, &[]).await?;
    formatter::output(cfg, &data)
}

pub async fn entities_list(cfg: &Config, from: String, to: String) -> Result<()> {
    let from_ts = util::parse_time_to_unix(&from)?;
    let to_ts = util::parse_time_to_unix(&to)?;
    let path = format!("/api/unstable/apm/entities?start={from_ts}&end={to_ts}");
    let data = client::raw_get(cfg, &path, &[]).await?;
    formatter::output(cfg, &data)
}

pub async fn dependencies_list(cfg: &Config, env: String, from: String, to: String) -> Result<()> {
    let from_ts = util::parse_time_to_unix(&from)?;
    let to_ts = util::parse_time_to_unix(&to)?;
    let path = format!("/api/v1/service_dependencies?start={from_ts}&end={to_ts}&env={env}");
    let data = client::raw_get(cfg, &path, &[]).await?;
    formatter::output(cfg, &data)
}

pub async fn services_operations(
    cfg: &Config,
    service: String,
    env: String,
    from: String,
    to: String,
) -> Result<()> {
    let from_ts = util::parse_time_to_unix(&from)?;
    let to_ts = util::parse_time_to_unix(&to)?;
    let path =
        format!("/api/v1/trace/operation_names/{service}?env={env}&start={from_ts}&end={to_ts}");
    let data = client::raw_get(cfg, &path, &[]).await?;
    formatter::output(cfg, &data)
}

pub async fn services_resources(
    cfg: &Config,
    service: String,
    name: String,
    env: String,
    from: String,
    to: String,
) -> Result<()> {
    let from_ts = util::parse_time_to_unix(&from)?;
    let to_ts = util::parse_time_to_unix(&to)?;
    let path = format!(
        "/api/ui/apm/resources?service={service}&name={name}&env={env}&from={from_ts}&to={to_ts}"
    );
    let data = client::raw_get(cfg, &path, &[]).await?;
    formatter::output(cfg, &data)
}

pub async fn flow_map(
    cfg: &Config,
    query: String,
    limit: i64,
    from: String,
    to: String,
) -> Result<()> {
    let from_ts = util::parse_time_to_unix(&from)?;
    let to_ts = util::parse_time_to_unix(&to)?;
    let path =
        format!("/api/ui/apm/flow-map?query={query}&limit={limit}&start={from_ts}&end={to_ts}");
    let data = client::raw_get(cfg, &path, &[]).await?;
    formatter::output(cfg, &data)
}

pub async fn troubleshooting_list(
    cfg: &Config,
    hostname: String,
    timeframe: Option<String>,
) -> Result<()> {
    let path = "/api/unstable/apm/instrumentation-errors";
    let mut query = vec![("hostname", hostname.as_str())];
    let tf_owned;
    if let Some(tf) = &timeframe {
        tf_owned = tf.clone();
        query.push(("timeframe", tf_owned.as_str()));
    }
    let data = client::raw_get(cfg, path, &query).await?;
    formatter::output(cfg, &data)
}

pub async fn service_config_get(
    cfg: &Config,
    service_name: String,
    env: Option<String>,
    service_instance_ids: Option<String>,
) -> Result<()> {
    let mut query = vec![("service_name", service_name.as_str())];
    let env_owned;
    if let Some(e) = &env {
        env_owned = e.clone();
        query.push(("env", env_owned.as_str()));
    }
    let ids_owned;
    if let Some(ids) = &service_instance_ids {
        ids_owned = ids.clone();
        query.push(("service_instance_ids", ids_owned.as_str()));
    }
    let data = client::raw_get(cfg, "/api/unstable/apm/service-config", &query).await?;
    formatter::output(cfg, &data)
}

pub async fn service_library_config_get(
    cfg: &Config,
    service_name: String,
    env: Option<String>,
    language: Option<String>,
    mixed: bool,
) -> Result<()> {
    let mut query = vec![("service_name", service_name.as_str())];
    let env_owned;
    if let Some(e) = &env {
        env_owned = e.clone();
        query.push(("env", env_owned.as_str()));
    }
    let lang_owned;
    if let Some(l) = &language {
        lang_owned = l.clone();
        query.push(("language_name", lang_owned.as_str()));
    }
    if mixed {
        query.push(("is_mixed", "true"));
    }
    let data = client::raw_get(cfg, "/api/unstable/apm/service-library-config", &query).await?;
    formatter::output(cfg, &data)
}

#[cfg(test)]
mod tests {

    use crate::test_support::*;

    #[tokio::test]
    async fn test_apm_services_list() {
        let _lock = lock_env().await;
        let mut s = mockito::Server::new_async().await;
        let cfg = test_config(&s.url());
        mock_all(&mut s, r#"{"data": []}"#).await;
        let _ = super::services_list(&cfg, "prod".into(), "1h".into(), "now".into()).await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_apm_services_resources_uses_from_to() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());

        let mock = server
            .mock("GET", "/api/ui/apm/resources")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("service".into(), "web".into()),
                mockito::Matcher::UrlEncoded("name".into(), "http.request".into()),
                mockito::Matcher::UrlEncoded("env".into(), "prod".into()),
                mockito::Matcher::Regex("from=".into()),
                mockito::Matcher::Regex("to=".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": []}"#)
            .create_async()
            .await;

        let result = super::services_resources(
            &cfg,
            "web".into(),
            "http.request".into(),
            "prod".into(),
            "1h".into(),
            "now".into(),
        )
        .await;
        assert!(
            result.is_ok(),
            "services_resources failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_apm_troubleshooting_list() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());

        let mock = server
            .mock("GET", "/api/unstable/apm/instrumentation-errors")
            .match_query(mockito::Matcher::UrlEncoded(
                "hostname".into(),
                "my-host".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": []}"#)
            .create_async()
            .await;

        let result = super::troubleshooting_list(&cfg, "my-host".into(), None).await;
        assert!(
            result.is_ok(),
            "troubleshooting list failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_apm_troubleshooting_list_with_timeframe() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());

        let mock = server
            .mock("GET", "/api/unstable/apm/instrumentation-errors")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("hostname".into(), "my-host".into()),
                mockito::Matcher::UrlEncoded("timeframe".into(), "4h".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data": []}"#)
            .create_async()
            .await;

        let result = super::troubleshooting_list(&cfg, "my-host".into(), Some("4h".into())).await;
        assert!(
            result.is_ok(),
            "troubleshooting list with timeframe failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_apm_service_config_get() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());

        let mock = server
            .mock("GET", "/api/unstable/apm/service-config")
            .match_query(mockito::Matcher::UrlEncoded(
                "service_name".into(),
                "my-service".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"service_name":"my-service","service_configs":[]}"#)
            .create_async()
            .await;

        let result = super::service_config_get(&cfg, "my-service".into(), None, None).await;
        assert!(
            result.is_ok(),
            "service_config_get failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_apm_service_config_get_with_filters() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());

        let mock = server
            .mock("GET", "/api/unstable/apm/service-config")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("service_name".into(), "my-service".into()),
                mockito::Matcher::UrlEncoded("env".into(), "prod".into()),
                mockito::Matcher::UrlEncoded("service_instance_ids".into(), "id-1,id-2".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"service_name":"my-service","service_configs":[]}"#)
            .create_async()
            .await;

        let result = super::service_config_get(
            &cfg,
            "my-service".into(),
            Some("prod".into()),
            Some("id-1,id-2".into()),
        )
        .await;
        assert!(
            result.is_ok(),
            "service_config_get with filters failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_apm_service_library_config_get() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());

        let mock = server
            .mock("GET", "/api/unstable/apm/service-library-config")
            .match_query(mockito::Matcher::UrlEncoded(
                "service_name".into(),
                "my-service".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"service_name":"my-service","configs":[]}"#)
            .create_async()
            .await;

        let result =
            super::service_library_config_get(&cfg, "my-service".into(), None, None, false).await;
        assert!(
            result.is_ok(),
            "service_library_config_get failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }

    #[tokio::test]
    async fn test_apm_service_library_config_get_with_filters() {
        let _lock = lock_env().await;
        let mut server = mockito::Server::new_async().await;
        let cfg = test_config(&server.url());

        let mock = server
            .mock("GET", "/api/unstable/apm/service-library-config")
            .match_query(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("service_name".into(), "my-service".into()),
                mockito::Matcher::UrlEncoded("env".into(), "prod".into()),
                mockito::Matcher::UrlEncoded("language_name".into(), "python".into()),
                mockito::Matcher::UrlEncoded("is_mixed".into(), "true".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"service_name":"my-service","is_mixed":true,"configs":[]}"#)
            .create_async()
            .await;

        let result = super::service_library_config_get(
            &cfg,
            "my-service".into(),
            Some("prod".into()),
            Some("python".into()),
            true,
        )
        .await;
        assert!(
            result.is_ok(),
            "service_library_config_get with filters failed: {:?}",
            result.err()
        );
        mock.assert_async().await;
        cleanup_env();
    }
}
