use std::process::Command;
use anyhow::{Context, Result, anyhow};
use clap::{Parser, Subcommand};
use tokio;

mod api;
mod extractor;
mod player;
mod ui;
mod history;
mod utils;

use api::{search_anime, get_anime_episodes};
use extractor::get_episode_url;
use player::play_with_mpv;
use ui::select_from_list;

#[derive(Parser)]
#[command(name = "anirust", version, about = "Anime CLI em Rust")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Anime search query
    query: Option<String>,

    /// Specify the video quality
    #[arg(short, long)]
    quality: Option<String>,

    /// Specify the episode number
    #[arg(short, long)]
    episode: Option<String>,

    /// Use VLC to play the video
    #[arg(short, long)]
    vlc: bool,

    /// Download the video instead of playing it
    #[arg(short, long)]
    download: bool,

    /// Continue watching from history
    #[arg(short, long)]
    continue_watching: bool,

    /// Play dubbed version
    #[arg(long)]
    dub: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Update the application
    Update,
    
    /// Delete history
    Delete,
    
    /// Show logs
    Logview,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Check dependencies
    check_dependencies()?;
    
    match &cli.command {
        Some(Commands::Update) => {
            println!("Update functionality not implemented yet");
            return Ok(());
        },
        Some(Commands::Delete) => {
            println!("Delete history functionality not implemented yet");
            return Ok(());
        },
        Some(Commands::Logview) => {
            println!("Logview functionality not implemented yet");
            return Ok(());
        },
        None => {
            // Continue with normal execution
        }
    }
    
    // Set default quality
    let quality = cli.quality.unwrap_or_else(|| "best".to_string());
    
    // Set mode (sub or dub)
    let mode = if cli.dub { "dub" } else { "sub" };
    
    if cli.continue_watching {
        println!("Continue watching functionality not implemented yet");
        return Ok(());
    }
    
    // Get search query
    let query = match cli.query {
        Some(q) => q,
        None => {
            ui::prompt_input("Enter anime search query: ")?
        }
    };
    
    // Search for anime
    println!("Searching for: {}", query);
    let search_results = search_anime(&query).await?;
    
    if search_results.is_empty() {
        return Err(anyhow!("No anime found for query: {}", query));
    }
    
    // Display results and let user select
    let titles: Vec<String> = search_results.iter()
        .map(|anime| anime.title.clone())
        .collect();
    
    let selected_index = select_from_list(&titles, "Select anime:")?;
    let selected_anime = &search_results[selected_index];
    
    println!("Selected: {}", selected_anime.title);
    
    // Get episodes
    let episodes = get_anime_episodes(selected_anime.mal_id).await?;
    
    if episodes.is_empty() {
        return Err(anyhow!("No episodes found for: {}", selected_anime.title));
    }
    
    // Let user select episode or use provided episode number
    let episode_index = match &cli.episode {
        Some(ep_str) => {
            let ep_num = ep_str.parse::<usize>().context("Invalid episode number")?;
            if ep_num == 0 || ep_num > episodes.len() {
                return Err(anyhow!("Episode {} not available. Total episodes: {}", ep_num, episodes.len()));
            }
            ep_num - 1 // Convert to 0-based index
        },
        None => {
            let episode_titles: Vec<String> = episodes.iter()
                .map(|ep| format!("Episode {}", ep.url))
                .collect();
            
            select_from_list(&episode_titles, "Select episode:")?
        }
    };
    
    let selected_episode = &episodes[episode_index];
    println!("Selected: Episode {}", selected_episode.url);
    
    // Get video URL
    let video_url = get_episode_url(&selected_anime.mal_id.to_string(), &selected_episode.url, mode, &quality).await?;
    println!("Video URL: {}", video_url);
    
    // Play or download
    if cli.download {
        println!("Download functionality not implemented yet");
        return Ok(());
    } else {
        // Play with selected player
        if cli.vlc {
            println!("VLC playback not implemented yet");
            return Ok(());
        } else {
            play_with_mpv(&video_url)?;
        }
    }
    
    Ok(())
}

fn check_dependencies() -> Result<()> {
    // Check for curl
    if Command::new("which").arg("curl").output()?.status.success() == false {
        return Err(anyhow!("curl is not installed. Please install it."));
    }
    
    // Check for mpv
    if Command::new("which").arg("mpv").output()?.status.success() == false {
        println!("Warning: mpv is not installed. Some functionality may be limited.");
    }
    
    Ok(())
}
