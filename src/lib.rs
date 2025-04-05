use serde::Deserialize;
use std::net::SocketAddr;
use thiserror::Error;

use a2s::{A2SClient, info::ExtendedServerInfo};

#[derive(Error, Debug)]
pub enum DayzMonitorError {
    #[error("Tokio IO error: {0}")]
    TokioIOError(#[from] tokio::io::Error),
    #[error("A2S error: {0}")]
    A2SError(#[from] a2s::errors::Error),
    #[error(
        "Failed to extract server info from a2s response, 'keywords' did not exist in response."
    )]
    ExtractServerInfoKeywordsNonExistant,
}

#[derive(Debug, Deserialize)]
pub struct DayzMonitorConfig {
    pub discord_token: String,
    pub server_address: SocketAddr,
    pub server_name: String,
    pub voice_channel_id: Option<u64>,
}

#[derive(Debug)]
pub struct ServerInfo {
    pub server_time: Option<String>,
    pub players_in_queue: Option<u32>,
    pub players: u32,
    pub max_players: u32,
}

pub async fn retrieve_server_info(
    client: &A2SClient,
    addr: SocketAddr,
) -> Result<ServerInfo, DayzMonitorError> {
    tracing::debug!("Retrieving current server info for '{addr}'");
    let info = client.info(addr).await?;

    let mut server_info = extract_time_and_queue(info.extended_server_info)
        .ok_or(DayzMonitorError::ExtractServerInfoKeywordsNonExistant)?;
    server_info.players = info.players as u32;
    server_info.max_players = info.max_players as u32;

    Ok(server_info)
}

fn extract_time_and_queue(info: ExtendedServerInfo) -> Option<ServerInfo> {
    let values = info.keywords?;
    let split: Vec<&str> = values.split(",").collect();

    let mut server_info = ServerInfo {
        server_time: None,
        players_in_queue: None,
        players: 0,
        max_players: 0,
    };

    for value in split {
        if value.starts_with("lqs") {
            server_info.players_in_queue = value.replace("lqs", "").parse::<u32>().ok();
        } else if value.contains(":") {
            server_info.server_time = Some(value.to_owned())
        }
    }

    Some(server_info)
}
