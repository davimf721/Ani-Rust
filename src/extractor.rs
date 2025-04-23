use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use regex::Regex;

// Estrutura para representar um link de vídeo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoLink {
    pub quality: String,
    pub url: String,
}

// Constantes para a API
const ANIMEFIRE_BASE: &str = "https://animefire.plus";
const ALLANIME_REFR: &str = "https://allanime.day";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";


// Função para obter a URL do episódio
pub async fn get_episode_url(anime_id: &str, episode_number: &str, mode: &str, quality: &str) -> Result<String> {
    // Obter os links de embed do episódio
    let embed_links = get_episode_embed_links(anime_id, episode_number, mode).await?;
    
    if embed_links.is_empty() {
        return Err(anyhow!("No embed links found for episode {}", episode_number));
    }
    
    // Extrair links de vídeo de cada provedor
    let mut all_links = Vec::new();
    
    for (provider, link) in embed_links {
        match provider.as_str() {
            "Default" => {
                // Provedor wixmp (default)
                let links = extract_wixmp_links(&link).await?;
                all_links.extend(links);
            },
            "Sak" => {
                // Provedor dropbox
                let links = extract_dropbox_links(&link).await?;
                all_links.extend(links);
            },
            "Kir" => {
                // Provedor wetransfer
                let links = extract_wetransfer_links(&link).await?;
                all_links.extend(links);
            },
            "S-mp4" => {
                // Provedor sharepoint
                let links = extract_sharepoint_links(&link).await?;
                all_links.extend(links);
            },
            "Luf-mp4" => {
                // Provedor gogoanime
                let links = extract_gogoanime_links(&link).await?;
                all_links.extend(links);
            },
            _ => {
                // Provedor desconhecido, tentar extração genérica
                if let Ok(links) = extract_generic_links(&link).await {
                    all_links.extend(links);
                }
            }
        }
    }
    
    // Ordenar links por qualidade (maior para menor)
    all_links.sort_by(|a, b| {
        let a_quality = a.quality.parse::<i32>().unwrap_or(0);
        let b_quality = b.quality.parse::<i32>().unwrap_or(0);
        b_quality.cmp(&a_quality)
    });
    
    // Selecionar link com a qualidade desejada
    let selected_link = select_quality(&all_links, quality)?;
    
    Ok(selected_link)
}

// Função para obter os links de embed do episódio
async fn get_episode_embed_links(anime_id: &str, episode_number: &str, mode: &str) -> Result<Vec<(String, String)>> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    // Construir a query GraphQL
    let episode_embed_gql = "query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) {
        episode(
            showId: $showId
            translationType: $translationType
            episodeString: $episodeString
        ) {
            episodeString
            sourceUrls
        }
    }";
    
    // Construir as variáveis para a query
    let variables = serde_json::json!({
        "showId": anime_id,
        "translationType": mode,
        "episodeString": episode_number
    });
    
    // Fazer a requisição para a API
    let body = serde_json::json!({
        "query": episode_embed_gql,
        "variables": variables
    });
    
    let response_text = client.post(format!("{}/api", ANIMEFIRE_BASE))
        .header("Referer", ALLANIME_REFR)
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&body)?)
        .send()
        .await?
        .text()
        .await?;
    
    // Extrair os links de embed usando regex (similar ao script original)
    let re = Regex::new(r#"sourceUrl":"--([^"]*)".*?sourceName":"([^"]*)""#)?;
    
    let mut embed_links = Vec::new();
    for cap in re.captures_iter(&response_text) {
        let url = cap[1].replace("\\u002F", "/");
        let provider = cap[2].to_string();
        embed_links.push((provider, url));
    }
    
    Ok(embed_links)
}

