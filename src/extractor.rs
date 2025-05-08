// src/extractor.rs

use anyhow::{Result, anyhow};
use gogoanime_scraper::utils::get_html;
use gogoanime_scraper::parser::get_media_url;

/// Retorna a URL de streaming do episódio desejado.
pub async fn extract_video_url(anime_url: &str, episode: u32) -> Result<String> {
    // Remove o sufixo ".html" (caso exista) e monta a URL de episódio
    let base = anime_url.trim_end_matches(".html");
    let ep_page = format!("{}/episode-{}", base, episode);

    // Baixa o HTML da página de episódio (String)
    let html: String = get_html(ep_page.clone()).await?;

    // Chama get_media_url sem o `?`, já que ele retorna Vec<String>
    let media_urls: Vec<String> = get_media_url(html);

    // Escolhe a primeira URL disponível ou retorna erro
    let first_url = media_urls
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("Nenhuma URL de mídia encontrada"))?;

    Ok(first_url)
}

