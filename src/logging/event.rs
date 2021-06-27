use anyhow::Result;
use slog::{OwnedKVList, KV};

use super::ser::KubewardenLogSerializer;
use serde_json::json;

pub(crate) fn new(
    rinfo: &slog::Record,
    logger_values: &OwnedKVList,
) -> Result<serde_json::Map<String, serde_json::Value>> {
    let level: String = String::from(match rinfo.level() {
        slog::Level::Debug => "debug",
        slog::Level::Info => "info",
        slog::Level::Warning => "warning",
        slog::Level::Error => "error",
        // map the following levels to "error"
        slog::Level::Trace => "error",
        slog::Level::Critical => "error",
    });

    let mut serializer = KubewardenLogSerializer::start()?;
    let mut field_serializer = serializer.field_serializer();
    rinfo.kv().serialize(rinfo, &mut field_serializer)?;
    logger_values.serialize(rinfo, &mut field_serializer)?;
    let mut data = serializer.end()?;

    data.insert(String::from("level"), json!(level));
    data.insert(String::from("message"), json!(format!("{}", rinfo.msg())));
    data.insert(String::from("line"), json!(rinfo.line()));
    data.insert(String::from("column"), json!(rinfo.column()));
    data.insert(String::from("file"), json!(rinfo.file()));

    Ok(data)
}
