use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use reqwest::Client;

// Estrutura para representar um anime nos resultados de busca
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anime {
    pub id: String,
    pub title: String,
    pub image: Option<String>,
    pub total_episodes: Option<i32>,
}

// Estrutura para representar um episódio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub number: String,
    pub title: Option<String>,
    pub id: String,
}

// Constantes para a API
const ALLANIME_API: &str = "https://api.allanime.day";
const ALLANIME_REFR: &str = "https://allanime.day";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

// Função para buscar animes
pub async fn search_anime(query: &str) -> Result<Vec<Anime>> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    // Construir a query GraphQL similar ao ani-cli original
    let search_gql = "query($search: SearchInput, $limit: Int, $page: Int, $translationType: VaildTranslationTypeEnumType) {
        shows(search: $search, limit: $limit, page: $page, translationType: $translationType) {
            edges {
                _id
                name
                availableEpisodes
                thumbnail
            }
        }
    }";
    
    // Construir as variáveis para a query
    let variables = serde_json::json!({
        "search": {
            "allowAdult": true,
            "allowUnknown": true,
            "query": query
        },
        "limit": 20,
        "page": 1,
        "translationType": "sub"
    });
    
    // Fazer a requisição para a API
    let body = serde_json::json!({
        "query": search_gql,
        "variables": variables
    });
    
    let response_text = client.post(format!("{}/api", ALLANIME_API))
        .header("Referer", ALLANIME_REFR)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body)?)
        .send()
        .await?
        .text()
        .await?;
    
    // Parsear a resposta JSON
    let response: serde_json::Value = serde_json::from_str(&response_text)?;
    
    // Extrair os resultados da resposta
    let edges = response["data"]["shows"]["edges"].as_array()
        .ok_or_else(|| anyhow!("Invalid API response format"))?;
    
    // Converter para o formato Anime
    let mut results = Vec::new();
    for edge in edges {
        let id = edge["_id"].as_str().unwrap_or("").to_string();
        let title = edge["name"].as_str().unwrap_or("").to_string();
        let image = edge["thumbnail"].as_str().map(|s| s.to_string());
        let total_episodes = edge["availableEpisodes"]["sub"].as_i64().map(|e| e as i32);
        
        results.push(Anime {
            id,
            title,
            image,
            total_episodes,
        });
    }
    
    Ok(results)
}

// Função para obter a lista de episódios de um anime
pub async fn get_anime_episodes(anime_id: &str, mode: &str) -> Result<Vec<Episode>> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    // Construir a query GraphQL similar ao ani-cli original
    let episodes_gql = "query($showId: String!) {
        show(
            _id: $showId
        ) {
            _id
            availableEpisodes
            episodes {
                sub
                dub
                raw
                number
                title
            }
        }
    }";
    
    // Construir as variáveis para a query
    let variables = serde_json::json!({
        "showId": anime_id
    });
    
    // Fazer a requisição para a API
    let body = serde_json::json!({
        "query": episodes_gql,
        "variables": variables
    });
    
    let response_text = client.post(format!("{}/api", ALLANIME_API))
        .header("Referer", ALLANIME_REFR)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body)?)
        .send()
        .await?
        .text()
        .await?;
    
    // Parsear a resposta JSON
    let response: serde_json::Value = serde_json::from_str(&response_text)?;
    
    // Extrair os episódios da resposta
    let episodes = response["data"]["show"]["episodes"].as_array()
        .ok_or_else(|| anyhow!("Invalid API response format"))?;
    
    // Determinar o número total de episódios disponíveis para o modo selecionado
    let available_episodes = match mode {
        "sub" => response["data"]["show"]["availableEpisodes"]["sub"].as_i64(),
        "dub" => response["data"]["show"]["availableEpisodes"]["dub"].as_i64(),
        _ => None,
    }.unwrap_or(0) as usize;
    
    // Converter para o formato Episode
    let mut results = Vec::new();
    for (i, episode) in episodes.iter().enumerate() {
        // Verificar se o episódio está disponível no modo selecionado
        let is_available = match mode {
            "sub" => episode["sub"].as_bool().unwrap_or(false),
            "dub" => episode["dub"].as_bool().unwrap_or(false),
            _ => false,
        };
        
        // Só adicionar episódios disponíveis e dentro do limite
        if is_available && i < available_episodes {
            let number = episode["number"].as_str().unwrap_or("").to_string();
            let title = episode["title"].as_str().map(|s| s.to_string());
            
            results.push(Episode {
                number: number.clone(),
                title,
                id: format!("{}-{}", anime_id, number),
            });
        }
    }
    
    // Ordenar episódios por número
    results.sort_by(|a, b| {
        let a_num = a.number.parse::<f32>().unwrap_or(0.0);
        let b_num = b.number.parse::<f32>().unwrap_or(0.0);
        a_num.partial_cmp(&b_num).unwrap()
    });
    
    Ok(results)
}
