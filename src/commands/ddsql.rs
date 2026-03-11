use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::client;
use crate::config::Config;
use crate::formatter;
use crate::util;

/// Build a JSON:API request envelope for DDSQL endpoints.
fn build_request(
    jsonapi_type: &str,
    query: &str,
    from: &str,
    to: &str,
    interval: Option<i64>,
    limit: Option<i32>,
    offset: Option<i32>,
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
    limit: Option<i32>,
    offset: Option<i32>,
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
    let rows = columnar_to_rows(&data)?;
    formatter::output(cfg, &rows)
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

/// Transform a DDSQL columnar response into a row-based JSON array.
///
/// The table endpoint returns columns in one of two shapes:
///   Array: {"data": [{"attributes": {"columns": [...]}}]}
///   Object: {"data": {"attributes": {"columns": [...]}}}
///
/// Each column is: {"name": "col1", "values": ["a", "b"]}
///
/// This transforms it to: [{"col1": "a", "col2": 1}, {"col1": "b", "col2": 2}]
fn columnar_to_rows(resp: &Value) -> Result<Value> {
    // Try array shape first (observed in production), then object shape.
    let columns = resp
        .pointer("/data/0/attributes/columns")
        .or_else(|| resp.pointer("/data/attributes/columns"))
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("unexpected response: missing columns in response"))?;

    if columns.is_empty() {
        return Ok(json!([]));
    }

    let num_rows = columns[0]
        .get("values")
        .and_then(Value::as_array)
        .map(|v| v.len())
        .unwrap_or(0);

    let mut rows = Vec::with_capacity(num_rows);
    for i in 0..num_rows {
        let mut row = serde_json::Map::new();
        for col in columns {
            let name = col.get("name").and_then(Value::as_str).unwrap_or("unknown");
            let value = col
                .get("values")
                .and_then(Value::as_array)
                .and_then(|vals| vals.get(i))
                .cloned()
                .unwrap_or(Value::Null);
            row.insert(name.to_string(), value);
        }
        rows.push(Value::Object(row));
    }

    Ok(Value::Array(rows))
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
    fn test_columnar_to_rows_array_shape() {
        // Actual production shape: {"data": [{"attributes": {"columns": [...]}}]}
        let resp: Value = serde_json::from_str(
            r#"{"data":[{"attributes":{"columns":[
                {"name":"host","type":"string","values":["h1","h2"]},
                {"name":"cpu","type":"float64","values":[10,20]}
            ]},"id":"ddsql_response","type":"scalar_response"}]}"#,
        )
        .unwrap();
        let rows = columnar_to_rows(&resp).unwrap();
        let arr = rows.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["host"], "h1");
        assert_eq!(arr[0]["cpu"], 10);
        assert_eq!(arr[1]["host"], "h2");
        assert_eq!(arr[1]["cpu"], 20);
    }

    #[test]
    fn test_columnar_to_rows_object_shape() {
        // Fallback shape: {"data": {"attributes": {"columns": [...]}}}
        let resp: Value = serde_json::from_str(
            r#"{"data":{"attributes":{"columns":[
                {"name":"id","values":[42]}
            ]}}}"#,
        )
        .unwrap();
        let rows = columnar_to_rows(&resp).unwrap();
        let arr = rows.as_array().unwrap();
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["id"], 42);
    }

    #[test]
    fn test_columnar_to_rows_empty_columns() {
        let resp: Value =
            serde_json::from_str(r#"{"data":[{"attributes":{"columns":[]}}]}"#).unwrap();
        let rows = columnar_to_rows(&resp).unwrap();
        assert_eq!(rows, json!([]));
    }

    #[test]
    fn test_columnar_to_rows_missing_columns() {
        let resp: Value = serde_json::from_str(r#"{"data":[{"attributes":{}}]}"#).unwrap();
        assert!(columnar_to_rows(&resp).is_err());
    }

    #[test]
    fn test_columnar_to_rows_null_values() {
        let resp: Value = serde_json::from_str(
            r#"{"data":[{"attributes":{"columns":[
                {"name":"a","values":[1,null]},
                {"name":"b","values":[null,"x"]}
            ]}}]}"#,
        )
        .unwrap();
        let rows = columnar_to_rows(&resp).unwrap();
        let arr = rows.as_array().unwrap();
        assert_eq!(arr[0]["a"], 1);
        assert!(arr[0]["b"].is_null());
        assert!(arr[1]["a"].is_null());
        assert_eq!(arr[1]["b"], "x");
    }
}
