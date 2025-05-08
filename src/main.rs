use anyhow::Result;
use clap::Parser;

mod gogoanime;
mod player;
mod ui;

use gogoanime::{search_anime, get_episodes, get_stream_url, AnimeItem, EpisodeItem};
use ui::{prompt_input, select_from_list};
use player::play_with_mpv;

#[derive(Parser, Debug)]
#[command(name = "AniRust", about = "Assista animes via CLI!")]
struct Args {
    /// Termo de busca (nome do anime)
    #[arg(short, long)]
    query: Option<String>,

    /// Número do episódio (opcional)
    #[arg(short, long)]
    episode: Option<usize>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    println!("AniRust - Assistente de Anime");
    let query = args.query.unwrap_or_else(|| prompt_input("Digite o nome do anime:").unwrap());

    println!("Buscando animes...");
    let animes: Vec<AnimeItem> = match search_anime(&query).await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Erro ao buscar animes: {}", e);
            return Ok(());
        }
    };

    let titles: Vec<String> = animes.iter().map(|a| a.title.clone()).collect();
    let idx = select_from_list(&titles, "Selecione um anime:")?;
    let selected = &animes[idx];
    println!("Você escolheu: {}", selected.title);

    println!("Buscando episódios...");
    let episodes: Vec<EpisodeItem> = match get_episodes(&selected.id).await {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Erro ao obter episódios: {}", e);
            return Ok(());
        }
    };

    let eps_labels: Vec<String> = episodes
        .iter()
        .map(|e| {
            let ep_num = e.number.clone();
            match &e.title {
                Some(title) => format!("Ep. {} - {}", ep_num, title),
                _none => format!("Episódio {}", ep_num),
            }
        })
        .collect();

    let ep_idx = args.episode
        .and_then(|num| episodes.iter().position(|e| e.number.parse::<usize>().unwrap_or(0) == num))
        .unwrap_or_else(|| select_from_list(&eps_labels, "Selecione um episódio:").unwrap());

    let chosen = &episodes[ep_idx];
    println!("Carregando episódio {}...", chosen.number);

    let stream_url = match get_stream_url(&chosen.id).await {
        Ok(url) => url,
        Err(e) => {
            eprintln!("Erro ao obter URL de streaming: {}", e);
            return Ok(());
        }
    };
    
    println!("Abrindo reprodutor MPV...");
    match play_with_mpv(&stream_url) {
        Ok(_) => println!("Reprodução concluída!"),
        Err(e) => eprintln!("Erro ao reproduzir vídeo: {}", e),
    }
    
    Ok(())
}