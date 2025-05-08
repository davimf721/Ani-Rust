use anyhow::{Result, anyhow};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Serialize, Deserialize};
use url::Url;

const GOGOANIME_URL: &str = "https://gogoanime3.cc";
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeItem {
    pub id: String,       // URL slug para o anime
    pub title: String,    // Título visível do anime
    pub image: String,    // URL da imagem de capa (opcional)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeItem {
    pub id: String,       // ID do episódio para streaming
    pub number: String,   // Número do episódio
    pub title: Option<String>, // Título do episódio (se disponível)
}

/// Função para buscar animes pelo nome
pub async fn search_anime(query: &str) -> Result<Vec<AnimeItem>> {
    println!("Buscando anime no GoGoAnime: {}", query);
    
    // Constrói a URL de busca
    let search_url = format!("{}/search.html?keyword={}", GOGOANIME_URL, query);
    
    // Faz a requisição HTTP
    let client = Client::new();
    let resp = client
        .get(&search_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;
    
    if !resp.status().is_success() {
        return Err(anyhow!("Falha na busca: HTTP {}", resp.status()));
    }
    
    let html = resp.text().await?;
    let document = Html::parse_document(&html);
    
    // Seletor para os itens de resultado
    let item_selector = Selector::parse("div.last_episodes ul.items li").unwrap();
    let name_selector = Selector::parse("p.name a").unwrap();
    let img_selector = Selector::parse("div.img img").unwrap();
    
    let mut results = Vec::new();
    
    for item in document.select(&item_selector) {
        if let Some(name_el) = item.select(&name_selector).next() {
            let title = name_el.text().collect::<String>().trim().to_string();
            
            // Extrai o ID/slug do atributo href
            if let Some(href) = name_el.value().attr("href") {
                // Remove a leading "/" se existir
                let id = href.trim_start_matches('/').to_string();
                
                // Extrai a URL da imagem se disponível
                let image = item
                    .select(&img_selector)
                    .next()
                    .and_then(|img| img.value().attr("src"))
                    .unwrap_or("")
                    .to_string();
                
                results.push(AnimeItem {
                    id,
                    title,
                    image,
                });
            }
        }
    }
    
    if results.is_empty() {
        return Err(anyhow!("Nenhum anime encontrado para: \"{}\"", query));
    }
    
    println!("Encontrados {} resultados", results.len());
    Ok(results)
}

/// Função para obter a lista de episódios
pub async fn get_episodes(anime_id: &str) -> Result<Vec<EpisodeItem>> {
    println!("Buscando episódios para: {}", anime_id);
    
    // Constrói a URL do anime
    let anime_url = format!("{}/{}", GOGOANIME_URL, anime_id);
    
    // Faz a requisição HTTP
    let client = Client::new();
    let resp = client
        .get(&anime_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;
    
    if !resp.status().is_success() {
        return Err(anyhow!("Falha ao carregar página do anime: HTTP {}", resp.status()));
    }
    
    let html = resp.text().await?;
    let document = Html::parse_document(&html);
    
    // Precisamos pegar o ID do anime no GoGoAnime para acessar a lista de episódios
    let id_selector = Selector::parse("#movie_id").unwrap();
    let anime_id_value = document
        .select(&id_selector)
        .next()
        .and_then(|el| el.value().attr("value"))
        .ok_or_else(|| anyhow!("ID do anime não encontrado na página"))?;

    let ep_start_selector = Selector::parse("#episode_page a.active").unwrap();
    let total_eps = document
        .select(&ep_start_selector)
        .next()
        .and_then(|el| el.value().attr("ep_end"))
        .and_then(|ep| ep.parse::<i32>().ok())
        .unwrap_or(1);
        
    println!("Total de episódios encontrados: {}", total_eps);
    
    // Agora podemos montar a lista de episódios manualmente
    let mut episodes = Vec::new();
    for ep_num in 1..=total_eps {
        // Corrigindo o formato do ID do episódio para corresponder ao padrão do site
        let episode_id = if anime_id.starts_with("category/") {
            format!("{}-episode-{}", anime_id.trim_start_matches("category/"), ep_num)
        } else {
            format!("{}-episode-{}", anime_id, ep_num)
        };
        
        episodes.push(EpisodeItem {
            id: episode_id.clone(),
            number: ep_num.to_string(),
            title: None, // GoGoAnime normalmente não tem títulos dos episódios
        });
    }
    
    // Ordenar episódios em ordem decrescente (mais recentes primeiro)
    episodes.reverse();
    
    if episodes.is_empty() {
        return Err(anyhow!("Nenhum episódio encontrado para este anime"));
    }
    
    Ok(episodes)
}

/// Função para extrair a URL de streaming
pub async fn get_stream_url(episode_id: &str) -> Result<String> {
    println!("Extraindo URL de streaming para: {}", episode_id);
    
    // Ajustando o formato da URL do episódio para corresponder ao padrão do site
    let episode_url = if episode_id.contains("/") {
        format!("{}/{}", GOGOANIME_URL, episode_id)
    } else {
        format!("{}/watch/{}", GOGOANIME_URL, episode_id)
    };
    
    println!("URL do episódio: {}", episode_url);
    
    // Faz a requisição HTTP
    let client = Client::new();
    let resp = client
        .get(&episode_url)
        .header("User-Agent", USER_AGENT)
        .send()
        .await?;
    
    if !resp.status().is_success() {
        return Err(anyhow!("Falha ao carregar página do episódio: HTTP {}", resp.status()));
    }
    
    let html = resp.text().await?;
    let document = Html::parse_document(&html);
    
    // Encontra o iframe do player (tentando diferentes seletores)
    let iframe_selectors = [
        Selector::parse("div.play-video iframe").unwrap(),
        Selector::parse("iframe#player").unwrap(),
        Selector::parse("iframe[src*='streaming']").unwrap(),
        Selector::parse("iframe").unwrap(),
    ];
    
    let mut iframe_src = None;
    for selector in &iframe_selectors {
        if let Some(iframe) = document.select(selector).next() {
            if let Some(src) = iframe.value().attr("src") {
                iframe_src = Some(src.to_string());
                println!("Iframe encontrado com seletor: {:?}", selector);
                break;
            }
        }
    }
    
    let iframe_src = iframe_src.ok_or_else(|| {
        // Se não encontrarmos o iframe, vamos mostrar a estrutura do HTML para debug
        println!("Estrutura HTML da página:");
        document.select(&Selector::parse("body").unwrap()).for_each(|el| {
            println!("{:?}", el.html());
        });
        anyhow!("Player iframe não encontrado")
    })?;
    
    println!("URL do iframe: {}", iframe_src);
    
    // Se o iframe_src não tiver schema (começando com //), adiciona https:
    let iframe_url = if iframe_src.starts_with("//") {
        format!("https:{}", iframe_src)
    } else {
        iframe_src.to_string()
    };
    
    // Agora precisamos fazer uma segunda solicitação para a página do iframe
    let resp = client
        .get(&iframe_url)
        .header("User-Agent", USER_AGENT)
        .header("Referer", episode_url)
        .send()
        .await?;
    
    if !resp.status().is_success() {
        return Err(anyhow!("Falha ao carregar player: HTTP {}", resp.status()));
    }
    
    let player_html = resp.text().await?;
    
    // Buscamos pelo link direto do vídeo no HTML ou JSON do player
    // Note: Essa parte é instável e pode precisar de atualizações frequentes
    // conforme o site muda sua estrutura
    
    // Vamos tentar diferentes padrões para encontrar a URL do vídeo
    // 1. Procura pelo padrão "file":"URL_DO_VIDEO"
    if let Some(pos) = player_html.find("\"file\":\"") {
        let start = pos + 8; // 8 é o comprimento de "\"file\":\""
        if let Some(end) = player_html[start..].find("\"") {
            let video_url = &player_html[start..start + end];
            // Decodifica sequências de escape JSON (\/)
            let video_url = video_url.replace("\\/", "/");
            println!("URL de streaming encontrada (padrão 1): {}", video_url);
            return Ok(video_url);
        }
    }
    
    // 2. Procura pelo padrão "src":"URL_DO_VIDEO"
    if let Some(pos) = player_html.find("\"src\":\"") {
        let start = pos + 7; // 7 é o comprimento de "\"src\":\""
        if let Some(end) = player_html[start..].find("\"") {
            let video_url = &player_html[start..start + end];
            let video_url = video_url.replace("\\/", "/");
            println!("URL de streaming encontrada (padrão 2): {}", video_url);
            return Ok(video_url);
        }
    }
    
    // 3. Procura por URLs .mp4 ou .m3u8
    let html_lowercase = player_html.to_lowercase();
    for pattern in &[".mp4", ".m3u8"] {
        if let Some(pos) = html_lowercase.find(pattern) {
            // Retrocedendo até encontrar http ou https
            let mut start = pos;
            while start > 0 && !html_lowercase[start-7..start].contains("http") {
                start -= 1;
                if start < 7 { break; }
            }
            
            if start >= 7 {
                let proto_start = html_lowercase[start-7..start].find("http").unwrap() + (start-7);
                let mut end = pos + pattern.len();
                
                // Avança até encontrar aspas, espaço ou >
                while end < html_lowercase.len() && !"\"> ".contains(&html_lowercase[end..=end]) {
                    end += 1;
                }
                
                let video_url = &player_html[proto_start..end];
                println!("URL de streaming encontrada (padrão 3): {}", video_url);
                return Ok(video_url.to_string());
            }
        }
    }
    
    // 4. Se não encontrarmos nada, vamos procurar qualquer URL de outro player
    let document = Html::parse_document(&player_html);
    let link_selector = Selector::parse("a[href*='streaming'], a[href*='watch'], iframe[src*='embed']").unwrap();
    
    if let Some(link) = document.select(&link_selector).next() {
        if let Some(href) = link.value().attr("href").or_else(|| link.value().attr("src")) {
            println!("Link alternativo encontrado: {}", href);
            
            // Se for URL relativa, adiciona o domínio base
            let video_url = if href.starts_with("http") {
                href.to_string()
            } else if href.starts_with("//") {
                format!("https:{}", href)
            } else {
                // Obtém domínio base da iframe_url
                let base_url = Url::parse(&iframe_url)?;
                let domain = format!("{}://{}", base_url.scheme(), base_url.host_str().unwrap_or(""));
                format!("{}{}", domain, href)
            };
            
            println!("Redirecionando para: {}", video_url);
            return Ok(video_url);
        }
    }
    
    // Se chegamos aqui, não conseguimos encontrar a URL do vídeo
    println!("ERRO: Não foi possível encontrar a URL do vídeo no HTML:");
    println!("Primeiros 200 caracteres: {}", &player_html[..200.min(player_html.len())]);
    println!("Últimos 200 caracteres: {}", &player_html[player_html.len() - 200.min(player_html.len())..]);
    
    Err(anyhow!("Não foi possível extrair a URL do vídeo"))
}