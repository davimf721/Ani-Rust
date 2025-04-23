use anyhow::{anyhow, Result, Context};
use std::process::{Command, Stdio};
use std::path::Path;

// Estrutura para opções do player
#[derive(Debug, Clone)]
pub struct PlayerOptions {
    pub fullscreen: bool,
    pub no_detach: bool,
    pub exit_after_play: bool,
    pub skip_intro: bool,
    pub skip_title: Option<String>,
    pub nextep_countdown: bool,
}

impl Default for PlayerOptions {
    fn default() -> Self {
        Self {
            fullscreen: true,
            no_detach: false,
            exit_after_play: false,
            skip_intro: false,
            skip_title: None,
            nextep_countdown: false,
        }
    }
}

// Função para reproduzir vídeo com MPV
pub fn play_with_mpv(stream_url: &str) -> Result<()> {
    // Verificar se mpv está instalado
    let mpv_path = find_mpv()?;
    
    println!("Iniciando MPV...");
    
    // Configurar argumentos para o MPV
    let mut args = vec![
        "--no-terminal",     // Não usa o terminal para output
        "--msg-level=all=info", // Nível de log informativo
        "--hwdec=no",        // Desativa aceleração de hardware
        "--vo=x11",          // Usa o driver de saída X11 (sem aceleração)
        "--gpu-context=x11", // Contexto X11 para GPU
        "--opengl-backend=x11", // Backend OpenGL X11
        "--fs",              // Inicia em tela cheia
        "--force-window=yes", // Força a abertura da janela
        "--keep-open=yes",   // Mantém a janela aberta após o término
        "--ytdl=no",         // Desativa o uso interno do youtube-dl
    ];
    
    // Adicionar URL do stream
    args.push(stream_url);
    
    // Executar MPV
    let status = Command::new(&mpv_path)  // Usando referência aqui para não mover o valor
        .args(&args)
        .spawn()
        .context("Falha ao iniciar o MPV")?
        .wait()
        .context("Erro durante a reprodução no MPV")?;
        
    if !status.success() {
        // Tenta uma abordagem alternativa se o MPV falhar
        println!("MPV encontrou um problema, tentando método alternativo...");
        
        // Tenta com ffplay como alternativa
        if let Ok(ffplay_path) = find_ffplay() {
            println!("Tentando reproduzir com ffplay...");
            Command::new(ffplay_path)
                .args(&[
                    "-autoexit",
                    "-fs",
                    "-nodisp",           // Desativa a exibição de informações
                    "-vf", "format=yuv420p", // Força formato de pixel compatível
                    "-x", "1280",        // Largura fixa
                    "-y", "720",         // Altura fixa
                    "-sws_flags", "bilinear", // Algoritmo de escala simples
                    "-loglevel", "warning", // Reduz logs
                    stream_url
                ])
                .spawn()
                .context("Falha ao iniciar ffplay")?
                .wait()
                .context("Erro durante a reprodução no ffplay")?;
        } else {
            // Se ffplay não estiver disponível, tenta baixar o vídeo e reproduzir localmente
            println!("ffplay não encontrado. Tentando baixar o vídeo e reproduzir localmente...");
            
            let temp_file = "/tmp/anirust_video.mp4";
            
            // Baixa o vídeo
            let download_status = Command::new("curl")
                .args(&["-L", "-o", temp_file, stream_url])
                .spawn()
                .context("Falha ao iniciar download com curl")?
                .wait()
                .context("Erro durante o download do vídeo")?;
                
            if download_status.success() {
                // Reproduz o arquivo baixado
                Command::new(&mpv_path)  // Usando referência aqui para não mover o valor
                    .args(&[
                        "--no-terminal",
                        "--hwdec=no",    // Desativa aceleração de hardware
                        "--vo=x11",      // Usa o driver de saída X11 (sem aceleração)
                        "--fs",
                        temp_file
                    ])
                    .spawn()
                    .context("Falha ao iniciar MPV com arquivo local")?
                    .wait()
                    .context("Erro durante a reprodução do arquivo local")?;
                    
                // Remove o arquivo temporário
                std::fs::remove_file(temp_file).ok();
            }
        }
    }
    
    Ok(())
}

// Função para reproduzir vídeo com VLC
pub fn play_with_vlc(stream_url: &str) -> Result<()> {
    // Verificar se vlc está instalado
    let vlc_path = find_vlc()?;
    
    println!("Iniciando VLC...");
    
    // Executar VLC
    let status = Command::new(&vlc_path)  // Usando referência aqui para não mover o valor
        .args(&[
            "--fullscreen",
            "--no-video-title-show",
            stream_url
        ])
        .spawn()
        .context("Falha ao iniciar o VLC")?
        .wait()
        .context("Erro durante a reprodução no VLC")?;
        
    if !status.success() {
        return Err(anyhow!("VLC retornou com erro"));
    }
    
    Ok(())
}

// Função para baixar vídeo
pub fn download_video(stream_url: &str, output_path: &str) -> Result<()> {
    println!("Baixando vídeo para: {}", output_path);
    
    // Criar diretório de saída se não existir
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Baixar o vídeo com curl
    let status = Command::new("curl")
        .args(&[
            "-L",
            "-o", output_path,
            stream_url
        ])
        .spawn()
        .context("Falha ao iniciar download com curl")?
        .wait()
        .context("Erro durante o download do vídeo")?;
        
    if !status.success() {
        return Err(anyhow!("Download falhou"));
    }
    
    println!("Download concluído: {}", output_path);
    Ok(())
}

// Funções auxiliares para encontrar executáveis de players

fn find_mpv() -> Result<String> {
    // Verificar se mpv está instalado via flatpak
    let flatpak_check = Command::new("flatpak")
        .args(&["info", "io.mpv.Mpv"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    
    if let Ok(status) = flatpak_check {
        if status.success() {
            return Ok("flatpak run io.mpv.Mpv".to_string());
        }
    }
    
    // Verificar se mpv está instalado normalmente
    let which_check = Command::new("which")
        .arg("mpv")
        .output()?;
    
    if which_check.status.success() {
        let path = String::from_utf8(which_check.stdout)?;
        return Ok(path.trim().to_string());
    }
    
    // Se não encontrar, retornar erro
    Err(anyhow!("mpv não encontrado"))
}

fn find_vlc() -> Result<String> {
    let which_check = Command::new("which")
        .arg("vlc")
        .output()?;
    
    if which_check.status.success() {
        let path = String::from_utf8(which_check.stdout)?;
        return Ok(path.trim().to_string());
    }
    
    // Se não encontrar, retornar erro
    Err(anyhow!("vlc não encontrado"))
}

fn find_ffplay() -> Result<String> {
    let which_check = Command::new("which")
        .arg("ffplay")
        .output()?;
    
    if which_check.status.success() {
        let path = String::from_utf8(which_check.stdout)?;
        return Ok(path.trim().to_string());
    }
    
    // Se não encontrar, retornar erro
    Err(anyhow!("ffplay não encontrado"))
}
