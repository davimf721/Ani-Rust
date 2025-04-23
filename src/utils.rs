use anyhow::{Result, Context};
use regex::Regex;
use std::process::Command;

// Função para normalizar um título de anime para busca
// 
// Esta função aplica várias transformações para aumentar as chances de correspondência:
// - Converte para minúsculas
// - Remove prefixos comuns (o, a, the, etc.)
// - Remove caracteres especiais
// - Remove palavras comuns que podem variar entre sites
// - Substitui espaços por hífens
pub fn normalize_title(title: &str) -> String {
    let mut normalized = title.to_lowercase();
    
    // Remove prefixos comuns
    let prefixes = ["o ", "a ", "as ", "os ", "the ", "um ", "uma "];
    for prefix in prefixes.iter() {
        if normalized.starts_with(prefix) {
            normalized = normalized[prefix.len()..].to_string();
        }
    }
    
    // Substitui caracteres especiais e espaços
    let re = Regex::new(r"[:\!\?,\.'\").unwrap();
    normalized = re.replace_all(&normalized, "").to_string();
    
    // Remove artigos e palavras comuns que podem variar entre sites
    let skip_words = ["o", "a", "os", "as", "um", "uma", "the", "season", "temporada", "ova", "movie", "filme"];
    normalized = normalized.split_whitespace()
        .filter(|word| !skip_words.contains(word))
        .collect::<Vec<_>>()
        .join(" ");
    
    // Remove espaços e traços em excesso
    normalized = normalized.replace(' ', "-");
    while normalized.contains("--") {
        normalized = normalized.replace("--", "-");
    }
    
    normalized.trim_matches('-').to_string()
}

// Função para verificar se um programa está instalado
pub fn is_program_installed(program: &str) -> bool {
    Command::new("which")
        .arg(program)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// Função para formatar tempo em segundos para formato legível
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

// Função para extrair números de uma string
pub fn extract_number(s: &str) -> Option<f32> {
    let re = Regex::new(r"(\d+(\.\d+)?)").unwrap();
    re.captures(s)
        .and_then(|cap| cap[1].parse::<f32>().ok())
}

// Função para criar um diretório se não existir
pub fn ensure_directory(path: &str) -> Result<()> {
    let path = std::path::Path::new(path);
    if !path.exists() {
        std::fs::create_dir_all(path)
            .context(format!("Falha ao criar diretório: {}", path.display()))?;
    }
    Ok(())
}

// Função para obter o nome do arquivo a partir de uma URL
pub fn get_filename_from_url(url: &str) -> String {
    url.split('/')
        .last()
        .unwrap_or("video.mp4")
        .split('?')
        .next()
        .unwrap_or("video.mp4")
        .to_string()
}
