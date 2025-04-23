use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// Estrutura para armazenar o histórico de visualização
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WatchHistory {
    pub entries: Vec<HistoryEntry>,
}

// Entrada individual do histórico
#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub anime_id: String,
    pub anime_title: String,
    pub last_episode: String,
    pub total_episodes: Option<i32>,
    pub timestamp: u64,
}

impl WatchHistory {
    // Carrega o histórico do arquivo, ou cria um novo se não existir
    pub fn load() -> Result<Self> {
        let history_path = get_history_path()?;
        
        if !history_path.exists() {
            return Ok(WatchHistory::default());
        }
        
        let mut file = File::open(&history_path)
            .context("Falha ao abrir arquivo de histórico")?;
        
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .context("Falha ao ler arquivo de histórico")?;
        
        if contents.trim().is_empty() {
            return Ok(WatchHistory::default());
        }
        
        serde_json::from_str(&contents)
            .context("Falha ao deserializar histórico")
    }
    
    // Salva o histórico no arquivo
    pub fn save(&self) -> Result<()> {
        let history_path = get_history_path()?;
        
        // Cria o diretório se não existir
        if let Some(parent) = history_path.parent() {
            fs::create_dir_all(parent)
                .context("Falha ao criar diretório para histórico")?;
        }
        
        let json = serde_json::to_string_pretty(self)
            .context("Falha ao serializar histórico")?;
        
        let mut file = File::create(&history_path)
            .context("Falha ao abrir arquivo de histórico para escrita")?;
        
        file.write_all(json.as_bytes())
            .context("Falha ao escrever histórico no arquivo")?;
        
        Ok(())
    }
    
    // Adiciona ou atualiza uma entrada no histórico
    pub fn update_entry(&mut self, entry: HistoryEntry) -> Result<()> {
        // Verifica se já existe uma entrada para este anime
        if let Some(existing) = self.entries.iter_mut().find(|e| e.anime_id == entry.anime_id) {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
        
        // Limita o histórico a 100 entradas, removendo as mais antigas
        if self.entries.len() > 100 {
            self.entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            self.entries.truncate(100);
        }
        
        self.save()
    }
    
    // Obtém a entrada mais recente do histórico
    pub fn get_latest_entry(&self) -> Option<&HistoryEntry> {
        self.entries.iter()
            .max_by_key(|entry| entry.timestamp)
    }
    
    // Limpa o histórico
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        self.save()
    }
}

// Obtém o caminho para o arquivo de histórico
fn get_history_path() -> Result<PathBuf> {
    let mut path = dirs::config_dir()
        .context("Não foi possível determinar o diretório de configuração")?;
    
    path.push("ani-cli-rust");
    path.push("history.json");
    
    Ok(path)
}

// Obtém o timestamp atual
pub fn get_current_timestamp() -> Result<u64> {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH)
        .context("Erro ao obter timestamp")?;
    
    Ok(since_epoch.as_secs())
}
