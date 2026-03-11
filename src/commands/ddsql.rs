use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::client;
use crate::config::{Config, OutputFormat};
use crate::formatter;
use crate::util;

/// Build a JSON:API request envelope for DDSQL endpoints.
fn build_request(
    jsonapi_type: &str,
    query: &str,
    from: &str,
    to: &str,
    interval: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<Value> {
    let default_start =
        util::parse_time_to_unix_millis(from).map_err(|e| anyhow!("invalid --from: {e}"))?;
    let default_end =
        util::parse_time_to_unix_millis(to).map_err(|e| anyhow!("invalid --to: {e}"))?;

    let default_interval = interval.unwrap_or(60000);

    let mut attrs = json!({
        "query": query,
        "default_start": default_start,
        "default_end": default_end,
        "default_interval": default_interval,
    });

    if let Some(l) = limit {
        attrs["limit"] = json!(l);
    }
    if let Some(o) = offset {
        attrs["offset"] = json!(o);
        attrs["enable_pagination"] = json!(true);
    }

    Ok(json!({
        "data": {
            "type": jsonapi_type,
            "attributes": attrs,
        }
    }))
}

pub async fn table(
    cfg: &Config,
    query: &str,
    from: &str,
    to: &str,
    interval: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<()> {
    let body = build_request(
        "ddsql_table_request",
        query,
        from,
        to,
        interval,
        limit,
        offset,
    )?;
    let data = client::raw_post(cfg, "/api/v2/ddsql/table", body).await?;
    formatter::output(cfg, &data)
}

pub async fn csv(
    cfg: &Config,
    query: &str,
    from: &str,
    to: &str,
    interval: Option<i64>,
    limit: Option<i64>,
    offset: Option<i64>,
) -> Result<()> {
    let body = build_request(
        "ddsql_table_request",
        query,
        from,
        to,
        interval,
        limit,
        offset,
    )?;
    let data = client::raw_post(cfg, "/api/v2/ddsql/csv", body).await?;

    // Default format is "json", which we treat as "no explicit format" — print raw CSV.
    if cfg.output_format == OutputFormat::Json && !cfg.agent_mode {
        let csv_str = extract_csv(&data)?;
        print!("{csv_str}");
        Ok(())
    } else {
        formatter::output(cfg, &data)
    }
}

pub async fn time_series(
    cfg: &Config,
    query: &str,
    from: &str,
    to: &str,
    interval: Option<i64>,
) -> Result<()> {
    let body = build_request(
        "ddsql_timeseries_request",
        query,
        from,
        to,
        interval,
        None,
        None,
    )?;
    let data = client::raw_post(cfg, "/api/v2/ddsql/time_series", body).await?;
    formatter::output(cfg, &data)
}

/// Extract raw CSV string from the JSON:API response.
fn extract_csv(resp: &Value) -> Result<String> {
    // Try array shape first: {"data": [{"attributes": {"csv": "..."}}]}
    if let Some(csv) = resp
        .pointer("/data/0/attributes/csv")
        .and_then(Value::as_str)
    {
        return Ok(csv.to_string());
    }
    // Fallback to object shape: {"data": {"attributes": {"csv": "..."}}}
    if let Some(csv) = resp.pointer("/data/attributes/csv").and_then(Value::as_str) {
        return Ok(csv.to_string());
    }
    Err(anyhow!(
        "could not extract CSV from response:\n{}",
        serde_json::to_string_pretty(resp).unwrap_or_else(|_| resp.to_string())
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_table_all_flags() {
        let req = build_request(
            "ddsql_table_request",
            "SELECT 1",
            "1h",
            "now",
            Some(300000),
            Some(10),
            Some(20),
        )
        .unwrap();

        assert_eq!(req["data"]["type"], "ddsql_table_request");
        assert_eq!(req["data"]["attributes"]["default_interval"], 300000);
        assert_eq!(req["data"]["attributes"]["limit"], 10);
        assert_eq!(req["data"]["attributes"]["offset"], 20);
        assert_eq!(req["data"]["attributes"]["enable_pagination"], true);
        assert!(req["data"]["attributes"]["default_start"].is_i64());
        assert!(req["data"]["attributes"]["default_end"].is_i64());

        let start = req["data"]["attributes"]["default_start"].as_i64().unwrap();
        let end = req["data"]["attributes"]["default_end"].as_i64().unwrap();
        let now_ms = chrono::Utc::now().timestamp() * 1000;
        assert!((end - now_ms).abs() < 2000);
        assert!((start - (now_ms - 3600000)).abs() < 2000);
    }

    #[test]
    fn test_build_table_defaults() {
        let req = build_request(
            "ddsql_table_request",
            "SELECT 1",
            "1h",
            "now",
            None,
            None,
            None,
        )
        .unwrap();

        assert_eq!(req["data"]["type"], "ddsql_table_request");
        assert_eq!(req["data"]["attributes"]["default_interval"], 60000);
        assert!(
            req["data"]["attributes"].get("limit").is_none()
                || req["data"]["attributes"]["limit"].is_null()
        );
        assert!(
            req["data"]["attributes"].get("offset").is_none()
                || req["data"]["attributes"]["offset"].is_null()
        );
        assert!(
            req["data"]["attributes"].get("enable_pagination").is_none()
                || req["data"]["attributes"]["enable_pagination"].is_null()
        );
    }

    #[test]
    fn test_build_timeseries_type() {
        let req = build_request(
            "ddsql_timeseries_request",
            "SELECT avg(cpu) FROM metrics",
            "1h",
            "now",
            Some(60000),
            None,
            None,
        )
        .unwrap();

        assert_eq!(req["data"]["type"], "ddsql_timeseries_request");
    }

    #[test]
    fn test_build_csv_no_interval() {
        let req = build_request(
            "ddsql_table_request",
            "SELECT 1",
            "1h",
            "now",
            None,
            Some(50),
            None,
        )
        .unwrap();

        assert_eq!(req["data"]["type"], "ddsql_table_request");
        assert_eq!(req["data"]["attributes"]["default_interval"], 60000);
        assert_eq!(req["data"]["attributes"]["limit"], 50);
    }

    #[test]
    fn test_build_invalid_from() {
        let err = build_request(
            "ddsql_table_request",
            "SELECT 1",
            "garbage",
            "now",
            None,
            None,
            None,
        )
        .unwrap_err();

        assert!(err.to_string().contains("invalid --from"));
    }

    #[test]
    fn test_extract_csv_valid_array() {
        let resp: Value =
            serde_json::from_str(r#"{"data":[{"attributes":{"csv":"a,b\n1,2"}}]}"#).unwrap();
        assert_eq!(extract_csv(&resp).unwrap(), "a,b\n1,2");
    }

    #[test]
    fn test_extract_csv_valid_object() {
        let resp: Value =
            serde_json::from_str(r#"{"data":{"attributes":{"csv":"x,y\n3,4"}}}"#).unwrap();
        assert_eq!(extract_csv(&resp).unwrap(), "x,y\n3,4");
    }

    #[test]
    fn test_extract_csv_empty_array() {
        let resp: Value = serde_json::from_str(r#"{"data":[]}"#).unwrap();
        assert!(extract_csv(&resp).is_err());
    }

    #[test]
    fn test_extract_csv_missing_key() {
        let resp: Value = serde_json::from_str(r#"{"data":[{"attributes":{}}]}"#).unwrap();
        assert!(extract_csv(&resp).is_err());
    }

    #[test]
    fn test_extract_csv_no_attributes() {
        let resp: Value = serde_json::from_str(r#"{"data":[{}]}"#).unwrap();
        assert!(extract_csv(&resp).is_err());
    }
}
