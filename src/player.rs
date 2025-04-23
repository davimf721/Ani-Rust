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
    println!("Iniciando reprodução do vídeo...");
    
    // Tentar reproduzir diretamente com MPV primeiro
    if let Ok(mpv_path) = find_mpv() {
        println!("Usando MPV para reprodução...");
        
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
        match Command::new(&mpv_path)
            .args(&args)
            .spawn()
            .and_then(|mut child| child.wait()) {
            Ok(_) => return Ok(()),
            Err(e) => println!("Erro ao usar MPV: {}", e),
        }
    } else {
        println!("MPV não encontrado, tentando alternativas...");
    }
    
    // Se MPV falhar ou não estiver disponível, tentar ffplay
    if let Ok(ffplay_path) = find_ffplay() {
        println!("Tentando reproduzir com ffplay...");
        match Command::new(&ffplay_path)
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
            .and_then(|mut child| child.wait()) {
            Ok(_) => return Ok(()),
            Err(e) => println!("Erro ao usar ffplay: {}", e),
        }
    } else {
        println!("ffplay não encontrado, tentando alternativas...");
    }
    
    // Se tudo falhar, tentar baixar o vídeo e reproduzir localmente
    println!("Tentando baixar o vídeo e reproduzir localmente...");
    
    let temp_file = "/tmp/anirust_video.mp4";
    
    // Verificar se o diretório /tmp existe
    if !Path::new("/tmp").exists() {
        std::fs::create_dir_all("/tmp").context("Falha ao criar diretório temporário")?;
    }
    
    // Baixar o vídeo
    match Command::new("curl")
        .args(&["-L", "-o", temp_file, stream_url])
        .spawn()
        .and_then(|mut child| child.wait()) {
        Ok(status) => {
            if status.success() {
                println!("Download concluído, tentando reproduzir o arquivo local...");
                
                // Tentar reproduzir com MPV novamente
                if let Ok(mpv_path) = find_mpv() {
                    match Command::new(&mpv_path)
                        .args(&[
                            "--no-terminal",
                            "--hwdec=no",    // Desativa aceleração de hardware
                            "--vo=x11",      // Usa o driver de saída X11 (sem aceleração)
                            "--fs",
                            temp_file
                        ])
                        .spawn()
                        .and_then(|mut child| child.wait()) {
                        Ok(_) => {
                            // Remover o arquivo temporário
                            let _ = std::fs::remove_file(temp_file);
                            return Ok(());
                        },
                        Err(e) => println!("Erro ao reproduzir arquivo local com MPV: {}", e),
                    }
                }
                
                // Se MPV falhar, tentar com ffplay
                if let Ok(ffplay_path) = find_ffplay() {
                    match Command::new(&ffplay_path)
                        .args(&[
                            "-autoexit",
                            "-fs",
                            temp_file
                        ])
                        .spawn()
                        .and_then(|mut child| child.wait()) {
                        Ok(_) => {
                            // Remover o arquivo temporário
                            let _ = std::fs::remove_file(temp_file);
                            return Ok(());
                        },
                        Err(e) => println!("Erro ao reproduzir arquivo local com ffplay: {}", e),
                    }
                }
                
                // Se tudo falhar, pelo menos informar onde o arquivo foi baixado
                println!("Não foi possível reproduzir o vídeo, mas ele foi baixado em: {}", temp_file);
                return Ok(());
            } else {
                return Err(anyhow!("Falha ao baixar o vídeo"));
            }
        },
        Err(e) => {
            return Err(anyhow!("Erro ao iniciar download: {}", e));
        }
    }
}

// Função para reproduzir vídeo com VLC
pub fn play_with_vlc(stream_url: &str) -> Result<()> {
    // Verificar se vlc está instalado
    match find_vlc() {
        Ok(vlc_path) => {
            println!("Iniciando VLC...");
            
            // Executar VLC
            match Command::new(&vlc_path)
                .args(&[
                    "--fullscreen",
                    "--no-video-title-show",
                    stream_url
                ])
                .spawn()
                .and_then(|mut child| child.wait()) {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Erro ao executar VLC: {}", e)),
            }
        },
        Err(_) => {
            // Se VLC não estiver disponível, usar MPV como fallback
            println!("VLC não encontrado, usando MPV como alternativa...");
            play_with_mpv(stream_url)
        }
    }
}

// Função para baixar vídeo
pub fn download_video(stream_url: &str, output_path: &str) -> Result<()> {
    println!("Baixando vídeo para: {}", output_path);
    
    // Criar diretório de saída se não existir
    if let Some(parent) = Path::new(output_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Baixar o vídeo com curl
    match Command::new("curl")
        .args(&[
            "-L",
            "-o", output_path,
            stream_url
        ])
        .spawn()
        .and_then(|mut child| child.wait()) {
        Ok(status) => {
            if status.success() {
                println!("Download concluído: {}", output_path);
                Ok(())
            } else {
                Err(anyhow!("Download falhou"))
            }
        },
        Err(e) => Err(anyhow!("Erro ao iniciar download: {}", e)),
    }
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
    match Command::new("which")
        .arg("mpv")
        .output() {
        Ok(output) => {
            if output.status.success() {
                match String::from_utf8(output.stdout) {
                    Ok(path) => return Ok(path.trim().to_string()),
                    Err(_) => return Err(anyhow!("Erro ao converter caminho do MPV")),
                }
            }
            Err(anyhow!("mpv não encontrado"))
        },
        Err(_) => {
            // Tentar caminhos comuns
            for path in &["/usr/bin/mpv", "/usr/local/bin/mpv", "/bin/mpv"] {
                if Path::new(path).exists() {
                    return Ok(path.to_string());
                }
            }
            Err(anyhow!("mpv não encontrado"))
        }
    }
}

fn find_vlc() -> Result<String> {
    match Command::new("which")
        .arg("vlc")
        .output() {
        Ok(output) => {
            if output.status.success() {
                match String::from_utf8(output.stdout) {
                    Ok(path) => return Ok(path.trim().to_string()),
                    Err(_) => return Err(anyhow!("Erro ao converter caminho do VLC")),
                }
            }
            Err(anyhow!("vlc não encontrado"))
        },
        Err(_) => {
            // Tentar caminhos comuns
            for path in &["/usr/bin/vlc", "/usr/local/bin/vlc", "/bin/vlc"] {
                if Path::new(path).exists() {
                    return Ok(path.to_string());
                }
            }
            Err(anyhow!("vlc não encontrado"))
        }
    }
}

fn find_ffplay() -> Result<String> {
    match Command::new("which")
        .arg("ffplay")
        .output() {
        Ok(output) => {
            if output.status.success() {
                match String::from_utf8(output.stdout) {
                    Ok(path) => return Ok(path.trim().to_string()),
                    Err(_) => return Err(anyhow!("Erro ao converter caminho do ffplay")),
                }
            }
            Err(anyhow!("ffplay não encontrado"))
        },
        Err(_) => {
            // Tentar caminhos comuns
            for path in &["/usr/bin/ffplay", "/usr/local/bin/ffplay", "/bin/ffplay"] {
                if Path::new(path).exists() {
                    return Ok(path.to_string());
                }
            }
            Err(anyhow!("ffplay não encontrado"))
        }
    }
}
