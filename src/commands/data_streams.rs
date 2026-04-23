//! Data Streams Monitoring (DSM) commands.
//!
//! These endpoints are backed by `/api/ui/data_streams/...` on dsm-api and are
//! considered **experimental** — the API surface is not covered by Datadog's
//! public API compatibility guarantees and may change without notice.
//!
//! Endpoints:
//!   • GET  /api/ui/data_streams/kafka_topic_configs
//!   • GET  /api/ui/data_streams/kafka_broker_configs
//!   • POST /api/ui/data_streams/kafka_client_configs
//!   • POST /api/ui/data_streams/kafka_actions/read_messages
//!
//! Authentication: OAuth2 bearer token (e.g. `pup auth login`). DD_API_KEY +
//! DD_APP_KEY is not accepted by these UI routes.

use anyhow::Result;
use serde_json::{json, Value};

use crate::client;
use crate::config::Config;
use crate::formatter;

const TOPIC_CONFIGS_PATH: &str = "/api/ui/data_streams/kafka_topic_configs";
const BROKER_CONFIGS_PATH: &str = "/api/ui/data_streams/kafka_broker_configs";
const CLIENT_CONFIGS_PATH: &str = "/api/ui/data_streams/kafka_client_configs";
const READ_MESSAGES_PATH: &str = "/api/ui/data_streams/kafka_actions/read_messages";
const ALL_KAFKA_SCHEMAS_PATH: &str = "/api/ui/data_streams/all_kafka_schemas";
const SUBJECT_KAFKA_SCHEMAS_PATH: &str = "/api/ui/data_streams/subject_kafka_schemas";

/// GET /api/ui/data_streams/kafka_topic_configs
pub async fn kafka_topic_configs(cfg: &Config, kafka_cluster_id: &str, topic: &str) -> Result<()> {
    let query = [("kafka_cluster_id", kafka_cluster_id), ("topic", topic)];
    let resp = client::raw_get(cfg, TOPIC_CONFIGS_PATH, &query)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get kafka topic configs: {e:?}"))?;
    formatter::output(cfg, &resp)
}

/// GET /api/ui/data_streams/kafka_broker_configs
pub async fn kafka_broker_configs(
    cfg: &Config,
    kafka_cluster_id: &str,
    broker_id: &str,
) -> Result<()> {
    let query = [
        ("kafka_cluster_id", kafka_cluster_id),
        ("broker_id", broker_id),
    ];
    let resp = client::raw_get(cfg, BROKER_CONFIGS_PATH, &query)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get kafka broker configs: {e:?}"))?;
    formatter::output(cfg, &resp)
}

/// POST /api/ui/data_streams/kafka_client_configs
///
/// `services` is a list of `service:config_type` pairs where `config_type` is
/// either `producer` or `consumer`.
pub async fn kafka_client_configs(
    cfg: &Config,
    kafka_cluster_id: &str,
    services: Vec<(String, String)>,
) -> Result<()> {
    if services.is_empty() {
        anyhow::bail!("at least one --service SERVICE:producer|consumer is required");
    }
    let services_json: Vec<Value> = services
        .into_iter()
        .map(|(service, config_type)| json!({ "service": service, "config_type": config_type }))
        .collect();
    let body = json!({
        "kafka_cluster_id": kafka_cluster_id,
        "services": services_json,
    });
    let resp = client::raw_post(cfg, CLIENT_CONFIGS_PATH, body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get kafka client configs: {e:?}"))?;
    formatter::output(cfg, &resp)
}

/// POST /api/ui/data_streams/kafka_actions/read_messages
///
/// Dispatches a Kafka read-messages action to the agent via Remote Config and
/// polls until the agent responds. The caller's org must have a Datadog Agent
/// reachable by Remote Config that can connect to the target cluster, and the
/// caller must have the `data_streams_capture_messages` permission. Calls are
/// rate-limited to 10 per minute per user by dsm-api.
#[allow(clippy::too_many_arguments)]
pub async fn read_messages(
    cfg: &Config,
    cluster: &str,
    topic: &str,
    bootstrap_servers: &str,
    partition: Option<i32>,
    start_offset: i64,
    start_timestamp: Option<i64>,
    n_messages_retrieved: u32,
    max_scanned_messages: u32,
    filter: Option<String>,
    value_format: Option<String>,
    key_format: Option<String>,
    consumer_group_id: Option<String>,
) -> Result<()> {
    let mut body = json!({
        "cluster": cluster,
        "topic": topic,
        "bootstrap_servers": bootstrap_servers,
        "start_offset": start_offset,
        "n_messages_retrieved": n_messages_retrieved,
        "max_scanned_messages": max_scanned_messages,
    });
    if let Some(p) = partition {
        body["partition"] = json!(p);
    }
    if let Some(ts) = start_timestamp {
        body["start_timestamp"] = json!(ts);
    }
    if let Some(f) = filter {
        body["filter"] = json!(f);
    }
    if let Some(vf) = value_format {
        body["value_format"] = json!(vf);
    }
    if let Some(kf) = key_format {
        body["key_format"] = json!(kf);
    }
    if let Some(cg) = consumer_group_id {
        body["consumer_group_id"] = json!(cg);
    }

    let resp = client::raw_post(cfg, READ_MESSAGES_PATH, body)
        .await
        .map_err(|e| anyhow::anyhow!("failed to read kafka messages: {e:?}"))?;
    formatter::output(cfg, &resp)
}

/// GET /api/ui/data_streams/all_kafka_schemas — every Kafka Schema Registry
/// schema known to DSM for the org within the given time window. `start_unix`
/// and `end_unix` are optional unix-seconds; when omitted dsm-api defaults to
/// roughly the last hour.
pub async fn all_kafka_schemas(
    cfg: &Config,
    start_unix: Option<i64>,
    end_unix: Option<i64>,
) -> Result<()> {
    let mut query: Vec<(&str, String)> = Vec::new();
    if let Some(s) = start_unix {
        query.push(("start_unix", s.to_string()));
    }
    if let Some(e) = end_unix {
        query.push(("end_unix", e.to_string()));
    }
    let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let resp = client::raw_get(cfg, ALL_KAFKA_SCHEMAS_PATH, &query_refs)
        .await
        .map_err(|e| anyhow::anyhow!("failed to list kafka schema registry schemas: {e:?}"))?;
    formatter::output(cfg, &resp)
}

/// GET /api/ui/data_streams/subject_kafka_schemas — all version history of a
/// single Schema Registry subject on a Kafka cluster.
pub async fn subject_kafka_schemas(
    cfg: &Config,
    kafka_cluster_id: &str,
    subject: &str,
) -> Result<()> {
    let query = [
        ("kafka_cluster_id", kafka_cluster_id),
        ("subject", subject),
    ];
    let resp = client::raw_get(cfg, SUBJECT_KAFKA_SCHEMAS_PATH, &query)
        .await
        .map_err(|e| anyhow::anyhow!("failed to get subject kafka schemas: {e:?}"))?;
    formatter::output(cfg, &resp)
}
