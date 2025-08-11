use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameVersionLatest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameVersionEntry {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    // TODO: other fields
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameVersionsResponse {
    pub latest: GameVersionLatest,
    pub versions: Vec<GameVersionEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameVersionDownloadEntry {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameVersionDownloads {
    // TODO: other fields
    pub server: GameVersionDownloadEntry,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GameVersionMetadata {
    // TODO: other fields
    pub downloads: GameVersionDownloads,
    pub id: String,
}
