use std::collections::HashMap;
use std::path::Path;
use anyhow::{Context, Result};
use serde_json::Value;

pub async fn transform_registries(root_dir: &Path, registries: Vec<String>) -> Result<HashMap<String, HashMap<String, Value>>> {
    let mut transformed_registries = HashMap::new();
    for registry in registries {
        let root_registry_dir = root_dir.join("minecraft").join(&registry);
        let mut contents = tokio::fs::read_dir(root_registry_dir)
            .await
            .context("failed to read registry directory")?;

        let mut registry_entries = HashMap::new();
        while let Some(entry) = contents.next_entry().await? {
            let entry_path = entry.path();
            if !entry_path.is_file() {
                println!("Encountered a non-file registry: {}. It will be skipped.", entry_path.display());
                println!("Hint: if you want to transform a registry like worldgen/biome, include its name as is.");
                continue;
            }

            let entry_contents = tokio::fs::read_to_string(&entry_path).await?;
            let registry: Value = serde_json::from_str(&entry_contents)?;

            let entry_registry_name = entry_path.file_stem().and_then(|s| s.to_str()).unwrap();
            registry_entries.insert(format!("minecraft:{}", entry_registry_name), registry);
        }
        transformed_registries.insert(format!("minecraft:{}", registry), registry_entries);
    }
    Ok(transformed_registries)
}
