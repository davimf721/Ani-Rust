use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://api.consumet.org";

#[derive(Debug, Deserialize)]
pub struct AnimeItem {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<AnimeItem>,
}

#[derive(Debug, Deserialize)]
pub struct EpisodeItem {
    pub id: String,
    pub number: String,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
struct InfoResponse {
    episodes: Vec<EpisodeItem>,
}

#[derive(Debug, Deserialize)]
struct WatchResponse {
    sources: Vec<Source>,
}

#[derive(Debug, Deserialize)]
struct Source {
    pub url: String,
    pub quality: String,
}

/// Busca animes via Consumet Meta API (Gogoanime)
pub async fn search_anime(query: &str) -> Result<Vec<AnimeItem>> {
    let url = format!("{}/anime/gogoanime/{}?page=1", BASE_URL, query);
    let resp = Client::new()
        .get(&url)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("HTTP {}: {}", status, body);
    }

    let body = resp.text().await?;
    let search: SearchResponse = serde_json::from_str(&body)?;
    if search.results.is_empty() {
        anyhow::bail!("Nenhum anime encontrado para: \"{}\"", query);
    }
    Ok(search.results)
}

/// Obtém lista de episódios para um anime específico
 pub async fn get_episodes(anime_id: &str) -> Result<Vec<EpisodeItem>> {
    let url = format!("{}/anime/gogoanime/info/{}", BASE_URL, anime_id);
    let resp = Client::new()
        .get(&url)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("HTTP {}: {}", status, body);
    }

    let body = resp.text().await?;
    let info: InfoResponse = serde_json::from_str(&body)?;
    if info.episodes.is_empty() {
        anyhow::bail!("Nenhum episódio encontrado para anime id: {}", anime_id);
    }
    Ok(info.episodes)
}

/// Obtém o melhor link de streaming para um episódio específico
pub async fn get_stream_url(episode_id: &str) -> Result<String> {
    let url = format!("{}/anime/gogoanime/watch/{}", BASE_URL, episode_id);
    let resp = Client::new()
        .get(&url)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("HTTP {}: {}", status, body);
    }

    let body = resp.text().await?;
    let watch: WatchResponse = serde_json::from_str(&body)?;
    let best = watch.sources
        .into_iter()
        .max_by_key(|s| s.quality.parse::<u32>().unwrap_or(0))
        .ok_or_else(|| anyhow!("No sources available"))?;
    Ok(best.url)
}