// Função para extrair links do provedor wixmp
async fn extract_wixmp_links(embed_url: &str) -> Result<Vec<VideoLink>> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    let response_text = client.get(embed_url)
        .header("Referer", ALLANIME_REFR)
        .send()
        .await?
        .text()
        .await?;
    
    let mut links = Vec::new();
    
    // Extrair links de vídeo usando regex
    let re = Regex::new(r#"link":"([^"]*)".*?resolutionStr":"([^"]*)""#)?;
    for cap in re.captures_iter(&response_text) {
        let url = cap[1].replace("\\u002F", "/");
        let quality = cap[2].to_string();
        
        links.push(VideoLink {
            quality,
            url,
        });
    }
    
    // Extrair links HLS (m3u8)
    let re_hls = Regex::new(r#"hls","url":"([^"]*)".*?"hardsub_lang":"en-US""#)?;
    if let Some(cap) = re_hls.captures(&response_text) {
        let m3u8_url = cap[1].replace("\\u002F", "/");
        
        // Processar m3u8 para extrair diferentes qualidades
        let m3u8_links = process_m3u8(&m3u8_url).await?;
        links.extend(m3u8_links);
    }
    
    Ok(links)
}

// Função para processar m3u8 e extrair links de diferentes qualidades
async fn process_m3u8(m3u8_url: &str) -> Result<Vec<VideoLink>> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    let response_text = client.get(m3u8_url)
        .header("Referer", ALLANIME_REFR)
        .send()
        .await?
        .text()
        .await?;
    
    let mut links = Vec::new();
    let base_url = m3u8_url.rsplitn(2, '/').nth(1).unwrap_or("");
    
    // Extrair links de diferentes qualidades
    let re = Regex::new(r"#EXT-X-STREAM-INF:.*?RESOLUTION=\d+x(\d+).*?\n(.*?)$")?;
    for cap in re.captures_iter(&response_text) {
        let quality = cap[1].to_string();
        let path = cap[2].trim();
        
        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            format!("{}/{}", base_url, path)
        };
        
        links.push(VideoLink {
            quality,
            url,
        });
    }
    
    Ok(links)
}

// Funções para extrair links de outros provedores
async fn extract_dropbox_links(embed_url: &str) -> Result<Vec<VideoLink>> {
    // Implementação simplificada para dropbox
    Ok(vec![VideoLink {
        quality: "720".to_string(),
        url: embed_url.to_string(),
    }])
}

async fn extract_wetransfer_links(embed_url: &str) -> Result<Vec<VideoLink>> {
    // Implementação simplificada para wetransfer
    Ok(vec![VideoLink {
        quality: "720".to_string(),
        url: embed_url.to_string(),
    }])
}

async fn extract_sharepoint_links(embed_url: &str) -> Result<Vec<VideoLink>> {
    // Implementação simplificada para sharepoint
    Ok(vec![VideoLink {
        quality: "720".to_string(),
        url: embed_url.to_string(),
    }])
}

async fn extract_gogoanime_links(embed_url: &str) -> Result<Vec<VideoLink>> {
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    let response_text = client.get(embed_url)
        .header("Referer", ALLANIME_REFR)
        .send()
        .await?
        .text()
        .await?;
    
    let mut links = Vec::new();
    
    // Extrair links de vídeo usando regex
    let re = Regex::new(r#"source src="([^"]+)""#)?;
    if let Some(cap) = re.captures(&response_text) {
        let url = cap[1].to_string();
        
        // Se for m3u8, processar para extrair diferentes qualidades
        if url.ends_with(".m3u8") {
            let m3u8_links = process_m3u8(&url).await?;
            links.extend(m3u8_links);
        } else {
            links.push(VideoLink {
                quality: "720".to_string(),
                url,
            });
        }
    }
    
    Ok(links)
}

async fn extract_generic_links(embed_url: &str) -> Result<Vec<VideoLink>> {
    // Tentativa genérica de extrair links de vídeo
    let client = Client::builder()
        .user_agent(USER_AGENT)
        .build()?;
    
    let response_text = client.get(embed_url)
        .header("Referer", ALLANIME_REFR)
        .send()
        .await?
        .text()
        .await?;
    
    let mut links = Vec::new();
    
    // Tentar extrair links de vídeo usando diferentes padrões
    let patterns = [
        r#"source src="([^"]+)""#,
        r#"file: *"([^"]+)""#,
        r#"src: *"([^"]+)""#,
        r#"url: *"([^"]+)""#,
    ];
    
    for pattern in patterns {
        let re = Regex::new(pattern)?;
        if let Some(cap) = re.captures(&response_text) {
            let url = cap[1].to_string();
            
            // Se for m3u8, processar para extrair diferentes qualidades
            if url.ends_with(".m3u8") {
                if let Ok(m3u8_links) = process_m3u8(&url).await {
                    links.extend(m3u8_links);
                }
            } else {
                links.push(VideoLink {
                    quality: "720".to_string(),
                    url,
                });
            }
        }
    }
    
    Ok(links)
}

// Função para selecionar a qualidade desejada
fn select_quality(links: &[VideoLink], quality: &str) -> Result<String> {
    if links.is_empty() {
        return Err(anyhow!("No video links found"));
    }
    
    match quality {
        "best" => {
            // Retornar o link de maior qualidade
            Ok(links[0].url.clone())
        },
        "worst" => {
            // Retornar o link de menor qualidade
            Ok(links.last().unwrap().url.clone())
        },
        _ => {
            // Tentar encontrar a qualidade especificada
            for link in links {
                if link.quality == quality {
                    return Ok(link.url.clone());
                }
            }
            
            // Se não encontrar, retornar o link de maior qualidade
            println!("Specified quality not found, defaulting to best");
            Ok(links[0].url.clone())
        }
    }
}
