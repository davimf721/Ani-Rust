use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use reqwest::{header, Client};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anime {
    pub mal_id: u32,
    pub image: Option<String>,
    pub title: String,
    pub episodes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub mal_id: u32,
    pub title: Option<String>,
    pub url: String,
}

const JIKAN_API: &str = "https://api.jikan.moe/v4";
const USER_AGENT: &str = "MyAniRustClient/1.0 (https://github.com/seu-usuario)";

// Configuração da API Jikan
struct JikanConfig {
    client: Client,
}

impl JikanConfig {
    fn new() -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert("User-Agent", header::HeaderValue::from_static(USER_AGENT));
        
        Ok(Self {
            client: Client::builder()
                .default_headers(headers)
                .build()?,
        })
    }
}

// Função para buscar animes
pub async fn search_anime(query: &str) -> Result<Vec<Anime>> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    let response = client
        .get(format!("{}/anime", JIKAN_API))
        .query(&[("q", query)])
        .send()
        .await?
        .json::<JikanResponse<Vec<JikanAnime>>>()
        .await?;

    let results: Vec<Anime> = response.data
        .into_iter()
        .map(|anime| Anime {
            mal_id: anime.mal_id,
            title: anime.title,
            image: anime.images.jpg.image_url,
            episodes: anime.episodes,
        })
        .collect();

    if results.is_empty() {
        return Err(anyhow!("No anime found for query: {}", query));
    }
    
    Ok(results)
}

// Função para obter episódios
pub async fn get_anime_episodes(anime_id: u32) -> Result<Vec<Episode>> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;

    let response = client
        .get(format!("{}/anime/{}/episodes", JIKAN_API, anime_id))
        .send()
        .await?
        .json::<JikanResponse<Vec<JikanEpisode>>>() // Desserialização direta
        .await?;

    let episodes: Vec<Episode> = response.data
        .into_iter()
        .map(|ep| Episode {
            url: ep.url,
            title: ep.title,
            mal_id: ep.mal_id,
        })
        .collect();

    Ok(episodes)
}

// Estruturas para desserialização da API Jikan
#[derive(Debug, Deserialize)]
struct JikanResponse<T> {
    data: T,
    pagination: JikanPagination,
}
#[derive(Debug, Deserialize)]
struct JikanPagination {
    last_visible_page: u32,
    has_next_page: bool,
}

#[derive(Debug, Deserialize)]
struct JikanAnime {
    mal_id: u32,
    images: JikanImages,
    title: String,
    episodes: Option<u32>,
    
}

#[derive(Debug, Deserialize)]
struct JikanImages {
    jpg: JikanImage,
}

#[derive(Debug, Deserialize)]
struct JikanImage {
    image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JikanEpisode {
    mal_id: u32,
    title: Option<String>,
    url: String,  
}