use std::collections::HashMap;
use std::path::Path;
use anyhow::{Context, Result};
use serde_json::Value;
use tokio::fs;

pub async fn transform_blocks(contents: &Path) -> Result<HashMap<String, i64>> {
    let blocks_contents = fs::read_to_string(&contents).await.context("failed to read blocks")?;
    let blocks: HashMap<String, Value> = serde_json::from_str(&blocks_contents).context("failed to parse blocks")?;
    let mut transformed_blocks = HashMap::new();
    for (block_id, properties) in blocks {
        let protocol_id: i64 = match properties.get("states").and_then(Value::as_array) {
            Some(states) => states
                .iter()
                .find(|state| state
                    .get("default")
                    .and_then(Value::as_bool)
                    .unwrap_or(false))
                .and_then(|state| state.get("id").and_then(Value::as_i64))
                .context("no default state with an integer id")?,
            None => continue,
        };
        transformed_blocks.insert(block_id, protocol_id);
    }

    Ok(transformed_blocks)
}