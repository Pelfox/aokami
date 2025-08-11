use std::cmp::Ordering;
use std::env;
use std::env::current_dir;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;

use anyhow::{Context, Result};
use clap::Parser;
use reqwest::{Client, ClientBuilder};
use tokio::fs::{create_dir_all, read_dir, File};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use crate::cli::{CliArgs, ReleaseType, Subcommand, TransformSubcommand};
use crate::types::{GameVersionMetadata, GameVersionsResponse};

mod types;
mod transform;
mod cli;

async fn get_work_dir(dir: &str, create: bool) -> Result<PathBuf> {
    let mut work_dir = current_dir().context("failed to get current directory")?;
    work_dir.push(dir);
    if !work_dir.exists() && create {
        create_dir_all(&work_dir).await?;
    }
    Ok(work_dir)
}

async fn get_versions(client: &Client) -> Result<GameVersionsResponse> {
    client
        .get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .send()
        .await
        .context("failed to request game manifest")?
        .json()
        .await
        .context("failed to extract manifest json")
}

async fn download_server(client: &Client, versions_dir: &Path, url: &str) -> Result<PathBuf> {
    let metadata: GameVersionMetadata = client.get(url).send().await?.json().await?;

    let file_path = versions_dir.join(format!("{}.jar", metadata.id));
    if file_path.exists() {
        eprintln!("Server file already exists.");
        return Ok(file_path)
    }

    let mut server_file = File::create(&file_path).await?;
    let mut stream = client.get(metadata.downloads.server.url).send().await?.bytes_stream();
    while let Some(item) = stream.next().await {
        server_file.write_all(&item?).await?;
    }

    server_file.flush().await?;
    Ok(file_path)
}

async fn build_java_command<'a>(jar: &'a str, initial_args: &'a str) -> Result<(String, Vec<&'a str>)> {
    let java_home = env::var("JAVA_HOME").context("JAVA_HOME is not set")?;
    let args = vec![
        "-DbundlerMainClass=net.minecraft.data.Main",
        "-jar",
        &jar,
        initial_args
    ];
    let java_path = format!("{}/bin/java", java_home);
    Ok((java_path, args))
}

#[derive(Debug, Eq, PartialEq)]
struct Version(u32, u32, u32);

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        (self.0, self.1, self.2).cmp(&(other.0, other.1, other.2))
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.0, self.1, self.2)
    }
}

async fn find_latest_version(versions_dir: &PathBuf) -> Result<String> {
    let mut versions = Vec::new();
    let mut dir_contents = read_dir(versions_dir).await?;
    while let Some(entry) = dir_contents.next_entry().await? {
        let path = entry.path();
        let file_stem = path.file_stem().unwrap().display().to_string();
        let parts = file_stem.split('.').collect::<Vec<&str>>();

        let version = Version(parts[0].parse()?, parts[1].parse()?, parts[2].parse()?);
        versions.push(version);
    }
    versions.sort();
    Ok(versions.last().unwrap().to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = CliArgs::parse();

    let versions_dir = get_work_dir(args.versions_dir.as_str(), true).await?;
    let output_dir = get_work_dir(args.output_dir.as_str(), true).await?;

    match args.command {
        Subcommand::Download { r#type: release_type, version } => {
            let client = ClientBuilder::new()
                .user_agent("Aokami")
                .build()
                .context("failed to build request client")?; // TODO: include crate version

            let versions = get_versions(&client)
                .await
                .context("failed to get game versions")?;

            let selected_version_entry = versions.versions.iter().find(|v| {
                if version == "latest" {
                    if release_type == ReleaseType::Release {
                        v.id == versions.latest.release
                    } else {
                        v.id == versions.latest.snapshot
                    }
                } else {
                    v.id == version
                }
            });
            if let Some(version) = selected_version_entry {
                println!("Downloading server for {} ({})...", version.id, version.version_type);
                download_server(&client, &versions_dir, &version.url).await?;
            } else {
                eprintln!("Failed to find a version to download.");
            }
        },
        Subcommand::Generate { version, generator_args } => {
            let mut target_version = version;
            if target_version == "latest" {
                target_version = find_latest_version(&versions_dir).await?;
                println!("Selected version: {}", target_version);
            }
            let target_version_path = versions_dir.join(format!("{}.jar", target_version));
            let (java_path, args) = build_java_command(target_version_path.to_str().unwrap(), generator_args.as_str()).await?;
            let status = Command::new(java_path)
                .args(args)
                .current_dir(output_dir.clone())
                .status()
                .await?;
            println!("Generation done with status {}", status.code().unwrap_or(0));
        },
        Subcommand::Transform { sub } => {
            match sub {
                TransformSubcommand::Registry { output_file, registries } => {
                    let transformed_registries = transform::transform_registries(&output_dir.join("generated").join("data"), registries).await?;
                    let registries_file = transform::get_output_path(&output_dir).join(output_file);
                    create_dir_all(&registries_file.parent().unwrap()).await?;
                    let processed_registries_contents = serde_json::to_string_pretty(&transformed_registries)?;
                    tokio::fs::write(&registries_file, processed_registries_contents).await?;
                    println!("Transformed {} registries. Written into {}.", transformed_registries.len(), registries_file.display());
                },
                TransformSubcommand::Blocks { output_file } => {
                    let transformed_blocks = transform::transform_blocks(&output_dir.join("generated").join("reports").join("blocks.json")).await?;
                    let blocks_file = transform::get_output_path(&output_dir).join(output_file);
                    create_dir_all(&blocks_file.parent().unwrap()).await?;
                    let processed_registries_contents = serde_json::to_string_pretty(&transformed_blocks)?;
                    tokio::fs::write(&blocks_file, processed_registries_contents).await?;
                    println!("Transformed {} blocks. Written into {}.", transformed_blocks.len(), blocks_file.display());
                }
            }
        },
    }

    Ok(())
}
