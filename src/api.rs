use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use reqwest::{header, Client};

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
#[derive(Clone)]
struct ApiConfig {
    client: reqwest::Client,
    headers: header::HeaderMap,
}

impl ApiConfig {
    fn new() -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        const BASE_URL: &str = "https://allanime.day";
        headers.insert("Referer", header::HeaderValue::from_static(BASE_URL));
        headers.insert("Origin", header::HeaderValue::from_static(BASE_URL));
        headers.insert("User-Agent", header::HeaderValue::from_static(USER_AGENT));
        
        Ok(Self {
            client: reqwest::Client::builder()
                .default_headers(headers.clone())
                .build()?,
            headers,
        })
    }
}

// Função para buscar animes
pub async fn search_anime(query: &str) -> Result<Vec<Anime>> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    // Construir a query GraphQL similar ao ani-cli original
    const SEARCH_QUERY: &str = r#"
query(
    $search: SearchInput
    $limit: Int
    $page: Int
    $translationType: VaildTranslationTypeEnumType
) {
    shows(
        search: $search
        limit: $limit
        page: $page
        translationType: $translationType
    ) {
        edges {
            _id
            name
            thumbnail
            availableEpisodes
        }
    }
}"#;
    
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
        "query": SEARCH_QUERY,
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
    
    // Verificar se há erros na resposta
    if response["errors"].is_array() {
        let error_msg = response["errors"][0]["message"].as_str()
            .unwrap_or("Unknown API error");
        return Err(anyhow!("API Error: {}", error_msg));
    }
    
    // Verificar se a estrutura da resposta é válida
    if !response["data"].is_object() {
        return Err(anyhow!("Invalid API response: 'data' field missing or not an object"));
    }
    
    if !response["data"]["shows"].is_object() {
        return Err(anyhow!("Invalid API response: 'data.shows' field missing or not an object"));
    }
    
    // Extrair os resultados da resposta com tratamento de erro melhorado
    let edges = match response["data"]["shows"]["edges"].as_array() {
        Some(arr) => arr,
        None => {
            // Tentar uma estrutura alternativa que pode estar sendo usada pela API
            if let Some(arr) = response["data"]["shows"]["items"].as_array() {
                arr
            } else {
                return Err(anyhow!("Invalid API response format: 'edges' or 'items' array not found"));
            }
        }
    };
    
    // Converter para o formato Anime
    let mut results = Vec::new();
    for edge in edges {
        let id = edge["_id"].as_str().unwrap_or("").to_string();
        if id.is_empty() {
            continue; // Pular entradas sem ID
        }
        
        let title = edge["name"].as_str().unwrap_or("").to_string();
        let image = edge["thumbnail"].as_str().map(|s| s.to_string());
        
        // Tratamento mais robusto para total_episodes
        let total_episodes = if edge["availableEpisodes"].is_object() {
            edge["availableEpisodes"]["sub"].as_i64().map(|e| e as i32)
        } else if edge["availableEpisodes"].is_i64() {
            Some(edge["availableEpisodes"].as_i64().unwrap() as i32)
        } else {
            None
        };
        
        results.push(Anime {
            id,
            title,
            image,
            total_episodes,
        });
    }
    
    if results.is_empty() {
        return Err(anyhow!("No anime found for query: {}", query));
    }
    
    Ok(results)
}

// Função para obter a lista de episódios de um anime
pub async fn get_anime_episodes(anime_id: &str, mode: &str) -> Result<Vec<Episode>> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    // Construir a query GraphQL similar ao ani-cli original
    const EPISODES_QUERY: &str = r#"
query($showId: String!) {
    show(_id: $showId) {
        _id
        availableEpisodes
        episodesList {
            total
            episodes {
                episodeId
                episodeNumber
                episodeString
                title
                notes
            }
        }
    }
}"#;
    
    let variables = serde_json::json!({
        "showId": anime_id
    });
    
    let body = serde_json::json!({
        "query": EPISODES_QUERY,
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

    let response: serde_json::Value = serde_json::from_str(&response_text)?;

    let episodes = match response["data"]["show"]["episodes"].as_array() {
        Some(arr) => arr,
        None => return Err(anyhow!("No episodes found in API response")),
    };

    
    // Verificar se há erros na resposta
    if response["errors"].is_array() {
        let error_msg = response["errors"][0]["message"].as_str()
            .unwrap_or("Unknown API error");
        return Err(anyhow!("API Error: {}", error_msg));
    }
    
    // Verificar se a estrutura da resposta é válida
    if !response["data"].is_object() {
        return Err(anyhow!("Invalid API response: 'data' field missing or not an object"));
    }
    
    if !response["data"]["show"].is_object() {
        return Err(anyhow!("Invalid API response: 'data.show' field missing or not an object"));
    }
    
    // Extrair os episódios da resposta com tratamento de erro melhorado
    let episodes = match response["data"]["show"]["episodes"].as_array() {
        Some(arr) => arr,
        None => {
            // Tentar uma estrutura alternativa que pode estar sendo usada pela API
            if let Some(arr) = response["data"]["show"]["episodesList"].as_array() {
                arr
            } else {
                return Err(anyhow!("Invalid API response format: 'episodes' or 'episodesList' array not found"));
            }
        }
    };
    
    // Determinar o número total de episódios disponíveis para o modo selecionado
    let available_episodes = if response["data"]["show"]["availableEpisodes"].is_object() {
        match mode {
            "sub" => response["data"]["show"]["availableEpisodes"]["sub"].as_i64(),
            "dub" => response["data"]["show"]["availableEpisodes"]["dub"].as_i64(),
            _ => None,
        }.unwrap_or(0) as usize
    } else if response["data"]["show"]["availableEpisodes"].is_i64() {
        response["data"]["show"]["availableEpisodes"].as_i64().unwrap_or(0) as usize
    } else {
        episodes.len() // Fallback para o número de episódios na lista
    };
    
    // Converter para o formato Episode
    let mut results = Vec::new();
    for (i, episode) in episodes.iter().enumerate() {
        // Verificar se o episódio está disponível no modo selecionado
        let is_available = if episode[mode].is_boolean() {
            episode[mode].as_bool().unwrap_or(false)
        } else {
            true // Se não houver campo específico, assumir disponível
        };
        
        // Só adicionar episódios disponíveis e dentro do limite
        if is_available && i < available_episodes {
            let number = episode["number"].as_str().unwrap_or("").to_string();
            if number.is_empty() {
                continue; // Pular episódios sem número
            }
            
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
    
    if results.is_empty() {
        return Err(anyhow!("No episodes found for anime ID: {}", anime_id));
    }
    
    Ok(results)
}
