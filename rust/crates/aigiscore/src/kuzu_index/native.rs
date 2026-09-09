//! In-process Kuzu execution. JSON projection remains independent of the database runtime.

use super::{KuzuIndexError, KuzuQueryOutput};
use kuzu::{Connection, Database, SystemConfig, Value};
use serde_json::{json, Map, Number, Value as Json};
use std::collections::HashSet;
use std::path::Path;

fn config(read_only: bool) -> SystemConfig {
    SystemConfig::default()
        .read_only(read_only)
        .buffer_pool_size(512 * 1024 * 1024)
        .max_num_threads(4)
}

pub(crate) fn materialize(db: &Path, nodes: &Path, relations: &Path) -> Result<(), KuzuIndexError> {
    let db = Database::new(db, config(false))?;
    let connection = Connection::new(&db)?;
    connection.query("CREATE NODE TABLE CodeNode(id STRING, kind STRING, name STRING, qualifiedName STRING, filePath STRING, language STRING, parentSymbolId STRING, ownerTypeName STRING, returnTypeName STRING, visibility STRING, parameterCount INT64, requiredParameterCount INT64, startLine INT64, endLine INT64, PRIMARY KEY(id))")?;
    connection.query("CREATE REL TABLE CodeRelation(FROM CodeNode TO CodeNode, type STRING, referenceKind STRING, relationKind STRING, layer STRING, strength STRING, origin STRING, resolutionTier STRING, confidenceMillis INT64, reason STRING, line INT64, occurrenceCount INT64)")?;
    for (table, path, endpoints) in [
        ("CodeNode", nodes, ""),
        (
            "CodeRelation",
            relations,
            "from='CodeNode', to='CodeNode', ",
        ),
    ] {
        // Kuzu string literals interpret backslash escapes, including escaped quotes.
        let path = path
            .to_str()
            .ok_or_else(|| KuzuIndexError::Invalid("Kuzu CSV path is not UTF-8".into()))?
            .replace('\\', "\\\\")
            .replace('\'', "\\'");
        connection.query(&format!("COPY {table} FROM '{path}' ({endpoints}HEADER=true, ESCAPE='\"', DELIM=',', QUOTE='\"', PARALLEL=false, auto_detect=false)"))?;
    }
    connection.query("CHECKPOINT")?;
    Ok(())
}

pub fn query_kuzu(path: &Path, cypher: &str) -> Result<KuzuQueryOutput, KuzuIndexError> {
    crate::artifacts::kuzu::verify_kuzu_pin(path)?;
    let db = Database::new(path, config(true))?;
    let connection = Connection::new(&db)?;
    // prepare rejects multiple statements; the Rust result API exposes only one.
    let mut statement = connection.prepare(cypher)?;
    let result = connection.execute(&mut statement, vec![])?;
    let columns = result.get_column_names();
    let mut names = HashSet::new();
    if columns.iter().any(|name| !names.insert(name)) {
        return Err(KuzuIndexError::Invalid(
            "duplicate result columns; use distinct AS aliases".into(),
        ));
    }
    // The upstream iterator unwraps temporal conversions outside time's range.
    // Turn that reachable Rust panic into an explicit query failure.
    let rows = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        result
            .map(|row| {
                if row.len() != columns.len() {
                    return Err(KuzuIndexError::Invalid(
                        "Kuzu row width differs from column schema".into(),
                    ));
                }
                columns
                    .iter()
                    .cloned()
                    .zip(row)
                    .map(|(name, value)| value_json(&value).map(|value| (name, value)))
                    .collect::<Result<Map<String, Json>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()
    }))
    .map_err(|_| {
        KuzuIndexError::Invalid("native binding could not decode the query result".into())
    })??;
    Ok(KuzuQueryOutput {
        db_path: path.to_path_buf(),
        row_count: rows.len(),
        columns,
        rows,
    })
}

fn properties(values: &[(String, Value)]) -> Result<Json, KuzuIndexError> {
    values
        .iter()
        .map(|(name, value)| value_json(value).map(|value| (name.clone(), value)))
        .collect::<Result<Map<_, _>, _>>()
        .map(Json::Object)
}

fn internal_id(id: &kuzu::InternalID) -> Json {
    json!({"table": id.table_id, "offset": id.offset})
}

fn node_json(node: &kuzu::NodeVal) -> Result<Json, KuzuIndexError> {
    Ok(
        json!({"_id": internal_id(node.get_node_id()), "_label": node.get_label_name(), "properties": properties(node.get_properties())?}),
    )
}

fn rel_json(rel: &kuzu::RelVal) -> Result<Json, KuzuIndexError> {
    Ok(
        json!({"_src": internal_id(rel.get_src_node()), "_dst": internal_id(rel.get_dst_node()), "_label": rel.get_label_name(), "properties": properties(rel.get_properties())?}),
    )
}

fn finite(value: f64) -> Result<Json, KuzuIndexError> {
    Number::from_f64(value).map(Json::Number).ok_or_else(|| {
        KuzuIndexError::Invalid(
            "non-finite query result cannot be represented as a JSON number".into(),
        )
    })
}

fn value_json(value: &Value) -> Result<Json, KuzuIndexError> {
    Ok(match value {
        Value::Null(_) => Json::Null,
        Value::Bool(v) => json!(v),
        Value::Int8(v) => json!(v),
        Value::Int16(v) => json!(v),
        Value::Int32(v) => json!(v),
        Value::Int64(v) => json!(v),
        Value::UInt8(v) => json!(v),
        Value::UInt16(v) => json!(v),
        Value::UInt32(v) => json!(v),
        Value::UInt64(v) => json!(v),
        // Preserve precision across JSON implementations without silently rounding.
        Value::Int128(v) => json!(v.to_string()),
        Value::Decimal(v) => json!(v.to_string()),
        Value::Double(v) => finite(*v)?,
        Value::Float(v) => finite(f64::from(*v))?,
        Value::String(v) => json!(v),
        Value::UUID(v) => json!(v.to_string()),
        Value::Date(v) => json!(v.to_string()),
        Value::Interval(v) => json!({"nanoseconds": v.whole_nanoseconds().to_string()}),
        Value::Timestamp(v)
        | Value::TimestampTz(v)
        | Value::TimestampNs(v)
        | Value::TimestampMs(v)
        | Value::TimestampSec(v) => json!(v.to_string()),
        Value::InternalID(v) => internal_id(v),
        Value::Blob(v) => json!(v),
        Value::List(_, values) | Value::Array(_, values) => {
            Json::Array(values.iter().map(value_json).collect::<Result<_, _>>()?)
        }
        Value::Struct(values) => properties(values)?,
        Value::Map(_, values) => Json::Array(
            values
                .iter()
                .map(|(key, value)| {
                    Ok(json!({"key": value_json(key)?, "value": value_json(value)?}))
                })
                .collect::<Result<_, KuzuIndexError>>()?,
        ),
        Value::Union { value, .. } => value_json(value)?,
        Value::Node(v) => node_json(v)?,
        Value::Rel(v) => rel_json(v)?,
        Value::RecursiveRel { nodes, rels } => json!({
            "_nodes": nodes.iter().map(node_json).collect::<Result<Vec<_>, _>>()?,
            "_rels": rels.iter().map(rel_json).collect::<Result<Vec<_>, _>>()?,
        }),
    })
}
